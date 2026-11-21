"""Explicit NAS credential file parsing and offline preflight; no real credentials."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

if sys.platform == 'linux':
    import linux_session


@unittest.skipUnless(sys.platform == 'linux', 'Linux session launcher')
class ConfigTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.path = self.root/'elevenlabs.local.json'
        self.config = {'schema_version':1,'account_id':'file-test','api_key':'synthetic-config-key',
                       'enable_generation':False,'request_limit':0,'credit_limit':0,'session_seconds':60,'state_root':''}
        self.script = Path(__file__).with_name('linux_session.py')
    def tearDown(self):
        self.temp.cleanup()
    def write(self, value=None):
        self.path.write_text(json.dumps(self.config if value is None else value), encoding='utf-8-sig', newline='\r\n')
    def check(self, success):
        p = subprocess.run([sys.executable,str(self.script),'--config',str(self.path),'--check-config'],capture_output=True,timeout=10)
        self.assertEqual(p.returncode, 0 if success else 1)
        self.assertNotIn(self.config['api_key'].encode(), p.stdout+p.stderr)
        self.assertEqual(sorted(x.name for x in self.root.iterdir()), ['elevenlabs.local.json'])
        if success:
            self.assertEqual(json.loads(p.stdout), {'ok':True,'engine_started':False,'network_requests':0})
    def test_windows_encoding_and_nas_mode_are_supported_without_engine(self):
        self.write(); self.path.chmod(0o777)
        self.assertEqual(linux_session.load_config(self.path),self.config)
        self.check(True)
    def test_missing_or_invalid_fields_and_numeric_types_are_rejected(self):
        for key,value in [('api_key',''),('enable_generation',True),('request_limit',True),('session_seconds',False),('state_root','relative')]:
            data=copy.deepcopy(self.config); data[key]=value
            self.write(data); self.check(False)
        self.write({**self.config,'unknown':'synthetic-config-key'}); self.check(False)
    def test_duplicate_nonfinite_and_size_errors_never_echo_inputs(self):
        raw=json.dumps(self.config)
        for text in [raw[:-1]+',"api_key":"other"}',raw.replace('"request_limit": 0','"request_limit": NaN'), 'X'*8193, '{"api_key":"synthetic-config-key",bad}']:
            self.path.write_text(text); self.check(False)
    def test_symlink_and_mixed_cli_settings_are_rejected(self):
        self.write()
        target=self.root/'target.json'; self.path.rename(target); self.path.symlink_to(target)
        with self.assertRaises(OSError): linux_session.load_config(self.path)
        self.path.unlink(); target.rename(self.path)
        p=subprocess.run([sys.executable,str(self.script),'--config',str(self.path),'--account-id','override','--check-config'],capture_output=True,timeout=10)
        self.assertEqual(p.returncode,1)
        self.assertNotIn(self.config['api_key'].encode(),p.stdout+p.stderr)


if __name__=='__main__':
    unittest.main()
