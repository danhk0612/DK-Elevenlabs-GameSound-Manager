import unittest
from unittest.mock import patch
from sound_starter import compile_starter, main


class Starter(unittest.TestCase):
    def test_reusable_pack_cost_and_dialogue_snapshots(self):
        voices = {kind: 'fixture-' + kind for kind in ('energetic', 'calm', 'sturdy')}
        plan = compile_starter('one', 'demo', 'first', voices, 2000)
        self.assertEqual(plan['reserved_credits'], 1800)
        self.assertEqual(plan['generation_requests'], 8)
        self.assertEqual(plan['mode'], 'plan_only')
        self.assertEqual(plan, compile_starter('one', 'demo', 'first', voices, 2000))
        self.assertEqual(len({c['job_id'] for c in plan['commands'] if c['op'] == 'job.submit'}), 8)
        dialogues = {c['dialogue_id']: c for c in plan['commands'] if c['op'] == 'dialogue.put'}
        for job in plan['commands']:
            if job.get('generation_kind') == 'tts':
                self.assertEqual(dialogues[job['dialogue_id']]['text'], job['text'])
                self.assertEqual(job['language'], 'ko')

    def test_selected_music_needs_no_voice_and_no_transport(self):
        plan = compile_starter('one', 'demo', 'first', {}, 750, ['music-battle'])
        self.assertEqual(len(plan['commands']), 1)
        self.assertEqual(plan['reserved_credits'], 750)
        self.assertTrue(plan['commands'][0]['force_instrumental'])

    def test_fail_closed_budget_selection_and_voice_mapping(self):
        for voices, cap, only in [({}, 0, None), ({}, True, None), ({}, 10001, None),
                                  ({}, 749, ['music-battle']), ({}, 2000, ['calm-hit']),
                                  ({}, 2000, []), ({}, 2000, ['unknown']),
                                  ({}, 2000, ['music-battle', 'music-battle']),
                                  ({'calm': ''}, 2000, ['calm-hit'])]:
            with self.assertRaises(ValueError):
                compile_starter('one', 'demo', 'first', voices, cap, only)

    @patch('sound_starter.subprocess.run')
    def test_budget_failure_never_launches_native_client(self, run):
        with patch('builtins.print'):
            self.assertEqual(main(['--project', 'one', '--account', 'demo', '--run', 'first',
                                   '--max-reserved', '1', '--only', 'music-battle', '--cli', 'unused']), 1)
        run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
