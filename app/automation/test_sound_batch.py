import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from sound_batch import Failure, compile_batch, load_manifest, run_batch


class Batch(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='DK batch 한글 ')
        self.addCleanup(self.temp.cleanup)
        root = Path(self.temp.name)
        self.source = root / 'synthetic.wav'
        self.source.write_bytes(b'synthetic fixture only')
        recipe = {'schema_version': 1, 'project': {'project_id': 'one', 'name': 'Test', 'workspace': str(root)},
                  'source': {'asset_id': 'hit', 'version_id': 'original', 'name': 'Hit', 'kind': 'sfx', 'source_path': str(self.source), 'source_note': 'synthetic'},
                  'candidate': {'version_id': 'take-a', 'source_path': str(self.source), 'source_note': 'candidate fixture'},
                  'review': {'review_id': 'review-a', 'summary': 'Listen'}}
        other = copy.deepcopy(recipe)
        other['candidate']['version_id'] = 'take-b'
        other['review']['review_id'] = 'review-b'
        self.manifest = {'schema_version': 1, 'batch_id': 'test-batch',
                         'items': [{'item_id': 'a', 'recipe': recipe}, {'item_id': 'b', 'recipe': other}]}

    def test_shared_source_is_valid_and_native_validations_are_deduplicated(self):
        compiled, requests, digest = compile_batch(self.manifest)
        self.assertEqual(len(compiled), 2)
        self.assertEqual(len(requests), 12)  # Two seven-request recipes share project/source.
        self.assertEqual(len(digest), 64)
        reordered = json.loads(json.dumps(self.manifest, sort_keys=True))
        self.assertEqual(compile_batch(reordered)[2], digest)
        changed = copy.deepcopy(self.manifest)
        changed['items'][1]['recipe']['review']['summary'] = 'Different'
        self.assertNotEqual(compile_batch(changed)[2], digest)

    def test_conflicting_project_source_candidate_review_and_items_rejected(self):
        for field, change, code in [
            ('project', {'name': 'Other'}, 'batch_project_conflict'),
            ('source', {'source_note': 'Other'}, 'batch_source_conflict'),
            ('candidate', {'version_id': 'take-a'}, 'batch_candidate_conflict'),
            ('review', {'review_id': 'review-a'}, 'batch_review_id_conflict'),
        ]:
            bad = copy.deepcopy(self.manifest)
            bad['items'][1]['recipe'][field].update(change)
            context = {}
            with self.assertRaisesRegex(Failure, code):
                compile_batch(bad, context)
            self.assertEqual(context['item_id'], 'b')
        bad = copy.deepcopy(self.manifest)
        bad['items'][1]['item_id'] = 'a'
        with self.assertRaisesRegex(Failure, 'duplicate_batch_item'):
            compile_batch(bad)

    def test_global_operation_and_review_ids_cannot_be_hidden_in_another_project(self):
        bad = copy.deepcopy(self.manifest)
        for item in bad['items']:
            recipe = item['recipe']
            recipe['tools'] = {'ffmpeg_path': str(self.source), 'ffprobe_path': str(self.source)}
            recipe['analysis'] = {'operation_id': 'same-analysis', 'waveform_bins': 32, 'silence_db': -60, 'silence_min_ms': 20, 'clip_level': .99}
        bad['items'][1]['recipe']['project']['project_id'] = 'two'
        with self.assertRaisesRegex(Failure, 'batch_operation_id_conflict'):
            compile_batch(bad)
        bad['items'][1]['recipe']['analysis']['operation_id'] = 'other-analysis'
        bad['items'][1]['recipe']['review']['review_id'] = 'review-a'
        with self.assertRaisesRegex(Failure, 'batch_review_id_conflict'):
            compile_batch(bad)

    def test_candidate_cannot_be_another_items_import_source(self):
        bad = copy.deepcopy(self.manifest)
        # Make the first item consume b's candidate. Shared source definitions
        # already reject this dependency, before either item can execute.
        bad['items'][0]['recipe']['source']['version_id'] = 'take-b'
        with self.assertRaisesRegex(Failure, 'batch_source_conflict|batch_cross_item_dependency'):
            compile_batch(bad)

    def test_manifest_limits_unknown_fields_duplicate_keys_and_nonfinite_json(self):
        for mutation in [lambda b: b.update(items=[]), lambda b: b.update(schema_version=True),
                         lambda b: b.update(items=b['items']*17), lambda b: b.update(approved=True),
                         lambda b: b.update(batch_id='../escape')]:
            bad = copy.deepcopy(self.manifest)
            mutation(bad)
            with self.assertRaises(Failure): compile_batch(bad)
        path = Path(self.temp.name) / 'manifest.json'
        for raw in [b'{"batch_id":"one","batch_id":"two"}', b'{"n":NaN}', b'x'*(1024*1024+1)]:
            path.write_bytes(raw)
            with self.assertRaises(Failure): load_manifest(path)

    @staticmethod
    def output(context):
        return {'review': {'snapshot': {'candidate': {'sha256': 'a'*64}}, 'state': 'pending'}, 'check': {'matches': True}}

    def run_fake(self, failure=None, keep=False, selected=None):
        compiled = compile_batch(self.manifest)[0]
        created, closed, events = [], [], []
        class Client:
            def __init__(_, item): created.append(item['item_id']); _.id = item['item_id']
            def close(_): closed.append(_.id)
        def execute(client, plan, wait_seconds, poll_seconds, context, on_progress=None):
            context.update(stage='candidate', operation_id='operation-a')
            if on_progress: on_progress({'stage': 'candidate', 'op': 'audio.submit'})
            if client.id == 'a' and failure: raise failure
            return self.output(context)
        with patch('sound_batch.execute', side_effect=execute):
            data, code = run_batch(compiled, selected or {'a','b'}, Client, 1, .1, keep,
                                   on_progress=events.append, on_item=lambda row: None)
        return data, code, created, closed, events

    def test_success_selection_and_progress_keep_item_identity(self):
        data, code, created, closed, events = self.run_fake(selected={'b'})
        self.assertEqual(code, 0)
        self.assertEqual([row['status'] for row in data['items']], ['not_selected','succeeded'])
        self.assertEqual(created, closed)
        self.assertEqual(created, ['b'])
        self.assertEqual(events[0]['item_id'], 'b')
        self.assertEqual(data['approval'], 'not_granted')
        self.assertNotIn('source_path', data['items'][1])

    def test_default_stops_but_keep_going_allows_only_terminal_known_errors(self):
        data, code, created, closed, _ = self.run_fake(Failure('version_id_conflict'))
        self.assertEqual(code, 1)
        self.assertEqual(created, ['a'])
        self.assertEqual([row['status'] for row in data['items']], ['failed','not_run'])
        self.assertEqual(data['items'][0]['operation_id'], 'operation-a')
        data, code, created, closed, _ = self.run_fake(Failure('version_id_conflict'), keep=True)
        self.assertEqual(created, ['a','b'])
        self.assertEqual(created, closed)
        self.assertEqual(data['counts']['succeeded'], 1)
        self.assertEqual(code, 1)  # Continuing never converts a partial failure to success.

    def test_uncertain_outcome_and_unknown_error_always_stop_even_with_keep_going(self):
        for error in ['engine_response_timeout','transport_eof','mcp_response_id_mismatch',
                      'wait_deadline_work_may_continue','future_unknown_error']:
            data, code, created, closed, _ = self.run_fake(Failure(error), keep=True)
            self.assertEqual(code, 1)
            self.assertEqual(created, ['a'])
            self.assertEqual(closed, ['a'])
            self.assertEqual(data['counts']['not_run'], 1)

    def test_interrupt_closes_client_reports_ids_and_never_advances(self):
        data, code, created, closed, _ = self.run_fake(KeyboardInterrupt(), keep=True)
        self.assertEqual(code, 130)
        self.assertEqual(created, closed)
        self.assertEqual([row['status'] for row in data['items']], ['interrupted','not_run'])
        self.assertEqual(data['items'][0]['operation_id'], 'operation-a')


if __name__ == '__main__':
    unittest.main()
