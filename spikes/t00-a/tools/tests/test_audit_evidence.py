import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile

MODULE = Path(__file__).resolve().parents[1] / "audit_evidence.py"
spec = importlib.util.spec_from_file_location("audit_evidence", MODULE)
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)
SHA = "a" * 40


def encode(value):
    return json.dumps(value).encode()


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="T00 한글 공백 ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.files = {"source-sha.txt": ("\ufeff" + SHA + "\r\n").encode()}
        self.records = {}
        for name, count in (("automatic", 16), ("lifecycle", 11), ("gui", 6)):
            report = {"source_sha": SHA, "result": "passed_automated_only",
                      "os": "Windows-2022Server-10.0.20348-SP0",
                      "checks": {str(i): "passed" for i in range(count)}}
            filename = "windows-" + name + ".json"
            self.files[filename] = encode(report)
            self.records[filename] = {"sha256": audit.digest(self.files[filename]),
                                      "automated_checks": count}
        filename = "windows-pipe-acl.json"
        self.files[filename] = encode({"source_sha": SHA, "result": "passed_automated_only",
            "account_cleanup": "passed: removed", "native_evidence": {
                "owner_sid": "owner", "client_sid": "other", "impersonated_sid": "other",
                "reverted_sid": "owner", "owner_open_error": 0,
                "permitted_control_open_error": 0, "protected_open_error": 5}})
        self.records[filename] = {"sha256": audit.digest(self.files[filename])}
        self.write_archive()

    def write_archive(self, extra=None, manifest_extra=""):
        manifest = {name: audit.digest(data) for name, data in self.files.items()}
        sums = "\ufeff" + "\r\n".join(value.upper() + "  " + name.replace("/", "\\")
                                             for name, value in manifest.items())
        self.archive = self.root / "증거.zip"
        with zipfile.ZipFile(self.archive, "w") as zf:
            for name, data in self.files.items():
                zf.writestr(name, data)
            zf.writestr("SHA256SUMS.txt", (sums + manifest_extra).encode())
            if extra:
                zf.writestr(*extra)
        receipt = {"source_sha": SHA, "zip_sha256": audit.digest(self.archive.read_bytes()),
                   "zip_bytes": self.archive.stat().st_size, "manifest": manifest,
                   "report_receipts": self.records, "run_id": 1, "artifact_id": 2,
                   "artifact_expires_at": "2026-12-04T09:49:13Z"}
        self.receipt = self.root / "receipt.json"
        self.receipt.write_bytes(encode(receipt))

    def run_audit(self, **kwargs):
        return audit.audit(self.archive, self.receipt, **kwargs)

    def user_inputs(self):
        automatic = json.loads(self.files["windows-automatic.json"])
        automatic["os"] = "Windows-11-10.0.26100-SP0"
        self.automatic = self.root / "user-automatic.json"
        self.automatic.write_bytes(encode(automatic))
        observation = {"source_sha": SHA, "executor": "tester", "os": "Windows 11 x64 build 26100",
                       "notes": "Non-admin; submit then close while busy; same PID/job; heard WAV."}
        for key in ("gui_visible", "gui_common_command", "close_job_reconnect", "synthetic_audio_audible"):
            observation[key] = "passed"
        self.observation = self.root / "user-observation.json"
        self.observation.write_bytes(encode(observation))
        return {"user_automatic": self.automatic, "user_observation": self.observation}

    def test_missing_user_stays_pending_and_no_files_changed(self):
        before = {p.name: p.read_bytes() for p in self.root.iterdir()}
        result = self.run_audit()
        self.assertEqual(result["status"], "pending")
        self.assertEqual(result["ci_checks_passed"], 33)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir()})

    def test_full_submission_requires_human_review_not_completion(self):
        result = self.run_audit(**self.user_inputs())
        self.assertEqual(result["status"], "manual_review_required")
        self.assertFalse(result["task_completed"])
        self.assertFalse(result["t00_b_authorized"])

    def test_media_ended_does_not_prove_listening(self):
        args = self.user_inputs()
        observation = json.loads(self.observation.read_bytes())
        observation["synthetic_audio_audible"] = {"ended": True}
        self.observation.write_bytes(encode(observation))
        self.assertIn("user_observation.synthetic_audio_audible", self.run_audit(**args)["pending"])

    def test_tampered_zip_rejected(self):
        with self.archive.open("ab") as stream:
            stream.write(b"tampered")
        with self.assertRaisesRegex(audit.AuditError, "digest"):
            self.run_audit()

    def test_unsafe_and_duplicate_members_rejected(self):
        for name in ("../escape", "C:/escape", "source-sha.txt", "SOURCE-SHA.TXT"):
            with self.subTest(name=name):
                self.write_archive(extra=(name, b"bad"))
                with self.assertRaises(audit.AuditError):
                    self.run_audit()

    def test_uncovered_member_rejected(self):
        self.write_archive(extra=("extra.txt", b"bad"))
        with self.assertRaisesRegex(audit.AuditError, "coverage"):
            self.run_audit()

    def test_manifest_tampering_rejected(self):
        self.files["source-sha.txt"] = b"b" * 40
        self.write_archive()
        with self.assertRaisesRegex(audit.AuditError, "source SHA"):
            self.run_audit()

    def test_user_wrong_sha_and_missing_check_rejected(self):
        args = self.user_inputs()
        original = json.loads(self.automatic.read_bytes())
        for change in ("sha", "check", "failed"):
            document = copy.deepcopy(original)
            if change == "sha":
                document["source_sha"] = "b" * 40
            elif change == "check":
                del document["checks"]["0"]
            else:
                document["checks"]["0"] = "failed"
            self.automatic.write_bytes(encode(document))
            with self.assertRaises(audit.AuditError):
                self.run_audit(**args)

    def test_ci_server_cannot_be_submitted_as_user_result(self):
        self.automatic = self.root / "user-automatic.json"
        self.automatic.write_bytes(self.files["windows-automatic.json"])
        with self.assertRaisesRegex(audit.AuditError, "desktop"):
            self.run_audit(user_automatic=self.automatic)

    def test_failed_and_blank_human_observations(self):
        args = self.user_inputs()
        document = json.loads(self.observation.read_bytes())
        document["synthetic_audio_audible"] = "failed"
        self.observation.write_bytes(encode(document))
        with self.assertRaises(audit.AuditError):
            self.run_audit(**args)
        document["synthetic_audio_audible"] = ""
        self.observation.write_bytes(encode(document))
        self.assertEqual(self.run_audit(**args)["status"], "pending")

    def test_duplicate_json_keys_rejected(self):
        with self.assertRaises(audit.AuditError):
            audit.object_json(b'{"checks":{},"checks":{}}')

    def test_cli_exit_codes_and_private_input_not_echoed(self):
        def cli(*extra):
            result = subprocess.run([sys.executable, str(MODULE), "--zip", str(self.archive),
                "--receipt", str(self.receipt), *extra], capture_output=True, text=True)
            return result, json.loads(result.stdout)
        result, body = cli()
        self.assertEqual(result.returncode, 2)
        self.assertEqual(body["status"], "pending")
        self.user_inputs()
        result, body = cli("--user-automatic", str(self.automatic), "--user-observation", str(self.observation))
        self.assertEqual(result.returncode, 0)
        self.observation.write_bytes(b'{"PRIVATE_DATA":')
        result, body = cli("--user-observation", str(self.observation))
        self.assertEqual(result.returncode, 1)
        self.assertEqual(body["status"], "invalid")
        self.assertNotIn("PRIVATE_DATA", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
