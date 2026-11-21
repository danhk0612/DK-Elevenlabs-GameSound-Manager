"""Start a Linux engine credential session without key argv/env.
Explicit --config supports the user-authorized NAS file workflow.
Interactive key input requires a terminal; automation may supply a private pipe FD.
The engine stays in the foreground; other CLI/MCP clients use its DK_SOUND_ROOT.
"""
import argparse
import getpass
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import warnings
import resource


CONFIG_LIMIT = 8192


def unique_object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('duplicate field')
        value[key] = item
    return value


def load_config(path):
    # No permission-mode requirement: Windows/NAS mounts can synthesize mode 0777.
    # NAS share ACLs control the user's explicitly chosen plaintext settings file.
    flags = os.O_RDONLY | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_NONBLOCK', 0)
    fd = os.open(path, flags)
    with os.fdopen(fd, 'rb') as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError('config must be a regular file')
        raw = stream.read(CONFIG_LIMIT + 1)
    if len(raw) > CONFIG_LIMIT:
        raise ValueError('config too large')
    config = json.loads(raw.decode('utf-8-sig'), object_pairs_hook=unique_object,
                        parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite number')))
    fields = {'schema_version','account_id','api_key','enable_generation','request_limit','credit_limit','session_seconds','state_root'}
    if not isinstance(config, dict) or set(config) != fields or type(config['schema_version']) is not int or config['schema_version'] != 1:
        raise ValueError('invalid config fields')
    if any(type(config[k]) is not int for k in ('request_limit','credit_limit','session_seconds')) or type(config['enable_generation']) is not bool:
        raise ValueError('invalid config types')
    if any(not isinstance(config[k], str) for k in ('account_id','api_key','state_root')):
        raise ValueError('invalid config text')
    return config


def default_engine():
    app = Path(__file__).resolve().parents[1]
    for profile in ('debug','release'):
        path = app/'target'/profile/'dk-sound-engine'
        if path.is_file():
            return path
    raise ValueError('engine binary unavailable')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--engine', type=Path)
    parser.add_argument('--config', type=Path, help='Explicit local NAS JSON file; secret is never printed')
    parser.add_argument('--check-config', action='store_true', help='Validate only; no engine, files, or HTTP')
    parser.add_argument('--account-id')
    parser.add_argument('--session-seconds', type=int)
    parser.add_argument('--request-limit', type=int)
    parser.add_argument('--credit-limit', type=int)
    parser.add_argument('--enable-generation', action='store_true')
    parser.add_argument('--key-fd', type=int, help='Read key from a private inherited pipe/socket FD; never a key filename')
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('Linux only')
    child = None
    try:
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        key = None
        if args.config is not None:
            if args.key_fd is not None or args.enable_generation or any(v is not None for v in (args.account_id,args.session_seconds,args.request_limit,args.credit_limit)):
                raise ValueError('config and inline settings are exclusive')
            config = load_config(args.config)
            args.account_id = config['account_id']
            args.session_seconds = config['session_seconds']
            args.request_limit = config['request_limit']
            args.credit_limit = config['credit_limit']
            args.enable_generation = config['enable_generation']
            key = config.pop('api_key')
            state = config['state_root']
            # NAS file, Docker-local private runtime. No NAS SQLite/mode assumptions.
            root = Path(state) if state else Path.home()/'.local'/'state'/'dk-gamesound'
            if not root.is_absolute():
                raise ValueError('state_root must be absolute')
            del config
        elif args.check_config:
            raise ValueError('check-config requires config')
        if type(args.session_seconds) is not int or not 60 <= args.session_seconds <= 86400 or type(args.request_limit) is not int or not 0 <= args.request_limit <= 1000000 or type(args.credit_limit) is not int or not 0 <= args.credit_limit <= 1000000000:
            raise ValueError('invalid explicit session limits')
        if args.enable_generation and (not args.request_limit or not args.credit_limit):
            raise ValueError('generation requires positive limits')
        if not args.account_id or len(args.account_id) > 64 or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789_-' for c in args.account_id):
            raise ValueError('invalid account ID')
        if key is None and args.key_fd is None:
            if not sys.stdin.isatty():
                raise ValueError('terminal or --key-fd private pipe required')
            with warnings.catch_warnings():
                warnings.simplefilter('error', getpass.GetPassWarning)
                key = getpass.getpass('ElevenLabs API key (not saved): ')
        elif key is None:
            mode = os.fstat(args.key_fd).st_mode
            if not (stat.S_ISFIFO(mode) or stat.S_ISSOCK(mode)):
                raise ValueError('key FD must be a pipe/socket')
            with os.fdopen(os.dup(args.key_fd), 'rb') as stream:
                raw = stream.read(259)
            if len(raw) > 258:
                raise ValueError('key input exceeds bound')
            key = raw.decode('ascii').removesuffix('\n').removesuffix('\r')
            del raw
        if not key or len(key) > 256 or any(not 33 <= ord(c) <= 126 for c in key):
            raise ValueError('invalid key input')
        if args.check_config:
            print(json.dumps({'ok': True, 'engine_started': False, 'network_requests': 0}))
            return 0
        if args.config is not None:
            # Create only runtime parents after complete offline validation.
            root.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            os.environ['DK_SOUND_ROOT'] = str(root)
        if not os.environ.get('DK_SOUND_ROOT'):
            raise ValueError('DK_SOUND_ROOT is required')
        if args.engine is None:
            args.engine = default_engine()
        if not args.engine.is_absolute() or not args.engine.is_file():
            raise ValueError('engine must be an existing absolute executable')
        payload = json.dumps({'account_id': args.account_id, 'key': key, 'enabled': args.enable_generation,
                              'request_limit': args.request_limit, 'credit_limit': args.credit_limit,
                              'session_seconds': args.session_seconds}, separators=(',', ':')).encode('utf-8')
        del key
        # No secret in process arguments/environment, stdout, stderr or temporary file.
        child = subprocess.Popen([str(args.engine), '--session'], stdin=subprocess.PIPE)
        try:
            child.stdin.write(payload)
            child.stdin.close()
        except BrokenPipeError:
            pass
        del payload
        return child.wait()
    except KeyboardInterrupt:
        # Foreground session engine also receives the terminal signal. Never replay.
        print('session_interrupted', file=sys.stderr)
        return 130
    except Exception:
        # Never print exception text, input bytes or Python tracebacks containing key data.
        print('session_start_failed', file=sys.stderr)
        if child is not None and child.poll() is None:
            child.terminate()
            child.wait(timeout=10)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
