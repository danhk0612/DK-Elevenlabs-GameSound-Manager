use crate::storage::Store;
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    store: Store,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "DK 한글 공백 {}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("workspace")).unwrap();
        // A tiny synthesized PCM WAV; no user sound fixture.
        let mut wav = b"RIFF".to_vec();
        wav.extend(40u32.to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(8000u32.to_le_bytes());
        wav.extend(16000u32.to_le_bytes());
        wav.extend(2u16.to_le_bytes());
        wav.extend(16u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend(4u32.to_le_bytes());
        wav.extend([0u8; 4]);
        fs::write(root.join("합성 원본.wav"), wav).unwrap();
        let store = Store::open(&root.join("settings")).unwrap();
        Self { root, store }
    }
    fn run(&mut self, op: &str, mut args: Value) -> Value {
        args["op"] = json!(op);
        args["protocol_version"] = json!(1);
        self.store.execute(&args)
    }
    fn project(&mut self, id: &str) {
        let result = self.run(
            "project.create",
            json!({"project_id":id,"workspace":self.root.join("workspace"),"name":"같은 이름"}),
        );
        assert_eq!(result["ok"], true, "{result}");
    }
    fn import(&mut self, project: &str) -> Value {
        self.run("asset.import",json!({"project_id":project,"asset_id":"hit","version_id":"original","kind":"sfx","name":"효과음","source_path":self.root.join("합성 원본.wav"),"source_note":"synthesized fixture"}))
    }
    fn db(&self, id: &str) -> rusqlite::Connection {
        rusqlite::Connection::open(
            self.root
                .join("workspace")
                .join(format!("dk-gamesound-{id}"))
                .join("project.sqlite3"),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn escaped_metadata_pages_fit_transport_without_skipping_items() {
    let mut f = Fixture::new();
    f.project("one");
    for n in 0..12 {
        let response=f.run("requirement.put",json!({"project_id":"one","requirement_id":format!("req-{n:02}"),"name":"test","kind":"sfx","description":"\u{0008}".repeat(2048)}));
        assert_eq!(response["ok"], true);
    }
    let mut after = String::new();
    let mut found = std::collections::BTreeSet::new();
    loop {
        let page = f.run(
            "requirement.list",
            json!({"project_id":"one","after":after}),
        );
        assert!(serde_json::to_vec(&page).unwrap().len() < crate::protocol::MAX_FRAME);
        for row in page["data"]["items"].as_array().unwrap() {
            assert!(found.insert(row["requirement_id"].as_str().unwrap().to_owned()));
        }
        match page["data"]["next_after"].as_str() {
            Some(next) => after = next.to_owned(),
            None => break,
        }
    }
    assert_eq!(found.len(), 12);
}

#[test]
fn projects_and_links_are_scoped_and_original_is_preserved() {
    let mut f = Fixture::new();
    f.project("one");
    f.project("two");
    let before = fs::read(f.root.join("합성 원본.wav")).unwrap();
    let one = f.import("one");
    assert_eq!(one["ok"], true, "{one}");
    assert_eq!(
        f.run("asset.list", json!({"project_id":"two"}))["data"]["items"],
        json!([])
    );
    assert_eq!(f.run("requirement.put",json!({"project_id":"two","requirement_id":"hit","name":"Hit","kind":"sfx","description":""}))["ok"],true);
    assert_eq!(f.run("requirement.link",json!({"project_id":"two","requirement_id":"hit","asset_id":"hit","version_id":"original"}))["ok"],false);
    let two = f.import("two");
    assert_eq!(two["ok"], true, "{two}");
    assert_ne!(one["data"]["path"], two["data"]["path"]);
    assert_eq!(fs::read(f.root.join("합성 원본.wav")).unwrap(), before);
}

#[test]
fn stable_ids_reuse_and_candidates_never_replace_originals() {
    let mut f = Fixture::new();
    f.project("one");
    let first = f.import("one");
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(f.import("one"), first);
    let original = PathBuf::from(first["data"]["path"].as_str().unwrap());
    let before = fs::read(&original).unwrap();
    fs::write(f.root.join("candidate.wav"), b"synthetic candidate bytes").unwrap();
    let candidate=f.run("asset.import",json!({"project_id":"one","asset_id":"hit","version_id":"take-2","parent_version_id":"original","name":"효과음","kind":"sfx","source_path":f.root.join("candidate.wav"),"source_note":"candidate fixture"}));
    assert_eq!(candidate["ok"], true, "{candidate}");
    assert_eq!(candidate["data"]["role"], "candidate");
    assert_eq!(fs::read(&original).unwrap(), before);
    fs::write(f.root.join("합성 원본.wav"), b"different input").unwrap();
    assert_eq!(f.import("one")["error"]["code"], "version_id_conflict");
    assert_eq!(fs::read(original).unwrap(), before);
}

#[test]
fn restart_recovers_publish_before_database_commit_without_source() {
    let mut f = Fixture::new();
    f.project("one");
    let imported = f.import("one");
    assert_eq!(imported["ok"], true);
    f.db("one")
        .execute("UPDATE versions SET state='copying'", [])
        .unwrap();
    fs::remove_file(f.root.join("합성 원본.wav")).unwrap();
    f.store = Store::open(&f.root.join("settings")).unwrap();
    let result = f.run("project.recover", json!({"project_id":"one"}));
    assert_eq!(result["data"]["items"][0]["result"], "ready", "{result}");
    assert_eq!(
        f.run(
            "asset.file",
            json!({"project_id":"one","asset_id":"hit","version_id":"original"})
        )["ok"],
        true
    );
}

#[test]
fn partial_copy_can_resume_but_tampered_ready_file_cannot_be_overwritten() {
    let mut f = Fixture::new();
    f.project("one");
    let imported = f.import("one");
    assert_eq!(imported["ok"], true);
    let target = PathBuf::from(imported["data"]["path"].as_str().unwrap());
    fs::remove_file(&target).unwrap();
    f.db("one")
        .execute("UPDATE versions SET state='copying'", [])
        .unwrap();
    let staging = f
        .root
        .join("workspace/dk-gamesound-one/imports/hit/original.part");
    fs::write(&staging, b"partial").unwrap();
    let result = f.run("project.recover", json!({"project_id":"one"}));
    assert_eq!(
        result["data"]["items"][0]["result"],
        "import_integrity_mismatch"
    );
    assert_eq!(f.import("one")["ok"], true);
    fs::write(&target, b"external change").unwrap();
    assert_eq!(f.import("one")["error"]["code"], "asset_integrity_mismatch");
    assert_eq!(fs::read(target).unwrap(), b"external change");
}

#[test]
fn metadata_pagination_foreign_keys_and_kind_checks() {
    let mut f = Fixture::new();
    f.project("one");
    for n in 0..12 {
        let result=f.run("requirement.put",json!({"project_id":"one","requirement_id":format!("req-{n:02}"),"name":"ボス ' ; DROP TABLE assets; --","kind":"voice","description":"Korean 한글"}));
        assert_eq!(result["ok"], true);
    }
    let page = f.run("requirement.list", json!({"project_id":"one"}));
    assert_eq!(page["data"]["items"].as_array().unwrap().len(), 10);
    let next = f.run(
        "requirement.list",
        json!({"project_id":"one","after":page["data"]["next_after"]}),
    );
    assert_eq!(next["data"]["items"].as_array().unwrap().len(), 2);
    assert_eq!(f.run("dialogue.put",json!({"project_id":"one","dialogue_id":"line","character_id":"missing","text":"안녕","language":"ko"}))["error"]["code"],"character_not_found");
    assert_eq!(f.run("character.put",json!({"project_id":"one","character_id":"hero","name":"같은 이름","voice_id":"external-voice-ref"}))["ok"],true);
    assert_eq!(f.run("dialogue.put",json!({"project_id":"one","dialogue_id":"line","character_id":"hero","text":"안녕","language":"ko","requirement_id":"req-00"}))["ok"],true);
    assert_eq!(
        f.run("dialogue.list", json!({"project_id":"one"}))["data"]["items"][0]["text"],
        "안녕"
    );
}

#[test]
fn path_traversal_case_aliases_and_future_schema_are_rejected() {
    let mut f = Fixture::new();
    for id in ["../escape", "con", "UPPER"] {
        let result = f.run(
            "project.create",
            json!({"project_id":id,"workspace":f.root.join("workspace"),"name":"x"}),
        );
        assert_eq!(result["error"]["code"], "invalid_id");
    }
    f.project("one");
    f.db("one").pragma_update(None, "user_version", 99).unwrap();
    assert_eq!(
        f.run("project.get", json!({"project_id":"one"}))["error"]["code"],
        "unsupported_schema"
    );
}

#[cfg(unix)]
#[test]
fn managed_symlinks_are_not_followed() {
    let mut f = Fixture::new();
    f.project("one");
    let imported = f.import("one");
    assert_eq!(imported["ok"], true);
    let target = PathBuf::from(imported["data"]["path"].as_str().unwrap());
    fs::remove_file(&target).unwrap();
    std::os::unix::fs::symlink(f.root.join("합성 원본.wav"), &target).unwrap();
    let result = f.run(
        "asset.file",
        json!({"project_id":"one","asset_id":"hit","version_id":"original"}),
    );
    assert_eq!(result["error"]["code"], "managed_path_is_link");
}

fn generation_request(id: &str) -> Value {
    json!({"project_id":"one","account_id":"demo","job_id":id,"asset_id":id,"version_id":"take-1","name":"mock sound","generation_kind":"sfx","text":"A short click","model_id":"mock-sfx","output_format":"mp3_44100_128","reserve_credits":20,"duration_ms":500})
}
fn generation_setup(f: &mut Fixture, calls: i64, credits: i64) {
    f.project("one");
    assert_eq!(
        f.run(
            "account.create",
            json!({"account_id":"demo","provider":"mock"})
        )["ok"],
        true
    );
    assert_eq!(
        f.run(
            "budget.set",
            json!({"account_id":"demo","request_limit":calls,"credit_limit":credits})
        )["ok"],
        true
    );
}
#[test]
fn generation_requires_policy_reuses_ids_and_registers_only_candidates() {
    let mut f = Fixture::new();
    f.project("one");
    f.run(
        "account.create",
        json!({"account_id":"demo","provider":"mock"}),
    );
    assert_eq!(
        f.run("job.submit", generation_request("first"))["error"]["code"],
        "budget_not_configured"
    );
    f.run(
        "budget.set",
        json!({"account_id":"demo","request_limit":2,"credit_limit":40}),
    );
    assert_eq!(
        f.run("job.submit", generation_request("first"))["data"]["state"],
        "queued"
    );
    assert_eq!(
        f.run("job.submit", generation_request("first"))["data"]["state"],
        "queued"
    );
    let mut conflict = generation_request("first");
    conflict["text"] = json!("changed");
    assert_eq!(
        f.run("job.submit", conflict)["error"]["code"],
        "job_id_conflict"
    );
    assert!(crate::generation::tick(&mut f.store).unwrap());
    let job = f.run("job.get", json!({"project_id":"one","job_id":"first"}));
    assert_eq!(job["data"]["state"], "succeeded", "{job}");
    let versions = f.run(
        "asset.versions",
        json!({"project_id":"one","asset_id":"first"}),
    );
    assert_eq!(versions["data"]["items"][0]["role"], "candidate");
    assert_eq!(versions["data"]["items"][0]["extension"], "wav");
    assert!(!crate::generation::tick(&mut f.store).unwrap());
    assert_eq!(
        f.run("budget.get", json!({"account_id":"demo"}))["data"]["requests_held"],
        1
    );
}
#[test]
fn workspace_budget_is_shared_across_projects_and_account_aliases() {
    let mut f = Fixture::new();
    generation_setup(&mut f, 1, 20);
    f.project("two");
    f.run(
        "account.create",
        json!({"account_id":"alias","provider":"mock"}),
    );
    assert_eq!(f.run("job.submit", generation_request("first"))["ok"], true);
    let mut other = generation_request("second");
    other["account_id"] = json!("alias");
    other["project_id"] = json!("two");
    assert_eq!(
        f.run("job.submit", other.clone())["error"]["code"],
        "budget_exceeded"
    );
    assert_eq!(
        f.run("job.get", json!({"project_id":"two","job_id":"first"}))["error"]["code"],
        "job_not_found"
    );
    assert_eq!(
        f.run("job.cancel", json!({"project_id":"one","job_id":"first"}))["data"]["state"],
        "cancelled"
    );
    assert_eq!(f.run("job.submit", other)["ok"], true);
}
#[test]
fn interrupted_requests_are_never_replayed_and_unknown_usage_remains_held() {
    let mut f = Fixture::new();
    generation_setup(&mut f, 3, 60);
    f.run("job.submit", generation_request("queued"));
    f.run("job.submit", generation_request("attempted"));
    f.store
        .catalog
        .execute(
            "UPDATE generation_jobs SET attempted=1,state='running' WHERE id='attempted'",
            [],
        )
        .unwrap();
    let root = f.root.join("settings");
    let old = std::mem::replace(&mut f.store, Store::open(&root).unwrap());
    drop(old);
    assert_eq!(crate::generation::pending(&f.store), 0);
    assert!(!crate::generation::tick(&mut f.store).unwrap());
    assert_eq!(
        f.run("job.get", json!({"project_id":"one","job_id":"queued"}))["data"]["state"],
        "interrupted"
    );
    assert_eq!(
        f.run("job.get", json!({"project_id":"one","job_id":"attempted"}))["data"]["state"],
        "outcome_unknown"
    );
    assert_eq!(
        f.run(
            "job.cancel",
            json!({"project_id":"one","job_id":"attempted"})
        )["error"]["code"],
        "job_not_cancellable"
    );
    assert_eq!(
        f.run("budget.get", json!({"account_id":"demo"}))["data"]["requests_held"],
        2
    );
}
#[test]
fn received_output_can_be_attached_again_without_generation_or_overwriting() {
    let mut f = Fixture::new();
    generation_setup(&mut f, 1, 20);
    f.run("job.submit", generation_request("first"));
    crate::generation::tick(&mut f.store).unwrap();
    let before = f.run(
        "asset.file",
        json!({"project_id":"one","asset_id":"first","version_id":"take-1"}),
    );
    f.store
        .catalog
        .execute(
            "UPDATE generation_jobs SET state='output_ready',result=NULL WHERE id='first'",
            [],
        )
        .unwrap();
    let recovered = f.run("job.recover", json!({"project_id":"one","job_id":"first"}));
    assert_eq!(recovered["data"]["state"], "succeeded", "{recovered}");
    assert_eq!(
        before,
        f.run(
            "asset.file",
            json!({"project_id":"one","asset_id":"first","version_id":"take-1"})
        )
    );
    assert_eq!(
        f.run("budget.get", json!({"account_id":"demo"}))["data"]["requests_held"],
        1
    );
}
#[test]
fn changed_cost_or_tighter_budget_blocks_dispatch_and_secrets_are_not_commands() {
    let mut f = Fixture::new();
    generation_setup(&mut f, 3, 60);
    f.run("job.submit", generation_request("first"));
    f.run("job.submit", generation_request("second"));
    f.store
        .catalog
        .execute(
            "UPDATE generation_jobs SET state='failed',attempted=1,confirmed=100 WHERE id='first'",
            [],
        )
        .unwrap();
    crate::generation::tick(&mut f.store).unwrap();
    let second = f.run("job.get", json!({"project_id":"one","job_id":"second"}));
    assert_eq!(second["data"]["state"], "failed_preflight");
    assert_eq!(second["data"]["request_attempted"], false);
    let mut c = generation_request("third");
    c["key"] = json!("never-send-this");
    assert_eq!(f.run("job.submit", c)["ok"], false);
    let mut c = generation_request("third");
    c["duration_ms"] = json!("500");
    assert_eq!(f.run("job.submit", c)["ok"], false);
    f.run(
        "account.create",
        json!({"account_id":"live","provider":"elevenlabs"}),
    );
    assert_eq!(
        f.run(
            "budget.set",
            json!({"account_id":"live","request_limit":1,"credit_limit":1})
        )["error"]["code"],
        "live_policy_requires_gui"
    );
}

#[test]
fn generation_network_failure_or_rate_limit_is_attempted_once_and_not_replayed() {
    for (fault, expected) in [(1, "outcome_unknown"), (2, "failed")] {
        crate::provider::mock_fault(fault);
        let mut f = Fixture::new();
        generation_setup(&mut f, 2, 40);
        f.run("job.submit", generation_request("first"));
        crate::generation::tick(&mut f.store).unwrap();
        assert_eq!(
            f.run("job.get", json!({"project_id":"one","job_id":"first"}))["data"]["state"],
            expected
        );
        assert!(!crate::generation::tick(&mut f.store).unwrap());
        assert_eq!(crate::provider::mock_posts(), 1);
        assert_eq!(
            f.run("job.submit", generation_request("first"))["data"]["state"],
            expected
        );
        assert_eq!(crate::provider::mock_posts(), 1);
        assert_eq!(
            f.run("budget.get", json!({"account_id":"demo"}))["data"]["credits_held"],
            20
        );
    }
    crate::provider::mock_fault(0);
}
#[test]
fn generation_preserves_existing_original_and_checks_dialogue_snapshot() {
    let mut f = Fixture::new();
    generation_setup(&mut f, 2, 40);
    assert_eq!(f.import("one")["ok"], true);
    let original = f.run(
        "asset.file",
        json!({"project_id":"one","asset_id":"hit","version_id":"original"}),
    );
    let mut c = generation_request("new-take");
    c["asset_id"] = json!("hit");
    c["name"] = json!("효과음");
    assert_eq!(
        f.run("job.submit", c.clone())["error"]["code"],
        "candidate_parent_required"
    );
    c["parent_version_id"] = json!("original");
    assert_eq!(f.run("job.submit", c)["ok"], true);
    crate::generation::tick(&mut f.store).unwrap();
    assert_eq!(
        original,
        f.run(
            "asset.file",
            json!({"project_id":"one","asset_id":"hit","version_id":"original"})
        )
    );
    let mut c = generation_request("speech");
    c["generation_kind"] = json!("tts");
    c.as_object_mut().unwrap().remove("duration_ms");
    c["voice_id"] = json!("mock-voice");
    c["model_id"] = json!("mock-model");
    c["dialogue_id"] = json!("missing");
    assert_eq!(
        f.run("job.submit", c.clone())["error"]["code"],
        "dialogue_snapshot_mismatch"
    );
    c.as_object_mut().unwrap().remove("dialogue_id");
    assert_eq!(f.run("job.submit", c)["ok"], true);
    crate::generation::tick(&mut f.store).unwrap();
    assert_eq!(
        f.run("job.get", json!({"project_id":"one","job_id":"speech"}))["data"]["state"],
        "succeeded"
    );
}

impl Fixture {
    fn audio_tools(&mut self) {
        let ffmpeg = std::env::var("DK_TEST_FFMPEG")
            .expect("set DK_TEST_FFMPEG for ignored audio integration tests");
        let ffprobe = std::env::var("DK_TEST_FFPROBE")
            .expect("set DK_TEST_FFPROBE for ignored audio integration tests");
        let result = self.run(
            "audio.tools.set",
            json!({"ffmpeg_path":ffmpeg,"ffprobe_path":ffprobe}),
        );
        assert_eq!(result["ok"], true, "{result}");
    }
    fn signal(&mut self) {
        let mut wav = b"RIFF".to_vec();
        wav.extend(16036u32.to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(8000u32.to_le_bytes());
        wav.extend(16000u32.to_le_bytes());
        wav.extend(2u16.to_le_bytes());
        wav.extend(16u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend(16000u32.to_le_bytes());
        for frame in 0..8000 {
            wav.extend(
                if (2000..6000).contains(&frame) {
                    16384i16
                } else {
                    0i16
                }
                .to_le_bytes(),
            );
        }
        fs::write(self.root.join("합성 원본.wav"), wav).unwrap();
        self.project("one");
        assert_eq!(self.import("one")["ok"], true);
    }
    fn measure(&mut self, id: &str, version: &str) -> Value {
        self.run("audio.analyze",json!({"project_id":"one","asset_id":"hit","version_id":version,"operation_id":id,"waveform_bins":32,"silence_db":-60,"silence_min_ms":200,"clip_level":0.49}))
    }
}
fn edit_request(id: &str, version: &str) -> Value {
    json!({"project_id":"one","asset_id":"hit","version_id":version,"parent_version_id":"original","operation_id":id,"start_ms":250,"end_ms":750,"gain_db":-6.020599913,"fade_in_ms":0,"fade_out_ms":0,"output_format":"wav_pcm_s16le","sample_rate":8000,"channels":1})
}
#[test]
fn audio_parameters_are_explicit_and_invalid_ranges_never_reach_tools() {
    let mut f = Fixture::new();
    f.signal();
    assert_eq!(
        f.measure("check", "original")["error"]["code"],
        "audio_tools_not_configured"
    );
    for (field, value) in [
        ("end_ms", json!(250)),
        ("gain_db", json!("6dB,anull")),
        ("fade_in_ms", json!(600)),
        ("sample_rate", json!(12345)),
        ("channels", json!(0)),
        ("version_id", json!("original")),
    ] {
        let mut c = edit_request("edit", "candidate");
        c[field] = value;
        assert_eq!(f.run("audio.edit", c)["ok"], false);
    }
    let mut c = edit_request("edit", "candidate");
    c["loop_start_ms"] = json!(10);
    assert_eq!(
        f.run("audio.edit", c)["error"]["code"],
        "invalid_audio_loop"
    );
    let mut c = edit_request("edit", "candidate");
    c.as_object_mut().unwrap().remove("output_format");
    assert_eq!(f.run("audio.edit", c)["ok"], false);
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_real_signal_metrics_waveform_and_version_bound_reports() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let report = f.measure("check", "original");
    assert_eq!(report["ok"], true, "{report}");
    let metrics = &report["data"]["result"]["metrics"];
    assert_eq!(metrics["frames"], 8000);
    assert_eq!(metrics["duration_ms"], 1000.0);
    assert_eq!(metrics["sample_peak"], 0.5);
    assert!((metrics["rms"].as_f64().unwrap() - 0.353553390593).abs() < 0.000001);
    assert_eq!(metrics["threshold_exceeding_samples"], 4000);
    assert_eq!(metrics["quiet_ms"], 500.0);
    assert_eq!(metrics["longest_quiet_ms"], 250.0);
    assert_eq!(metrics["silence_region_count"], 2);
    assert_eq!(metrics["waveform"].as_array().unwrap().len(), 32);
    assert_eq!(f.measure("check", "original"), report);
    assert!(serde_json::to_vec(&report).unwrap().len() < crate::protocol::MAX_FRAME);
    f.project("two");
    assert_eq!(
        f.run(
            "audio.get",
            json!({"project_id":"two","operation_id":"check"})
        )["error"]["code"],
        "audio_operation_not_found"
    );
    let page = f.run("audio.list", json!({"project_id":"one"}));
    assert_eq!(page["data"]["items"].as_array().unwrap().len(), 1);
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_real_edit_trim_gain_fade_formats_and_original_preservation() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let original = f.run(
        "asset.file",
        json!({"project_id":"one","asset_id":"hit","version_id":"original"}),
    );
    let request = edit_request("edit", "take-1");
    let edited = f.run("audio.edit", request.clone());
    assert_eq!(edited["ok"], true, "{edited}");
    assert_eq!(edited["data"]["state"], "succeeded");
    assert_eq!(edited["data"]["result"]["candidate"]["role"], "candidate");
    assert_eq!(f.run("audio.edit", request), edited);
    let measured = f.measure("check", "take-1");
    assert_eq!(measured["ok"], true, "{measured}");
    let m = &measured["data"]["result"]["metrics"];
    assert_eq!(m["duration_ms"], 500.0);
    assert!((m["sample_peak"].as_f64().unwrap() - 0.25).abs() < 0.00005);
    let mut fade = edit_request("fade", "take-2");
    fade["fade_in_ms"] = json!(100);
    fade["fade_out_ms"] = json!(100);
    fade["sample_rate"] = json!(16000);
    fade["channels"] = json!(2);
    assert_eq!(f.run("audio.edit", fade)["ok"], true);
    let measured = f.measure("fade-check", "take-2");
    assert_eq!(measured["ok"], true, "{measured}");
    let report = &measured["data"]["result"];
    assert_eq!(report["metadata"]["channels"], 2);
    assert_eq!(report["metadata"]["sample_rate"], 16000);
    assert!(report["metrics"]["waveform"][0][1].as_f64().unwrap() < 0.06);
    for (id, format) in [("ogg", "ogg_vorbis_q4"), ("mp3", "mp3_128k")] {
        let mut request = edit_request(id, id);
        request["output_format"] = json!(format);
        request["sample_rate"] = json!(44100);
        let result = f.run("audio.edit", request);
        assert_eq!(result["ok"], true, "{result}");
        assert_eq!(f.measure(&format!("check-{id}"), id)["ok"], true);
    }
    assert_eq!(
        f.run(
            "asset.file",
            json!({"project_id":"one","asset_id":"hit","version_id":"original"})
        ),
        original
    );
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_real_recovery_reuses_output_without_tools_and_detects_tampering() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let edited = f.run("audio.edit", edit_request("edit", "take-1"));
    assert_eq!(edited["ok"], true, "{edited}");
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='output_ready' WHERE id='edit'",
            [],
        )
        .unwrap();
    f.store
        .catalog
        .execute("DELETE FROM audio_tools", [])
        .unwrap();
    let recovered = f.run(
        "audio.recover",
        json!({"project_id":"one","operation_id":"edit"}),
    );
    assert_eq!(recovered, edited);
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='output_ready' WHERE id='edit'",
            [],
        )
        .unwrap();
    fs::write(
        f.root.join("settings/audio-operations/edit/output.wav"),
        b"tampered",
    )
    .unwrap();
    assert_eq!(
        f.run(
            "audio.recover",
            json!({"project_id":"one","operation_id":"edit"})
        )["error"]["code"],
        "audio_output_integrity_mismatch"
    );
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_real_bad_input_conflicts_changed_tools_and_restart_do_not_overwrite() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let mut c = edit_request("edit", "take-1");
    c["end_ms"] = json!(2000);
    assert_eq!(
        f.run("audio.edit", c.clone())["error"]["code"],
        "audio_trim_out_of_range"
    );
    assert_eq!(f.run("audio.edit", c)["data"]["state"], "failed");
    assert_eq!(
        f.run("audio.edit", edit_request("other", "take-1"))["error"]["code"],
        "audio_target_reserved"
    );
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='processing' WHERE id='edit'",
            [],
        )
        .unwrap();
    let root = f.root.join("settings");
    let old = std::mem::replace(&mut f.store, Store::open(&root).unwrap());
    drop(old);
    assert_eq!(
        f.run(
            "audio.get",
            json!({"project_id":"one","operation_id":"edit"})
        )["data"]["state"],
        "interrupted"
    );
    let mut v: Value = serde_json::from_str(
        &f.store
            .catalog
            .query_row("SELECT settings FROM audio_tools", [], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
    )
    .unwrap();
    v["ffmpeg"]["sha256"] = json!("different");
    f.store
        .catalog
        .execute("UPDATE audio_tools SET settings=?1", [v.to_string()])
        .unwrap();
    assert_eq!(
        f.run("audio.tools.get", json!({}))["error"]["code"],
        "audio_tool_changed"
    );
    f.audio_tools();
    fs::write(f.root.join("bad.wav"), b"not an audio file").unwrap();
    assert_eq!(f.run("asset.import",json!({"project_id":"one","asset_id":"bad","version_id":"original","name":"bad","kind":"sfx","source_path":f.root.join("bad.wav"),"source_note":"synthetic invalid fixture"}))["ok"],true);
    let mut c = json!({"project_id":"one","asset_id":"bad","version_id":"original","operation_id":"bad-check","waveform_bins":16,"silence_db":-60,"silence_min_ms":1,"clip_level":1});
    assert_eq!(
        f.run("audio.analyze", c.clone())["error"]["code"],
        "audio_tool_failed"
    );
    c["silence_db"] = json!(-50);
    assert_eq!(
        f.run("audio.analyze", c)["error"]["code"],
        "audio_operation_id_conflict"
    );
}

fn queued_request(mut c: Value) -> Value {
    c["action"] = json!(if c.get("parent_version_id").is_some() {
        "edit"
    } else {
        "analyze"
    });
    c
}
fn queue_fixture(f: &mut Fixture) {
    f.signal();
    // No executable is run in queue policy tests. Execution must reject these paths.
    f.store.catalog.execute("INSERT INTO audio_tools VALUES(1,?1)",[json!({"ffmpeg":{"path":"/not-installed","sha256":"fixture","version":"fixture"},"ffprobe":{"path":"/not-installed","sha256":"fixture","version":"fixture"}}).to_string()]).unwrap();
}
fn audio_fast(root: &std::path::Path, op: &str, mut c: Value) -> Value {
    c["op"] = json!(op);
    c["protocol_version"] = json!(1);
    match crate::audio_queue::fast(root, &c) {
        Ok((data, _)) => crate::protocol::success(data),
        Err(e) => crate::protocol::error(e.0),
    }
}
#[test]
fn audio_queue_submission_idempotency_cancel_and_project_isolation() {
    let mut f = Fixture::new();
    queue_fixture(&mut f);
    f.project("two");
    let root = f.root.join("settings");
    let c = queued_request(edit_request("edit", "take-1"));
    let result = audio_fast(&root, "audio.submit", c.clone());
    assert_eq!(result["data"]["state"], "queued", "{result}");
    assert_eq!(crate::audio_queue::pending_at(&root).unwrap(), 1);
    f.store
        .catalog
        .execute("DELETE FROM audio_tools", [])
        .unwrap();
    assert_eq!(audio_fast(&root, "audio.submit", c), result);
    assert_eq!(
        audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"two","operation_id":"edit"})
        )["error"]["code"],
        "audio_operation_not_found"
    );
    assert_eq!(
        audio_fast(
            &root,
            "audio.cancel",
            json!({"project_id":"two","operation_id":"edit"})
        )["error"]["code"],
        "audio_operation_not_found"
    );
    let cancelled = audio_fast(
        &root,
        "audio.cancel",
        json!({"project_id":"one","operation_id":"edit"}),
    );
    assert_eq!(cancelled["data"]["state"], "cancelled");
    assert_eq!(
        audio_fast(
            &root,
            "audio.cancel",
            json!({"project_id":"one","operation_id":"edit"})
        ),
        cancelled
    );
    assert!(!crate::audio_queue::tick(&mut f.store).unwrap());
    assert_eq!(
        f.run(
            "asset.versions",
            json!({"project_id":"one","asset_id":"hit"})
        )["data"]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn audio_queue_bound_and_restart_do_not_replay_work() {
    let mut f = Fixture::new();
    queue_fixture(&mut f);
    let base = json!({"project_id":"one","asset_id":"hit","version_id":"original","action":"analyze","waveform_bins":16,"silence_db":-60,"silence_min_ms":1,"clip_level":1});
    for i in 0..32 {
        let mut c = base.clone();
        c["operation_id"] = json!(format!("check-{i}"));
        assert_eq!(f.run("audio.submit", c)["ok"], true);
    }
    let mut c = base.clone();
    c["operation_id"] = json!("overflow");
    assert_eq!(
        f.run("audio.submit", c)["error"]["code"],
        "audio_queue_full"
    );
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='processing',cancel_requested=1 WHERE id='check-0'",
            [],
        )
        .unwrap();
    let root = f.root.join("settings");
    let old = std::mem::replace(&mut f.store, Store::open(&root).unwrap());
    drop(old);
    assert_eq!(crate::audio_queue::pending(&f.store), 0);
    assert!(!crate::audio_queue::tick(&mut f.store).unwrap());
    assert_eq!(
        f.run(
            "audio.get",
            json!({"project_id":"one","operation_id":"check-0"})
        )["data"]["state"],
        "cancelled"
    );
    assert_eq!(
        f.run(
            "audio.get",
            json!({"project_id":"one","operation_id":"check-1"})
        )["data"]["state"],
        "interrupted"
    );
}
#[test]
fn concurrent_audio_submissions_reserve_a_target_once() {
    let mut f = Fixture::new();
    queue_fixture(&mut f);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut threads = Vec::new();
    for id in ["first", "second"] {
        let root = f.root.join("settings");
        let barrier = barrier.clone();
        let c = queued_request(edit_request(id, "take-1"));
        threads.push(std::thread::spawn(move || {
            barrier.wait();
            audio_fast(&root, "audio.submit", c)
        }));
    }
    let results = threads
        .into_iter()
        .map(|t| t.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        results.iter().filter(|r| r["ok"] == true).count(),
        1,
        "{results:?}"
    );
    assert!(results
        .iter()
        .any(|r| r["error"]["code"] == "audio_target_reserved"));
}
#[test]
fn audio_cancellation_commit_boundary_and_control_stop_publication() {
    let mut f = Fixture::new();
    queue_fixture(&mut f);
    f.run(
        "audio.submit",
        queued_request(edit_request("edit", "take-1")),
    );
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='processing',phase='encoding' WHERE id='edit'",
            [],
        )
        .unwrap();
    let root = f.root.join("settings");
    let cancel = audio_fast(
        &root,
        "audio.cancel",
        json!({"project_id":"one","operation_id":"edit"}),
    );
    assert_eq!(cancel["data"]["state"], "processing");
    assert_eq!(cancel["data"]["cancel_requested"], true);
    let control = crate::audio_queue::Control::new(&root, "edit").unwrap();
    assert_eq!(control.check().unwrap_err().0, "audio_cancelled");
    assert_eq!(
        crate::audio_queue::publish_output(&f.store, "edit", "wav", "fixture", &json!({}))
            .unwrap_err()
            .0,
        "audio_cancelled"
    );
    crate::audio_queue::fail(&f.store, "edit", &crate::storage::Error("audio_cancelled")).unwrap();
    assert_eq!(
        audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"one","operation_id":"edit"})
        )["data"]["state"],
        "cancelled"
    );
    f.store
        .catalog
        .execute(
            "UPDATE audio_operations SET state='processing',cancel_requested=0 WHERE id='edit'",
            [],
        )
        .unwrap();
    crate::audio_queue::publish_output(&f.store, "edit", "wav", "fixture", &json!({})).unwrap();
    assert_eq!(
        audio_fast(
            &root,
            "audio.cancel",
            json!({"project_id":"one","operation_id":"edit"})
        )["error"]["code"],
        "audio_not_cancellable"
    );
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_async_execution_pins_tools_and_creates_candidate_then_analysis() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let root = f.root.join("settings");
    let request = queued_request(edit_request("edit", "take-1"));
    assert_eq!(
        audio_fast(&root, "audio.submit", request.clone())["data"]["state"],
        "queued"
    );
    f.store
        .catalog
        .execute("DELETE FROM audio_tools", [])
        .unwrap();
    assert!(crate::audio_queue::tick(&mut f.store).unwrap());
    let result = audio_fast(
        &root,
        "audio.get",
        json!({"project_id":"one","operation_id":"edit"}),
    );
    assert_eq!(result["data"]["state"], "succeeded", "{result}");
    assert_eq!(result["data"]["result"]["candidate"]["role"], "candidate");
    assert_eq!(audio_fast(&root, "audio.submit", request), result);
    f.audio_tools();
    let inspect = json!({"project_id":"one","asset_id":"hit","version_id":"take-1","operation_id":"check","action":"analyze","waveform_bins":32,"silence_db":-60,"silence_min_ms":1,"clip_level":0.99});
    assert_eq!(
        audio_fast(&root, "audio.submit", inspect)["data"]["state"],
        "queued"
    );
    crate::audio_queue::tick(&mut f.store).unwrap();
    assert_eq!(
        audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"one","operation_id":"check"})
        )["data"]["result"]["metrics"]["duration_ms"],
        500.0
    );
}
#[test]
#[ignore = "requires explicit DK_TEST_FFMPEG and DK_TEST_FFPROBE"]
fn audio_async_active_work_can_be_observed_and_cancelled_from_another_connection() {
    let mut f = Fixture::new();
    f.signal();
    f.audio_tools();
    let root = f.root.join("settings");
    audio_fast(
        &root,
        "audio.submit",
        queued_request(edit_request("edit", "take-1")),
    );
    let worker_root = root.clone();
    let worker = std::thread::spawn(move || {
        let mut store = Store {
            root: worker_root.clone(),
            catalog: crate::storage::catalog_connection(&worker_root, true).unwrap(),
        };
        crate::audio_queue::tick(&mut store).unwrap()
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let current = audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"one","operation_id":"edit"}),
        );
        if current["data"]["state"] == "processing" {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{current}");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let second = audio_fast(
        &root,
        "audio.submit",
        queued_request(edit_request("second", "take-2")),
    );
    assert_eq!(second["data"]["state"], "queued", "{second}");
    let cancel = audio_fast(
        &root,
        "audio.cancel",
        json!({"project_id":"one","operation_id":"edit"}),
    );
    assert_eq!(cancel["data"]["cancel_requested"], true);
    assert!(worker.join().unwrap());
    assert_eq!(
        audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"one","operation_id":"second"})
        )["data"]["state"],
        "queued"
    );
    assert_eq!(
        audio_fast(
            &root,
            "audio.get",
            json!({"project_id":"one","operation_id":"edit"})
        )["data"]["state"],
        "cancelled"
    );
    assert_eq!(
        f.run(
            "asset.versions",
            json!({"project_id":"one","asset_id":"hit"})
        )["data"]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn audio_and_generation_share_atomic_target_reservations() {
    let mut f = Fixture::new();
    queue_fixture(&mut f);
    f.run(
        "account.create",
        json!({"account_id":"demo","provider":"mock"}),
    );
    f.run(
        "budget.set",
        json!({"account_id":"demo","request_limit":2,"credit_limit":100}),
    );
    let root = f.root.join("settings");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let audio_root = root.clone();
    let audio_barrier = barrier.clone();
    let local = std::thread::spawn(move || {
        audio_barrier.wait();
        audio_fast(
            &audio_root,
            "audio.submit",
            queued_request(edit_request("local", "take-1")),
        )
    });
    let remote = std::thread::spawn(move || {
        let mut store = Store {
            root: root.clone(),
            catalog: crate::storage::catalog_connection(&root, true).unwrap(),
        };
        let mut c = generation_request("remote");
        c["asset_id"] = json!("hit");
        c["version_id"] = json!("take-1");
        c["name"] = json!("효과음");
        c["parent_version_id"] = json!("original");
        c["op"] = json!("job.submit");
        c["protocol_version"] = json!(1);
        barrier.wait();
        store.execute(&c)
    });
    let results = [local.join().unwrap(), remote.join().unwrap()];
    assert_eq!(
        results.iter().filter(|r| r["ok"] == true).count(),
        1,
        "{results:?}"
    );
    let count: i64 = f
        .store
        .catalog
        .query_row(
            "SELECT (SELECT count(*) FROM audio_operations)+(SELECT count(*) FROM generation_jobs)",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}
