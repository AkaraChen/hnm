"""Exercise the command hook protocol and both runtime registrations."""

import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
HOOK = ROOT / 'templates/.codex/hooks/grilling_check.py'


class GrillingHookTests(unittest.TestCase):
    def run_hook(self, raw):
        result = subprocess.run([sys.executable, str(HOOK)], input=raw,
                                text=True, capture_output=True, check=True)
        self.assertEqual(result.stderr, '')
        return result.stdout

    def test_user_turn_without_transcript(self):
        output = json.loads(self.run_hook(json.dumps({
            'hook_event_name': 'UserPromptSubmit', 'prompt': 'Use your judgment.'
        })))
        self.assertEqual(set(output), {'hookSpecificOutput'})
        event = output['hookSpecificOutput']
        self.assertEqual(set(event), {'hookEventName', 'additionalContext'})
        self.assertEqual(event['hookEventName'], 'UserPromptSubmit')
        self.assertTrue(event['additionalContext'].isascii())
        self.assertIn('[grilling-check v3b]', event['additionalContext'])

    def test_invalid_input_and_unrelated_events_are_noops(self):
        for raw in ['', '{', 'null', '[]', '1', '{}',
                    '{"hook_event_name":"Stop","prompt":"hi"}',
                    '{"hook_event_name":"UserPromptSubmit","prompt":null}']:
            with self.subTest(raw=raw):
                self.assertEqual(self.run_hook(raw), '')

    def test_existing_prds_and_missing_transcripts_do_not_suppress(self):
        raw = json.dumps({'hook_event_name': 'UserPromptSubmit', 'prompt': 'Yes',
                          'cwd': str(ROOT), 'transcript_path': '/missing/transcript'})
        self.assertTrue(self.run_hook(raw))

    def test_registrations_and_dogfood_copy(self):
        self.assertEqual(HOOK.read_bytes(), (ROOT / '.codex/hooks/grilling_check.py').read_bytes())
        for config in ['.claude/settings.json', '.codex/hooks.json']:
            template = json.loads((ROOT / 'templates' / config).read_text())
            installed = json.loads((ROOT / config).read_text())
            self.assertEqual(template['hooks']['UserPromptSubmit'],
                             installed['hooks']['UserPromptSubmit'])
            registrations = template['hooks']['UserPromptSubmit']
            self.assertEqual(len(registrations), 1)
            self.assertIn('grilling_check.py', registrations[0]['hooks'][0]['command'])
            self.assertIn('PreToolUse', template['hooks'])


if __name__ == '__main__':
    unittest.main()
