//! Shared, bounded command descriptions for CLI JSON and MCP tools.
use serde_json::{json, Value};

pub struct Command {
    pub op: &'static str,
    pub required: &'static [&'static str],
    pub optional: &'static [&'static str],
    pub read_only: bool,
}

pub const COMMANDS: &[Command] = &[
    Command {
        op: "version",
        required: &[],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "status",
        required: &[],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "shutdown",
        required: &[],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "project.create",
        required: &["project_id", "workspace", "name"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "project.list",
        required: &[],
        optional: &["after", "limit", "search"],
        read_only: true,
    },
    Command {
        op: "project.get",
        required: &["project_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "project.recover",
        required: &["project_id"],
        optional: &["after", "limit"],
        read_only: false,
    },
    Command {
        op: "asset.import",
        required: &[
            "project_id",
            "asset_id",
            "version_id",
            "name",
            "kind",
            "source_path",
            "source_note",
        ],
        optional: &["parent_version_id"],
        read_only: false,
    },
    Command {
        op: "asset.list",
        required: &["project_id"],
        optional: &["after", "limit", "search", "kind"],
        read_only: true,
    },
    Command {
        op: "asset.versions",
        required: &["project_id", "asset_id"],
        optional: &["after", "limit"],
        read_only: true,
    },
    Command {
        op: "asset.file",
        required: &["project_id", "asset_id", "version_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "requirement.put",
        required: &[
            "project_id",
            "requirement_id",
            "name",
            "kind",
            "description",
        ],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "requirement.list",
        required: &["project_id"],
        optional: &["after", "limit", "search", "kind"],
        read_only: true,
    },
    Command {
        op: "requirement.link",
        required: &["project_id", "requirement_id", "asset_id", "version_id"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "character.put",
        required: &["project_id", "character_id", "name"],
        optional: &["voice_id", "notes"],
        read_only: false,
    },
    Command {
        op: "character.list",
        required: &["project_id"],
        optional: &["after", "limit", "search"],
        read_only: true,
    },
    Command {
        op: "dialogue.put",
        required: &[
            "project_id",
            "dialogue_id",
            "character_id",
            "text",
            "language",
        ],
        optional: &["requirement_id"],
        read_only: false,
    },
    Command {
        op: "dialogue.list",
        required: &["project_id"],
        optional: &["after", "limit", "search", "character_id"],
        read_only: true,
    },
];

pub fn find(op: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.op == op)
}
pub fn field_schema(key: &str) -> Value {
    match key {
        "limit" => json!({"type":"integer","minimum":1,"maximum":10}),
        "kind" => json!({"type":"string","enum":["bgm","sfx","voice"]}),
        "after" => {
            json!({"type":"string","maxLength":129,"description":"Opaque next_after cursor from the preceding page; omit for first page."})
        }
        "workspace" => {
            json!({"type":"string","maxLength":4096,"description":"Absolute existing local workspace directory; a managed project subfolder is created here."})
        }
        "source_path" => {
            json!({"type":"string","maxLength":4096,"description":"Absolute existing audio file. Copied, never moved or modified; maximum 512 MiB."})
        }
        key if key.ends_with("_id") && key != "voice_id" => {
            json!({"type":"string","maxLength":64,"pattern":"^[a-z0-9_-]+$","description":"Stable lowercase identifier. Windows reserved device names are rejected. project_id always scopes this request."})
        }
        _ => json!({"type":"string","maxLength":4096}),
    }
}
pub fn tools() -> Vec<Value> {
    COMMANDS.iter().filter(|c| c.op != "shutdown").map(|c| {
        let mut properties = serde_json::Map::new();
        for key in c.required.iter().chain(c.optional) { properties.insert((*key).to_owned(),field_schema(key)); }
        json!({"name":format!("sound_{}",c.op.replace('.',"_")),
            "description":format!("Local {}. Explicit project_id scopes project data. Never generates remotely or applies to game files.",c.op),
            "inputSchema":{"type":"object","properties":properties,"required":c.required,"additionalProperties":false},
            "annotations":{"readOnlyHint":c.read_only,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})
    }).collect()
}

pub fn validate(command: &Value) -> Result<(), &'static str> {
    let object = command.as_object().ok_or("invalid_request")?;
    if object.get("protocol_version").and_then(Value::as_u64) != Some(1) {
        return Err("invalid_request");
    }
    let op = object
        .get("op")
        .and_then(Value::as_str)
        .ok_or("invalid_request")?;
    let definition = find(op).ok_or("unsupported_operation")?;
    for key in object.keys() {
        if key != "op"
            && key != "protocol_version"
            && !definition.required.contains(&key.as_str())
            && !definition.optional.contains(&key.as_str())
        {
            return Err("unknown_field");
        }
    }
    for key in definition.required {
        if !object.contains_key(*key) {
            return Err("missing_field");
        }
    }
    for key in definition.required.iter().chain(definition.optional) {
        let Some(value) = object.get(*key) else {
            continue;
        };
        if *key == "limit" {
            if !value.as_u64().is_some_and(|n| (1..=10).contains(&n)) {
                return Err("invalid_limit");
            }
            continue;
        }
        let text = value.as_str().ok_or("invalid_field_type")?;
        let max = match *key {
            "description" | "text" => 2048,
            "source_note" | "notes" => 1024,
            "workspace" | "source_path" => 4096,
            _ => 200,
        };
        if text.len() > max || text.contains('\0') {
            return Err("invalid_field_length");
        }
        if (*key == "project_id" || key.ends_with("_id") && *key != "voice_id" || *key == "after")
            && !text.is_empty()
        {
            let parts: Vec<_> = if *key == "after" && op == "project.recover" {
                text.split(':').collect()
            } else {
                vec![text]
            };
            if parts.len() > 2 || parts.iter().any(|s| !valid_id(s)) {
                return Err("invalid_id");
            }
        }
        if definition.required.contains(key)
            && text.trim().is_empty()
            && !["source_note", "description"].contains(key)
        {
            return Err("empty_field");
        }
        if *key == "kind" && !["bgm", "sfx", "voice"].contains(&text) {
            return Err("invalid_kind");
        }
    }
    Ok(())
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        && ![
            "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
            "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
        ]
        .contains(&id)
}
