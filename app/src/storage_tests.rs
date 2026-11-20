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
