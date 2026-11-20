"""CI-only Windows batch exercise; --offline checks preflight on any host."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import wave


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--bin-dir', required=True, type=Path)
    parser.add_argument('--offline', action='store_true')
    parser.add_argument('--local-linux', action='store_true', help='Run real engine in a new private temporary Linux root')
    args = parser.parse_args()
    suffix = '.exe' if os.name == 'nt' else ''
    cli = (args.bin_dir / ('dk-sound-cli' + suffix)).resolve()
    mcp = (args.bin_dir / ('dk-sound-mcp' + suffix)).resolve()
    script = Path(__file__).with_name('sound_batch.py').resolve()
    if not args.offline and not (args.local_linux and sys.platform == 'linux') and (os.name != 'nt' or os.environ.get('GITHUB_ACTIONS') != 'true'):
        raise RuntimeError('Native batch smoke is restricted to the isolated Windows Actions runner')
    with tempfile.TemporaryDirectory(prefix='DK batch 한글 공백 ', ignore_cleanup_errors=True) as directory:
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
        recipe = {'schema_version': 1, 'project': {'project_id': 'batch-ci', 'name': '다건 자동화 CI', 'workspace': str(root)},
                  'source': {'asset_id': 'hit', 'version_id': 'original', 'kind': 'sfx', 'name': 'Hit', 'source_path': str(source), 'source_note': 'synthesized fixture'},
                  'candidate': {'version_id': 'take-a', 'source_path': str(source), 'source_note': 'synthesized candidate'},
                  'review': {'review_id': 'batch-review-a', 'summary': '검토 대기'}}
        def item(iid):
            value = copy.deepcopy(recipe)
            value['candidate']['version_id'] = 'take-' + iid
            value['review']['review_id'] = 'batch-review-' + iid
            return {'item_id': iid, 'recipe': value}
        manifest = {'schema_version': 1, 'batch_id': 'native-batch', 'items': [item('a'), item('b')]}
        path = root / '다건 작업.json'
        def run(value=manifest, extra=(), expected=0):
            path.write_text(json.dumps(value, ensure_ascii=False), encoding='utf-8')
            try:
                result = subprocess.run([sys.executable, str(script), str(path), '--cli', str(cli), '--request-seconds', '15', *extra],
                                        env=env, capture_output=True, timeout=120)
            except subprocess.TimeoutExpired as error:
                print((error.stderr or b'').decode('utf-8', errors='replace'), file=sys.stderr)
                raise
            lines = [json.loads(line) for line in result.stdout.splitlines()]
            if result.returncode != expected:
                print(result.stderr.decode('utf-8', errors='replace'), file=sys.stderr)
                raise AssertionError(lines)
            assert lines[-1]['ok'] is (expected == 0), lines
            return lines
        run(extra=['--validate-only'])
        bad = copy.deepcopy(manifest)
        bad['items'][1]['recipe']['source']['kind'] = 'invalid-kind'
        bad['items'][1]['recipe']['source']['asset_id'] = 'other'
        failed = run(bad, expected=1)[-1]
        assert failed['context']['item_id'] == 'b' and failed['error']['code'] == 'invalid_kind', failed
        duplicate = copy.deepcopy(manifest)
        duplicate['items'][1]['recipe']['review']['review_id'] = 'batch-review-a'
        assert run(duplicate, ['--only', 'a'], expected=1)[-1]['error']['code'] == 'batch_review_id_conflict'
        assert run(extra=['--only', 'missing'], expected=1)[-1]['error']['code'] == 'batch_unknown_selected_item'
        assert not list(settings.iterdir()), 'batch preflight wrote catalog or started engine'
        if args.offline:
            print('Offline batch preflight, later-item core validation and collisions: PASS')
            return
        def command(op, **kwargs):
            request = json.dumps({'protocol_version': 1, 'op': op, **kwargs}).encode('utf-8')
            response = subprocess.run([str(cli), 'request'], input=request, env=env, capture_output=True, timeout=20)
            return json.loads(response.stdout)
        try:
            selected = run(extra=['--only', 'b'])[-1]
            assert [r['status'] for r in selected['data']['items']] == ['not_selected', 'succeeded']
            assert command('review.get', project_id='batch-ci', review_id='batch-review-a')['ok'] is False
            first = run()[-1]
            assert first['data']['counts']['succeeded'] == 2
            second_lines = run(extra=['--mcp', str(mcp), '--jsonl'])
            assert [line['type'] for line in second_lines] == ['item','item','summary']
            assert second_lines[-1]['data']['counts']['succeeded'] == 2
            assert 'items' not in second_lines[-1]['data']
            assert first['data']['manifest_sha256'] == second_lines[-1]['data']['manifest_sha256']
            receipts = first['data']['items']
            assert [r['candidate_sha256'] for r in receipts] == [line['item']['candidate_sha256'] for line in second_lines[:-1]]
            assert all(r['candidate_sha256'] == hashlib.sha256(original).hexdigest() for r in receipts)
            assert command('review.withdraw', project_id='batch-ci', review_id='batch-review-a', reason='synthetic CI withdrawal')['ok'] is True
            mixed = {'schema_version': 1, 'batch_id': 'mixed-batch', 'items': [item('a'), item('c')]}
            stopped = run(mixed, expected=1)[-1]
            assert [r['status'] for r in stopped['data']['items']] == ['failed','not_run'], stopped
            assert command('review.get', project_id='batch-ci', review_id='batch-review-c')['ok'] is False
            continued = run(mixed, ['--mcp', str(mcp), '--keep-going', '--jsonl'], expected=1)
            assert [r['item']['status'] for r in continued[:-1]] == ['failed','succeeded'], continued
            assert continued[-1]['data']['counts']['failed'] == 1
            assert continued[-1]['ok'] is False
            assert source.read_bytes() == original
            assert (root/'dk-gamesound-batch-ci/originals/hit/original/source.wav').read_bytes() == original
            print('Native real engine batch: shared source, selection, CLI/MCP parity, JSONL, fail-stop/keep-going, preservation: PASS')
        finally:
            subprocess.run([str(cli), 'shutdown'], env=env, capture_output=True, timeout=15)
            time.sleep(.5)


if __name__ == '__main__':
    main()
