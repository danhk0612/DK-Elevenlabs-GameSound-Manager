"""Read-only T00-A evidence audit. Python 3.10+, standard library only."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import sys
import zipfile
import zlib


class AuditError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise AuditError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def object_json(data):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result
    result = json.loads(data.decode("utf-8-sig"), object_pairs_hook=unique)
    require(isinstance(result, dict), "JSON root must be an object")
    return result


def safe_name(name):
    name = name.replace("\\", "/")
    require(name and not name.startswith("/") and ":" not in name
            and all(part not in ("", ".", "..") for part in name.split("/")),
            "unsafe archive/manifest path")
    return str(PurePosixPath(name))


def passed_checks(report, expected_keys):
    checks = report.get("checks")
    require(isinstance(checks, dict) and set(checks) == set(expected_keys),
            "unexpected or missing check names")
    require(all(value == "passed" for value in checks.values()), "check failed")
    require(report.get("result") == "passed_automated_only", "unexpected report result")
    return len(checks)


def audit(archive, receipt, user_automatic=None, user_observation=None):
    """Receipt is trusted, previously reviewed metadata, not remote attestation."""
    raw_receipt = Path(receipt).read_bytes()
    expected = object_json(raw_receipt)
    sha = expected["source_sha"]
    require(re.fullmatch(r"[0-9a-f]{40}", sha) is not None, "invalid source SHA")
    require(Path(archive).stat().st_size <= 100_000_000, "ZIP too large")
    raw_zip = Path(archive).read_bytes()
    require(digest(raw_zip) == expected["zip_sha256"], "ZIP digest mismatch")
    require(len(raw_zip) == expected["zip_bytes"], "ZIP size mismatch")
    # Never extract or execute anything from the ZIP. Bound decompression work.
    import io
    with zipfile.ZipFile(io.BytesIO(raw_zip)) as zf:
        entries = zf.infolist()
        require(len(entries) <= 100, "too many ZIP entries")
        require(sum(e.file_size for e in entries) <= 100_000_000, "ZIP too large")
        files = {}
        folded = set()
        for entry in entries:
            name = safe_name(entry.filename)
            require(name.casefold() not in folded, "duplicate ZIP path")
            folded.add(name.casefold())
            require(not entry.is_dir() and not entry.flag_bits & 1, "unsupported ZIP entry")
            require((entry.external_attr >> 16) & 0o170000 != 0o120000, "ZIP symlink")
            files[name] = zf.read(entry)  # zipfile validates CRC here.
    manifest = {}
    for line in files["SHA256SUMS.txt"].decode("utf-8-sig").splitlines():
        match = re.fullmatch(r"([0-9a-fA-F]{64})  (.+)", line)
        require(match is not None, "invalid manifest line")
        name = safe_name(match[2])
        require(name not in manifest, "duplicate manifest path")
        manifest[name] = match[1].lower()
    require(set(files) - {"SHA256SUMS.txt"} == set(manifest), "manifest coverage mismatch")
    require(manifest == expected["manifest"], "manifest differs from trusted receipt")
    require(all(digest(files[name]) == value for name, value in manifest.items()),
            "manifest file digest mismatch")
    require(files["source-sha.txt"].decode("utf-8-sig").strip() == sha,
            "archive source SHA mismatch")
    reports = {}
    total = 0
    for name, record in expected["report_receipts"].items():
        require(digest(files[name]) == record["sha256"], "report digest mismatch")
        report = object_json(files[name])
        require(report.get("source_sha") == sha, "report source SHA mismatch")
        require(report.get("result") == "passed_automated_only", "report not passed")
        if name != "windows-pipe-acl.json":
            checks = report.get("checks", {})
            require(len(checks) == record["automated_checks"], "CI check count mismatch")
            total += passed_checks(report, checks)
        reports[name] = report
    require(total == 33, "expected 33 CI checks")
    native = reports["windows-pipe-acl.json"]["native_evidence"]
    require(native["owner_sid"] != native["client_sid"]
            and native["client_sid"] == native["impersonated_sid"]
            and native["owner_sid"] == native["reverted_sid"]
            and native["owner_open_error"] == 0
            and native["permitted_control_open_error"] == 0
            and native["protected_open_error"] == 5
            and reports["windows-pipe-acl.json"].get("account_cleanup", "").startswith("passed:"),
            "standalone pipe ACL evidence failed")
    pending = []
    receipts = {}
    for label, path in (("user_automatic", user_automatic), ("user_observation", user_observation)):
        if path is None:
            pending.append(label)
            continue
        data = Path(path).read_bytes()
        document = object_json(data)
        require(document.get("source_sha") == sha, label + " source SHA mismatch")
        receipts[label] = {"sha256": digest(data), "bytes": len(data)}
        if label == "user_automatic":
            passed_checks(document, reports["windows-automatic.json"]["checks"])
            os_name = document.get("os", "")
            require(isinstance(os_name, str) and os_name.startswith("Windows-")
                    and "server" not in os_name.lower(), "user Windows desktop evidence required")
            require(data != files["windows-automatic.json"], "CI report is not user evidence")
        else:
            # Existing collector fields are preserved. Notes and OS require human review.
            for key in ("executor", "os", "notes"):
                if not isinstance(document.get(key), str) or not document[key].strip() \
                        or document[key] in ("미검증", "unverified", "사용자 이름/별칭 기입",
                                             "화면·청감 관찰을 작성해 PR에 첨부"):
                    pending.append("user_observation." + key)
            for key in ("gui_visible", "gui_common_command", "close_job_reconnect",
                        "synthetic_audio_audible"):
                value = document.get(key)
                require(value not in ("failed", "실패"), "human observation reports failure: " + key)
                if value != "passed":
                    pending.append("user_observation." + key)
    return {
        "schema_version": 1,
        "status": "pending" if pending else "manual_review_required",
        "source_sha": sha,
        "run_id": expected["run_id"],
        "artifact_id": expected["artifact_id"],
        "artifact_expires_at": expected["artifact_expires_at"],
        "receipt_sha256": digest(raw_receipt),
        "zip_sha256": digest(raw_zip),
        "manifest_files_checked": len(manifest),
        "ci_checks_passed": total,
        "standalone_pipe_acl": "passed_not_engine_integration",
        "user_receipts": receipts,
        "pending": pending,
        "human_review": ["Confirm trusted receipt against GitHub run head/digest/conclusion/expiry",
                         "Confirm Windows 11 x64 non-admin OS/build and actual executor",
                         "Review GUI submit, close while busy, same engine/job reconnect observations",
                         "Confirm human audible WAV observation, not media ended events",
                         "Review D01-D06 and all T00-A acceptance conditions"],
        "task_completed": False,
        "t00_b_authorized": False,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zip", required=True, dest="archive")
    parser.add_argument("--receipt", required=True)
    parser.add_argument("--user-automatic")
    parser.add_argument("--user-observation")
    args = parser.parse_args()
    try:
        result = audit(**vars(args))
        code = 2 if result["pending"] else 0
    except (OSError, ValueError, KeyError, TypeError, AttributeError, zipfile.BadZipFile,
            RuntimeError, NotImplementedError, zlib.error) as error:
        # Do not echo input values/paths: reports can contain private user data.
        result = {"schema_version": 1, "status": "invalid", "error_type": type(error).__name__,
                  "reason": str(error) if isinstance(error, AuditError) else "invalid_input_structure_or_io",
                  "task_completed": False, "t00_b_authorized": False}
        code = 1
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return code


if __name__ == "__main__":
    sys.exit(main())
