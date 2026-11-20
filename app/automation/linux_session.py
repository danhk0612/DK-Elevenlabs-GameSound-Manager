"""Start a Linux engine credential session without key argv/env/files.
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--engine', required=True, type=Path)
    parser.add_argument('--account-id', required=True)
    parser.add_argument('--session-seconds', required=True, type=int)
    parser.add_argument('--request-limit', required=True, type=int)
    parser.add_argument('--credit-limit', required=True, type=int)
    parser.add_argument('--enable-generation', action='store_true')
    parser.add_argument('--key-fd', type=int, help='Read key from a private inherited pipe/socket FD; never a key filename')
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('Linux only')
    if not args.engine.is_absolute() or not args.engine.is_file():
        parser.error('--engine must be an existing absolute executable path')
    if not 60 <= args.session_seconds <= 86400 or not 0 <= args.request_limit <= 1000000 or not 0 <= args.credit_limit <= 1000000000:
        parser.error('invalid explicit session limits')
    if args.enable_generation and (not args.request_limit or not args.credit_limit):
        parser.error('generation requires positive request and credit limits')
    if not args.account_id or len(args.account_id) > 64 or any(c not in 'abcdefghijklmnopqrstuvwxyz0123456789_-' for c in args.account_id):
        parser.error('invalid account ID')
    if not os.environ.get('DK_SOUND_ROOT'):
        parser.error('DK_SOUND_ROOT is required')
    child = None
    try:
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        if args.key_fd is None:
            if not sys.stdin.isatty():
                raise ValueError('terminal or --key-fd private pipe required')
            with warnings.catch_warnings():
                warnings.simplefilter('error', getpass.GetPassWarning)
                key = getpass.getpass('ElevenLabs API key (not saved): ')
        else:
            mode = os.fstat(args.key_fd).st_mode
            if not (stat.S_ISFIFO(mode) or stat.S_ISSOCK(mode)):
                raise ValueError('key FD must be a pipe/socket')
            with os.fdopen(os.dup(args.key_fd), 'rb') as stream:
                raw = stream.read(258)
            key = raw.decode('ascii').removesuffix('\n').removesuffix('\r')
            del raw
        if not key or len(key) > 256 or any(not 33 <= ord(c) <= 126 for c in key):
            raise ValueError('invalid key input')
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
