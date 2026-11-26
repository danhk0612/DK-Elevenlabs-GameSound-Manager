"""Real engine credential-session startup/lifetime regression; fake key, zero HTTP."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', required=True, type=Path)
    args = parser.parse_args()
    if sys.platform != 'linux':
        parser.error('Linux only')
    cli = (args.bin_dir/'dk-sound-cli').resolve()
    engine = cli.with_name('dk-sound-engine')
    fake = 'synthetic-key-not-for-any-service'
    with tempfile.TemporaryDirectory(prefix='dk-session-') as directory:
        root = Path(directory)/'state'
        root.mkdir(mode=0o700)
        env = {**os.environ, 'DK_SOUND_ROOT': str(root)}
        def call(op, **fields):
            p = subprocess.run([str(cli), 'request'], input=json.dumps({'protocol_version':1,'op':op,**fields}).encode(), env=env, capture_output=True, timeout=20)
            assert fake.encode() not in p.stdout+p.stderr
            return json.loads(p.stdout)
        for payload in [b'{', b'X'*4097, json.dumps({'key':fake}).encode()]:
            bad = subprocess.run([str(engine),'--session'], input=payload, env=env, capture_output=True, timeout=15)
            assert bad.returncode == 1 and not bad.stdout
            assert fake.encode() not in bad.stderr
            assert not list(root.iterdir()), 'invalid session started engine/catalog'
        # Reject a valid-looking maximum key/newline prefix with extra trailing bytes.
        rfd,wfd = os.pipe()
        os.write(wfd, b'X'*256+b'\r\nX'); os.close(wfd)
        launcher = Path(__file__).with_name('linux_session.py')
        rejected = subprocess.run([sys.executable,str(launcher),'--engine',str(engine),'--account-id','session-test',
                                   '--session-seconds','60','--request-limit','0','--credit-limit','0','--key-fd',str(rfd)],
                                  env=env,pass_fds=(rfd,),capture_output=True,timeout=15)
        os.close(rfd)
        assert rejected.returncode == 1 and rejected.stderr.strip() == b'session_start_failed'
        assert not rejected.stdout and not list(root.iterdir())
        payload = {'account_id':'session-test','key':fake,'enabled':False,'request_limit':0,'credit_limit':0,'session_seconds':60}
        session = subprocess.Popen([str(engine),'--session'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
        try:
            session.stdin.write(json.dumps(payload).encode()); session.stdin.close()
            for _ in range(200):
                if (root/'engine.sock').exists(): break
                assert session.poll() is None, session.stderr.read()
                time.sleep(.05)
            else: raise AssertionError('session engine did not start')
            assert call('status')['data']['engine_pid'] == session.pid
            assert call('account.create',account_id='session-test',provider='elevenlabs')['data']['configured'] is True
            assert call('account.get',account_id='session-test')['data']['credential_configured'] is True
            assert call('account.create',account_id='other',provider='elevenlabs')['data']['configured'] is False
            assert call('budget.set',account_id='session-test',request_limit=1,credit_limit=10)['error']['code'] == 'live_policy_requires_gui'
            assert call('account.get',account_id='session-test',key=fake)['ok'] is False
            # No metadata/control call above performs an HTTP request.
            assert call('shutdown')['ok'] is True
            session.wait(timeout=15)
            assert fake.encode() not in session.stdout.read()+session.stderr.read()
            time.sleep(.2)
            assert call('account.get',account_id='session-test')['data']['credential_configured'] is False
            # A default restart never inherits a key; no plaintext was persisted.
            for path in root.rglob('*'):
                if path.is_file(): assert fake.encode() not in path.read_bytes(), path.name
            assert call('shutdown')['ok'] is True
            time.sleep(.2)
            # Launcher receives only a private inherited FD, not a key argv/env/file.
            rfd,wfd = os.pipe()
            os.write(wfd, fake.encode()); os.close(wfd)
            launcher = Path(__file__).with_name('linux_session.py')
            child = subprocess.Popen([sys.executable,str(launcher),'--engine',str(engine),'--account-id','session-test',
                                     '--session-seconds','60','--request-limit','0','--credit-limit','0','--key-fd',str(rfd)],
                                    env=env,pass_fds=(rfd,),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            os.close(rfd)
            try:
                for _ in range(200):
                    if (root/'engine.sock').exists(): break
                    assert child.poll() is None, child.stderr.read()
                    time.sleep(.05)
                assert call('account.get',account_id='session-test')['data']['credential_configured'] is True
                assert call('shutdown')['ok'] is True
                child.wait(timeout=15)
                assert child.returncode == 0
                assert fake.encode() not in child.stdout.read()+child.stderr.read()
            finally:
                if child.poll() is None: child.terminate(); child.wait(timeout=15)
            time.sleep(.2)
            # User-authorized Windows/NAS JSON mode: BOM/CRLF and synthesized 0777.
            config_path = Path(directory)/'elevenlabs.local.json'
            config = {'schema_version':1,'account_id':'session-test','api_key':fake,'enable_generation':False,
                      'request_limit':0,'credit_limit':0,'session_seconds':60,'state_root':str(root)}
            config_path.write_text(json.dumps(config),encoding='utf-8-sig',newline='\r\n')
            config_path.chmod(0o777)
            checked = subprocess.run([sys.executable,str(launcher),'--config',str(config_path),'--check-config'],env=env,capture_output=True,timeout=15)
            assert checked.returncode == 0 and json.loads(checked.stdout)['engine_started'] is False
            file_child = subprocess.Popen([sys.executable,str(launcher),'--config',str(config_path),'--engine',str(engine)],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            try:
                for _ in range(200):
                    if (root/'engine.sock').exists(): break
                    assert file_child.poll() is None, 'config session exited'
                    time.sleep(.05)
                assert call('account.get',account_id='session-test')['data']['credential_configured'] is True
                assert call('shutdown')['ok'] is True
                file_child.wait(timeout=15)
                assert file_child.returncode == 0
                assert fake.encode() not in file_child.stdout.read()+file_child.stderr.read()
                assert json.loads(config_path.read_text(encoding='utf-8-sig')) == config
                for path in root.rglob('*'):
                    if path.is_file(): assert fake.encode() not in path.read_bytes()
            finally:
                if file_child.poll() is None: file_child.terminate(); file_child.wait(timeout=15)
            print('Linux real engine private stdin session, account scope, no IPC key/policy, metadata-only, no persistence/restart, FD launcher: PASS')
        finally:
            if session.poll() is None:
                session.terminate(); session.wait(timeout=15)


if __name__ == '__main__':
    main()
