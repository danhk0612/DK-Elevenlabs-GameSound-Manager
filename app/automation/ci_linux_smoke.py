"""Real Linux headless isolation/lifecycle/mock generation and optional audio smoke.
Only new private temporary data and synthesized audio are used. No live provider.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time
import wave


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', required=True, type=Path)
    parser.add_argument('--ffmpeg', type=Path)
    parser.add_argument('--ffprobe', type=Path)
    args = parser.parse_args()
    if sys.platform != 'linux':
        raise RuntimeError('Linux only')
    if bool(args.ffmpeg) != bool(args.ffprobe):
        parser.error('both --ffmpeg and --ffprobe are required')
    cli = (args.bin_dir / 'dk-sound-cli').resolve()
    engine = cli.with_name('dk-sound-engine')
    with tempfile.TemporaryDirectory(prefix='dk-linux-') as directory:
        workspace = Path(directory)
        root = workspace / 'state'
        env = {**os.environ, 'DK_SOUND_ROOT': str(root)}
        def request(op, expect=True, override=None, **fields):
            raw = json.dumps({'protocol_version': 1, 'op': op, **fields}).encode()
            p = subprocess.run([str(cli), 'request'], input=raw, capture_output=True, env=override or env, timeout=25)
            data = json.loads(p.stdout)
            assert data['ok'] is expect, data
            assert p.returncode == (0 if expect else 1), p.returncode
            return data['data'] if expect else data['error']['code']
        def stop():
            for _ in range(100):
                subprocess.run([str(cli), 'shutdown'], env=env, capture_output=True, timeout=20)
                if not (root/'engine.sock').exists(): return
                time.sleep(.05)
            raise AssertionError('socket was not cleaned after shutdown')
        missing = dict(env)
        missing.pop('DK_SOUND_ROOT')
        assert request('status', expect=False, override=missing) == 'linux_root_required'
        relative = {**env, 'DK_SOUND_ROOT': 'relative'}
        assert request('status', expect=False, override=relative) == 'linux_root_must_be_absolute'
        root.mkdir(mode=0o755)
        assert request('status', expect=False) == 'linux_root_not_private'
        assert not list(root.iterdir())
        root.chmod(0o700)
        linked = workspace/'linked'
        linked.symlink_to(root, target_is_directory=True)
        assert request('status', expect=False, override={**env, 'DK_SOUND_ROOT': str(linked)}) == 'linux_root_not_private'
        # An unexpected object at the endpoint is preserved, never unlinked.
        endpoint = root/'engine.sock'
        endpoint.write_bytes(b'preserve conflict')
        p = subprocess.run([str(engine)], env=env, capture_output=True, timeout=10)
        assert p.returncode == 1 and p.stderr.strip() == b'socket_path_conflict', p.stderr
        assert endpoint.read_bytes() == b'preserve conflict'
        endpoint.unlink()
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(endpoint))
        stale.close()
        try:
            with ThreadPoolExecutor(max_workers=8) as pool:
                statuses = list(pool.map(lambda _: request('status'), range(8)))
            pid = statuses[0]['engine_pid']
            assert os.getpgid(pid) != os.getpgrp(), 'autostart engine inherited client signal group'
            assert all(s['engine_pid'] == pid for s in statuses), statuses
            assert endpoint.stat().st_mode & 0o777 == 0o600
            duplicate = subprocess.run([str(engine)], env=env, capture_output=True, timeout=10)
            assert duplicate.returncode == 1 and b'engine_already_owned_or_lock_failed' in duplicate.stderr
            request('project.create', project_id='linux-ci', name='Linux automation', workspace=str(workspace))
            request('account.create', account_id='mock', provider='mock')
            request('budget.set', account_id='mock', request_limit=1, credit_limit=20)
            job = dict(project_id='linux-ci', account_id='mock', job_id='mock-1', asset_id='mock-sfx', version_id='take-1', name='Synthetic only', generation_kind='sfx', text='Synthetic fixture', model_id='mock-sfx', output_format='mp3_44100_128', reserve_credits=20, duration_ms=500)
            request('job.submit', **job)
            for _ in range(200):
                result = request('job.get', project_id='linux-ci', job_id='mock-1')
                if result['state'] == 'succeeded': break
                assert result['state'] in ('queued', 'running', 'output_ready'), result
                time.sleep(.05)
            else: raise AssertionError(result)
            file = request('asset.file', project_id='linux-ci', asset_id='mock-sfx', version_id='take-1')
            request('job.submit', **job)  # Reuses the ID even after the one-request limit.
            assert request('job.submit', expect=False, **{**job, 'job_id': 'mock-2', 'asset_id': 'other'}) == 'budget_exceeded'
            os.kill(pid, signal.SIGKILL)
            time.sleep(.2)
            assert request('status')['engine_pid'] != pid
            assert request('job.get', project_id='linux-ci', job_id='mock-1')['state'] == 'succeeded'
            assert request('asset.file', project_id='linux-ci', asset_id='mock-sfx', version_id='take-1') == file
            print('Linux native root/endpoint protection, concurrent singleton, stale/crash restart, mock queue/budget/no replay: PASS')
            if args.ffmpeg:
                source = workspace/'합성 원본.wav'
                with wave.open(str(source), 'wb') as audio:
                    audio.setnchannels(1); audio.setsampwidth(2); audio.setframerate(8000)
                    audio.writeframes(b'\x00\x10' * 8000)
                before = hashlib.sha256(source.read_bytes()).hexdigest()
                recipe = {'schema_version': 1, 'project': {'project_id': 'audio-ci', 'name': 'Native audio', 'workspace': str(workspace)},
                          'source': {'asset_id': 'tone', 'version_id': 'original', 'kind': 'sfx', 'name': 'Synthesized', 'source_path': str(source), 'source_note': 'Synthetic fixture'},
                          'tools': {'ffmpeg_path': str(args.ffmpeg.resolve()), 'ffprobe_path': str(args.ffprobe.resolve())},
                          'candidate': {'version_id': 'edited', 'operation_id': 'edit-linux', 'start_ms': 0, 'end_ms': 500, 'gain_db': -3, 'fade_in_ms': 5, 'fade_out_ms': 25, 'output_format': 'wav_pcm_s16le', 'sample_rate': 8000, 'channels': 1},
                          'analysis': {'operation_id': 'analyze-linux', 'waveform_bins': 32, 'silence_db': -60, 'silence_min_ms': 20, 'clip_level': .999},
                          'review': {'review_id': 'audio-linux', 'summary': 'Synthetic listening candidate'}}
                path = workspace/'recipe.json'
                path.write_text(json.dumps(recipe), encoding='utf-8')
                script = Path(__file__).with_name('sound_pipeline.py')
                for transport in ([], ['--mcp', str(cli.with_name('dk-sound-mcp'))]):
                    p = subprocess.run([sys.executable, str(script), str(path), '--cli', str(cli), *transport], env=env, capture_output=True, timeout=60)
                    data = json.loads(p.stdout)
                    assert p.returncode == 0 and data['ok'], (data, p.stderr)
                    assert data['data']['check']['matches']
                assert hashlib.sha256(source.read_bytes()).hexdigest() == before
                print('Linux native CLI/MCP async FFmpeg edit, analysis, pinned candidate review, rerun/source preservation: PASS')
        finally:
            stop()


if __name__ == '__main__':
    main()
