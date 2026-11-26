import copy
import io
import os
import threading
import json
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from sound_pipeline import Client, Failure, Process, compile_recipe, decode, execute


class Recipes(unittest.TestCase):
    def setUp(self):
        progress = patch('sys.stderr', new_callable=io.StringIO)
        progress.start()
        self.addCleanup(progress.stop)
        self.temp = tempfile.TemporaryDirectory(prefix='DK automation 한글 ')
        root = Path(self.temp.name)
        source = root / '합성 소리.wav'
        source.write_bytes(b'synthetic test bytes')
        self.recipe = {'schema_version': 1, 'project': {'project_id': 'one', 'name': 'Test', 'workspace': str(root)},
                       'source': {'asset_id': 'hit', 'version_id': 'original', 'name': 'Hit', 'kind': 'sfx', 'source_path': str(source), 'source_note': 'synthetic'},
                       'candidate': {'version_id': 'take-1', 'source_path': str(source), 'source_note': 'synthetic candidate'},
                       'review': {'review_id': 'review-1', 'summary': 'Listen'}}

    def tearDown(self):
        self.temp.cleanup()

    def test_local_recipe_has_no_network_approval_or_game_operation(self):
        plan, validations = compile_recipe(self.recipe)
        self.assertEqual([s for s, _ in plan], ['project', 'source', 'candidate', 'file', 'review', 'check', 'history'])
        self.assertEqual(len(validations), 7)
        self.assertTrue(all(c.get('project_id') == 'one' for c in validations))
        self.assertEqual(plan[2][1]['parent_version_id'], 'original')
        for field in ['api_key', 'approved', 'output_path', 'job']:
            invalid = copy.deepcopy(self.recipe)
            invalid[field] = 'disallowed'
            with self.assertRaises(Failure):
                compile_recipe(invalid)

    def test_invalid_paths_fields_versions_and_analysis_fail_before_execution(self):
        for mutate in [lambda r: r['candidate'].update(version_id='original'),
                       lambda r: r['review'].update(approved=True),
                       lambda r: r['source'].update(source_path='relative.wav'),
                       lambda r: r.update(schema_version=True),
                       lambda r: r.update(analysis={'operation_id': 'analysis'}),
                       lambda r: r.update(tools={})]:
            invalid = copy.deepcopy(self.recipe)
            mutate(invalid)
            with self.assertRaises(Failure):
                compile_recipe(invalid)

    def edited(self):
        recipe = copy.deepcopy(self.recipe)
        recipe['candidate'] = {'version_id': 'take-1', 'operation_id': 'edit-1', 'start_ms': 0, 'end_ms': 100,
                               'gain_db': -3, 'fade_in_ms': 0, 'fade_out_ms': 5, 'output_format': 'wav_pcm_s16le', 'sample_rate': 8000, 'channels': 1}
        recipe['analysis'] = {'operation_id': 'analysis-1', 'waveform_bins': 32, 'silence_db': -60, 'silence_min_ms': 20, 'clip_level': .99}
        recipe['tools'] = {'ffmpeg_path': recipe['source']['source_path'], 'ffprobe_path': recipe['source']['source_path']}
        return recipe

    def test_async_pipeline_polls_same_id_and_binds_hash_and_analysis(self):
        plan, validations = compile_recipe(self.edited())
        self.assertEqual(len([c for c in validations if c['op'] == 'audio.get']), 2)
        calls = []
        class Fake:
            def call(_, c):
                calls.append(c)
                if c['op'] == 'audio.submit': return {'state': 'queued'}
                if c['op'] == 'audio.get': return {'state': 'succeeded'}
                if c['op'] == 'asset.file': return {'role': 'candidate', 'sha256': 'a' * 64}
                if c['op'] == 'review.submit': return {'state': 'pending'}
                if c['op'] == 'review.check': return {'matches': True}
                return {}
        with patch('sound_pipeline.time.sleep'):
            result = execute(Fake(), plan, 10, .1, {'project_id': 'one'})
        review = next(c for c in calls if c['op'] == 'review.submit')
        self.assertEqual(review['expected_sha256'], 'a' * 64)
        self.assertEqual(review['analysis_id'], 'analysis-1')
        self.assertEqual(result['approval'], 'not_granted')
        self.assertEqual([c['operation_id'] for c in calls if c['op'] == 'audio.get'], ['edit-1', 'analysis-1'])

    def test_interrupted_output_ready_failed_cancelled_do_not_resubmit_or_recover(self):
        plan, _ = compile_recipe(self.edited())
        for state in ['interrupted', 'output_ready', 'failed', 'cancelled']:
            calls = []
            class Fake:
                def call(_, c):
                    calls.append(c)
                    return {'state': state} if c['op'] == 'audio.submit' else {}
            with self.assertRaisesRegex(Failure, state):
                execute(Fake(), plan, 1, .1, {'project_id': 'one'})
            self.assertEqual(sum(c['op'] == 'audio.submit' for c in calls), 1)
            self.assertFalse(any(c['op'] in ['audio.recover', 'review.submit'] for c in calls))

        class Queued:
            def call(_, c): return {'state': 'queued'}
        with patch('sound_pipeline.time.monotonic', side_effect=[0, 2]):
            with self.assertRaisesRegex(Failure, 'wait_deadline_work_may_continue'):
                execute(Queued(), plan, 1, .1, {'project_id': 'one'})

    def test_withdrawn_review_is_never_reopened(self):
        plan, _ = compile_recipe(self.recipe)
        class Fake:
            def call(_, c):
                if c['op'] == 'asset.file': return {'role': 'candidate', 'sha256': 'a'*64}
                if c['op'] == 'review.submit': return {'state': 'withdrawn'}
                return {}
        with self.assertRaisesRegex(Failure, 'review_not_pending'):
            execute(Fake(), plan, 1, .1, {'project_id': 'one'})


class Transport(unittest.TestCase):
    def test_bounded_framing_and_timeout(self):
        for code, expected in [("print('x'*65537,flush=True)", 'response_too_large'),
                               ("import time;time.sleep(3)", 'transport_timeout'),
                               ("print('garbage',flush=True)", 'transport_read_failed')]:
            process = Process([sys.executable, '-u', '-c', code])
            try:
                with self.assertRaisesRegex(Failure, expected):
                    process.receive(.5)
            finally:
                process.close()

    def test_descendant_inheriting_stdout_does_not_block_single_response_cleanup(self):
        code = "import subprocess,sys;subprocess.Popen([sys.executable,'-c','import time;time.sleep(4)']);print('{\"ok\":true}',flush=True)"
        process = Process([sys.executable, '-u', '-c', code], single_response=True)
        self.assertEqual(process.receive(3), {'ok': True})
        start = time.monotonic()
        process.close()
        self.assertLess(time.monotonic() - start, 2)
        # Continuous MCP-style reader also has a finite cleanup path. On Windows,
        # synchronous read cancellation must release its inherited-pipe wait.
        process = Process([sys.executable, '-u', '-c', code])
        self.assertEqual(process.receive(3), {'ok': True})
        start = time.monotonic()
        process.close()
        self.assertLess(time.monotonic() - start, 3)
        if os.name == 'nt':
            self.assertFalse(process.reader.is_alive())

    def test_unread_input_write_has_a_deadline(self):
        released = threading.Event()
        class Stream:
            def write(self, _):
                released.wait(3)
                return len(_)
            def flush(self): pass
            def close(self): pass
        class Child:
            stdin, stdout = Stream(), io.BytesIO(b'')
            def kill(self): released.set()
            def wait(self, **_): return 0
        with patch('sound_pipeline.subprocess.Popen', return_value=Child()):
            process = Process(['fixture'])
            try:
                with self.assertRaisesRegex(Failure, 'write_timeout'):
                    process.send({'payload': 'x' * 64000}, timeout=.1)
                self.assertTrue(released.is_set())
            finally:
                process.close()

    def test_mcp_mismatched_request_ids_and_failed_envelopes_are_rejected(self):
        class Session:
            def send(self, *_): pass
            def receive(self, _): return {'jsonrpc': '2.0', 'id': 99, 'result': {}}
        client = Client(Path('unused'))
        client.session = Session()
        with self.assertRaisesRegex(Failure, 'id_mismatch'):
            client.rpc('tools/list', {})
        self.assertRaises(Failure, decode, b'{"bad": NaN}')


if __name__ == '__main__':
    unittest.main()
