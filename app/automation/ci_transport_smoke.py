"""CI-only real Windows engine exercise, plus portable offline CLI preflight."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import wave

from sound_pipeline import Client


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', required=True, type=Path)
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--local-linux', action='store_true', help='Run real engine in a new private temporary Linux root')
    args = parser.parse_args()
    suffix = '.exe' if os.name == 'nt' else ''
    cli = (args.bin_dir / ('dk-sound-cli' + suffix)).resolve()
    mcp = (args.bin_dir / ('dk-sound-mcp' + suffix)).resolve()
    runner = Path(__file__).with_name('sound_pipeline.py').resolve()
    if not args.offline and not (args.local_linux and sys.platform == 'linux') and (os.name != 'nt' or os.environ.get('GITHUB_ACTIONS') != 'true'):
        raise RuntimeError('Native engine smoke is restricted to the isolated Windows Actions runner')
    with tempfile.TemporaryDirectory(prefix='DK client 한글 공백 ', ignore_cleanup_errors=True) as directory:
        root = Path(directory)
        settings = root / 'localappdata'
        settings.mkdir(mode=0o700)
        env = {**os.environ, 'LOCALAPPDATA': str(settings), 'DK_SOUND_ROOT': str(settings)}
        source = root / '합성 원본.wav'
        with wave.open(str(source), 'wb') as audio:
            audio.setnchannels(1)
            audio.setsampwidth(2)
            audio.setframerate(8000)
            audio.writeframes(b'\x00\x00' * 800)
        original = source.read_bytes()
        recipe = {'schema_version': 1, 'project': {'project_id': 'client-ci', 'name': '자동화 CI', 'workspace': str(root)},
                  'source': {'asset_id': 'hit', 'version_id': 'original', 'kind': 'sfx', 'name': 'Hit', 'source_path': str(source), 'source_note': 'synthesized fixture'},
                  'candidate': {'version_id': 'take-1', 'source_path': str(source), 'source_note': 'synthesized candidate'},
                  'review': {'review_id': 'review-1', 'summary': '검토 대기'}}
        recipe_file = root / '작업 정의.json'
        def run_recipe(extra=(), expect=True):
            recipe_file.write_text(json.dumps(recipe, ensure_ascii=False), encoding='utf-8')
            try:
                result = subprocess.run([sys.executable, str(runner), str(recipe_file), '--cli', str(cli), '--request-seconds', '15', *extra],
                                        env=env, capture_output=True, timeout=60)
            except subprocess.TimeoutExpired as error:
                print((error.stderr or b'').decode('utf-8', errors='replace'), file=sys.stderr)
                raise
            data = json.loads(result.stdout)
            if data['ok'] is not expect:
                print(result.stderr.decode('utf-8', errors='replace'), file=sys.stderr)
            assert data['ok'] is expect, data
            assert result.returncode == (0 if expect else 1), result.returncode
            return data
        run_recipe(['--validate-only', '--mcp', str(mcp)])
        assert not list(settings.iterdir()), 'offline validation started engine or wrote catalog'
        invalid = dict(recipe['review'])
        recipe['review']['approved'] = True
        run_recipe(expect=False)
        recipe['review'] = invalid
        assert not list(settings.iterdir())
        recipe['source']['kind'] = 'invalid-kind'
        assert run_recipe(expect=False)['error']['code'] == 'invalid_kind'
        recipe['source']['kind'] = 'sfx'
        assert not list(settings.iterdir())
        if args.offline:
            # Native MCP handshake/discovery and malformed request checks need no engine.
            client = Client(cli, mcp)
            try:
                client.discover({'asset.file', 'review.submit', 'audio.submit'})
                try:
                    client.call({'protocol_version': 1, 'op': 'review.submit', 'project_id': 'client-ci',
                                 'review_id': 'review-1', 'asset_id': 'hit', 'version_id': 'take-1', 'expected_sha256': 'bad', 'summary': 'bad'})
                except Exception as error:
                    assert str(error) == 'invalid_sha256', str(error)
                else:
                    raise AssertionError('malformed hash accepted')
            finally:
                client.close()
            print('Offline recipe preflight / CLI validation / native MCP session: PASS')
            return
        try:
            first = run_recipe()
            stopped = subprocess.run([str(cli), 'shutdown'], env=env, capture_output=True, timeout=15)
            assert json.loads(stopped.stdout)['ok'] is True
            time.sleep(.5)
            # MCP now independently autostarts the engine from the same catalog.
            second = run_recipe(['--mcp', str(mcp)])
            assert first['data']['review'] == second['data']['review'], 'transport changed review record'
            assert second['data']['check']['matches'] is True
            assert source.read_bytes() == original
            assert first['data']['review']['snapshot']['candidate']['sha256'] == hashlib.sha256(original).hexdigest()
            # Closing the recipe's MCP client never removes records; a new client sees the same state.
            old_linux_root = os.environ.get('DK_SOUND_ROOT')
            os.environ['DK_SOUND_ROOT'] = str(settings)
            old_localappdata = os.environ.get('LOCALAPPDATA')
            os.environ['LOCALAPPDATA'] = str(settings)
            client = Client(cli, mcp)
            try:
                client.discover({'review.get', 'asset.history'})
                record = client.call({'protocol_version': 1, 'op': 'review.get', 'project_id': 'client-ci', 'review_id': 'review-1'})
                assert record == first['data']['review']
            finally:
                client.close()
                if old_linux_root is None: os.environ.pop('DK_SOUND_ROOT', None)
                else: os.environ['DK_SOUND_ROOT'] = old_linux_root
                if old_localappdata is None: os.environ.pop('LOCALAPPDATA', None)
                else: os.environ['LOCALAPPDATA'] = old_localappdata
            source.write_bytes(b'changed synthetic source')
            failure = run_recipe(['--mcp', str(mcp)], expect=False)
            assert failure['context']['stage'] == 'source'
            managed = root / 'dk-gamesound-client-ci' / 'originals' / 'hit' / 'original' / 'source.wav'
            assert managed.read_bytes() == original
            print('Native real engine: CLI/MCP recipe parity, reconnect, hash preservation, changed-source refusal: PASS')
        finally:
            subprocess.run([str(cli), 'shutdown'], env=env, capture_output=True, timeout=15)
            time.sleep(.5)


if __name__ == '__main__':
    main()
