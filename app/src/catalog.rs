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
    Command {
        op: "account.create",
        required: &["account_id", "provider"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "account.get",
        required: &["account_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "account.refresh",
        required: &["account_id"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "budget.set",
        required: &["account_id", "request_limit", "credit_limit"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "budget.get",
        required: &["account_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "voice.list",
        required: &["account_id"],
        optional: &["page_token"],
        read_only: true,
    },
    Command {
        op: "voice.get",
        required: &["account_id", "voice_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "voice.settings",
        required: &["account_id", "voice_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "model.list",
        required: &["account_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "job.submit",
        required: &[
            "project_id",
            "account_id",
            "job_id",
            "asset_id",
            "version_id",
            "name",
            "generation_kind",
            "text",
            "model_id",
            "output_format",
            "reserve_credits",
        ],
        optional: &[
            "voice_id",
            "language",
            "voice_settings",
            "duration_ms",
            "loop",
            "prompt_influence",
            "parent_version_id",
            "dialogue_id",
        ],
        read_only: false,
    },
    Command {
        op: "job.get",
        required: &["project_id", "job_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "job.list",
        required: &["project_id"],
        optional: &["after", "limit"],
        read_only: true,
    },
    Command {
        op: "job.cancel",
        required: &["project_id", "job_id"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "job.recover",
        required: &["project_id", "job_id"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "audio.tools.set",
        required: &["ffmpeg_path", "ffprobe_path"],
        optional: &[],
        read_only: false,
    },
    Command {
        op: "audio.tools.get",
        required: &[],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "audio.analyze",
        required: &[
            "project_id",
            "asset_id",
            "version_id",
            "operation_id",
            "waveform_bins",
            "silence_db",
            "silence_min_ms",
            "clip_level",
        ],
        optional: &["loop_start_ms", "loop_end_ms"],
        read_only: false,
    },
    Command {
        op: "audio.edit",
        required: &[
            "project_id",
            "asset_id",
            "version_id",
            "parent_version_id",
            "operation_id",
            "start_ms",
            "end_ms",
            "gain_db",
            "fade_in_ms",
            "fade_out_ms",
            "output_format",
            "sample_rate",
            "channels",
        ],
        optional: &["loop_start_ms", "loop_end_ms"],
        read_only: false,
    },
    Command {
        op: "audio.list",
        required: &["project_id"],
        optional: &["after", "limit"],
        read_only: true,
    },
    Command {
        op: "audio.get",
        required: &["project_id", "operation_id"],
        optional: &[],
        read_only: true,
    },
    Command {
        op: "audio.recover",
        required: &["project_id", "operation_id"],
        optional: &[],
        read_only: false,
    },
];

pub fn find(op: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.op == op)
}
pub fn field_schema(key: &str) -> Value {
    match key {
        "silence_min_ms" => json!({"type":"integer","minimum":1,"maximum":86400000}),
        "sample_rate" => {
            json!({"type":"integer","enum":[8000,16000,22050,24000,32000,44100,48000,96000]})
        }
        "channels" => json!({"type":"integer","minimum":1,"maximum":2}),
        "waveform_bins" => json!({"type":"integer","minimum":16,"maximum":256}),
        "gain_db" => json!({"type":"number","minimum":-60,"maximum":24}),
        "silence_db" => json!({"type":"number","minimum":-120,"maximum":0}),
        "clip_level" => json!({"type":"number","minimum":0.01,"maximum":1}),
        "start_ms" | "end_ms" | "fade_in_ms" | "fade_out_ms" | "loop_start_ms" | "loop_end_ms" => {
            json!({"type":"integer","minimum":0,"maximum":86400000})
        }
        "ffmpeg_path" | "ffprobe_path" => json!({"type":"string","maxLength":4096}),
        "request_limit" => json!({"type":"integer","minimum":1,"maximum":1000000}),
        "credit_limit" | "reserve_credits" => {
            json!({"type":"integer","minimum":1,"maximum":1000000000})
        }
        "duration_ms" => json!({"type":"integer","minimum":500,"maximum":30000}),
        "loop" => json!({"type":"boolean"}),
        "prompt_influence" => json!({"type":"number","minimum":0,"maximum":1}),
        "voice_settings" => {
            json!({"type":"object","properties":{"stability":{"type":"number","minimum":0,"maximum":1},"similarity_boost":{"type":"number","minimum":0,"maximum":1},"style":{"type":"number","minimum":0,"maximum":1},"speed":{"type":"number","minimum":0.7,"maximum":1.2},"use_speaker_boost":{"type":"boolean"}},"additionalProperties":false})
        }
        "provider" => json!({"type":"string","enum":["mock","elevenlabs"]}),
        "generation_kind" => json!({"type":"string","enum":["tts","sfx"]}),
        "output_format" => {
            json!({"type":"string","enum":["mp3_44100_128","wav_pcm_s16le","ogg_vorbis_q4","mp3_128k"]})
        }
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
        key if key.ends_with("_id") && !matches!(key, "voice_id" | "model_id") => {
            json!({"type":"string","maxLength":64,"pattern":"^[a-z0-9_-]+$","description":"Stable lowercase identifier. Windows reserved device names are rejected. project_id always scopes this request."})
        }
        _ => json!({"type":"string","maxLength":4096}),
    }
}
pub fn tools() -> Vec<Value> {
    COMMANDS.iter().filter(|c| c.op != "shutdown").map(|c| {
        let mut properties = serde_json::Map::new();
        for key in c.required.iter().chain(c.optional) { properties.insert((*key).to_owned(),field_schema(key)); }
        if c.op=="job.submit" { properties.insert("output_format".into(),json!({"type":"string","enum":["mp3_44100_128"]})); }
        if c.op=="audio.edit" { properties.insert("output_format".into(),json!({"type":"string","enum":["wav_pcm_s16le","ogg_vorbis_q4","mp3_128k"]})); }

        json!({"name":format!("sound_{}",c.op.replace('.',"_")),
            "description":format!("{}: Explicit project_id scopes project data. Mock operations are local. ElevenLabs operations require configured credentials and policy; generation creates candidates only. Never applies to game files.",c.op),
            "inputSchema":{"type":"object","properties":properties,"required":c.required,"additionalProperties":false},
            "annotations":{"readOnlyHint":c.read_only,"destructiveHint":false,"idempotentHint":!matches!(c.op,"job.cancel"),"openWorldHint":matches!(c.op,"account.refresh"|"voice.list"|"voice.get"|"voice.settings"|"model.list"|"job.submit")}})
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
        if [
            "sample_rate",
            "channels",
            "waveform_bins",
            "start_ms",
            "end_ms",
            "fade_in_ms",
            "fade_out_ms",
            "loop_start_ms",
            "loop_end_ms",
            "silence_min_ms",
        ]
        .contains(key)
        {
            let number = value.as_u64().ok_or("invalid_audio_integer")?;
            let valid = match *key {
                "sample_rate" => {
                    [8000, 16000, 22050, 24000, 32000, 44100, 48000, 96000].contains(&number)
                }
                "channels" => (1..=2).contains(&number),
                "waveform_bins" => (16..=256).contains(&number),
                "silence_min_ms" => (1..=86400000).contains(&number),
                _ => number <= 86400000,
            };
            if !valid {
                return Err("audio_parameter_out_of_range");
            }
            continue;
        }
        if ["gain_db", "silence_db", "clip_level"].contains(key) {
            let (min, max) = match *key {
                "gain_db" => (-60.0, 24.0),
                "silence_db" => (-120.0, 0.0),
                _ => (0.01, 1.0),
            };
            if !value.as_f64().is_some_and(|n| n >= min && n <= max) {
                return Err("audio_parameter_out_of_range");
            }
            continue;
        }
        if *key == "limit" {
            if !value.as_u64().is_some_and(|n| (1..=10).contains(&n)) {
                return Err("invalid_limit");
            }
            continue;
        }
        if [
            "request_limit",
            "credit_limit",
            "reserve_credits",
            "duration_ms",
        ]
        .contains(key)
        {
            let n = value.as_i64().ok_or("invalid_numeric_field")?;
            let (min, max) = match *key {
                "request_limit" => (1, 1000000),
                "duration_ms" => (500, 30000),
                _ => (1, 1000000000),
            };
            if n < min || n > max {
                return Err("numeric_field_out_of_range");
            }
            continue;
        }
        if *key == "loop" {
            if !value.is_boolean() {
                return Err("invalid_boolean");
            }
            continue;
        }
        if *key == "prompt_influence" {
            if !value.as_f64().is_some_and(|n| (0.0..=1.0).contains(&n)) {
                return Err("invalid_prompt_influence");
            }
            continue;
        }
        if *key == "voice_settings" {
            let fields = value.as_object().ok_or("invalid_voice_settings")?;
            for (name, v) in fields {
                if name == "use_speaker_boost" {
                    if !v.is_boolean() {
                        return Err("invalid_voice_settings");
                    }
                } else {
                    let range = match name.as_str() {
                        "stability" | "similarity_boost" | "style" => (0.0, 1.0),
                        "speed" => (0.7, 1.2),
                        _ => return Err("invalid_voice_settings"),
                    };
                    if !v.as_f64().is_some_and(|n| n >= range.0 && n <= range.1) {
                        return Err("invalid_voice_settings");
                    }
                }
            }
            continue;
        }
        let text = value.as_str().ok_or("invalid_field_type")?;
        let max = match *key {
            "description" | "text" => 2048,
            "source_note" | "notes" => 1024,
            "workspace" | "source_path" | "ffmpeg_path" | "ffprobe_path" => 4096,
            _ => 200,
        };
        if text.len() > max || text.contains('\0') {
            return Err("invalid_field_length");
        }
        if (*key == "project_id"
            || key.ends_with("_id") && !matches!(*key, "voice_id" | "model_id")
            || *key == "after")
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
    if op == "audio.analyze" || op == "audio.edit" {
        if command.get("loop_start_ms").is_some() != command.get("loop_end_ms").is_some()
            || command.get("loop_end_ms").is_some()
                && command["loop_end_ms"].as_u64() <= command["loop_start_ms"].as_u64()
        {
            return Err("invalid_audio_loop");
        }
    }
    if op == "audio.edit" {
        let start = command["start_ms"].as_u64().unwrap();
        let end = command["end_ms"].as_u64().unwrap();
        if end <= start
            || command["fade_in_ms"].as_u64().unwrap() + command["fade_out_ms"].as_u64().unwrap()
                > end - start
        {
            return Err("invalid_audio_trim_or_fade");
        }
        if command.get("loop_end_ms").is_some()
            && command["loop_end_ms"].as_u64().unwrap() > end - start
        {
            return Err("invalid_audio_loop");
        }
        if command["version_id"] == command["parent_version_id"] {
            return Err("audio_target_is_source");
        }
        if !["wav_pcm_s16le", "ogg_vorbis_q4", "mp3_128k"]
            .contains(&command["output_format"].as_str().unwrap_or(""))
        {
            return Err("audio_output_format_unsupported");
        }
    }
    if op == "account.create"
        && !["mock", "elevenlabs"].contains(&command["provider"].as_str().unwrap_or(""))
    {
        return Err("invalid_provider");
    }
    if op == "job.submit" {
        let kind = command["generation_kind"].as_str().unwrap_or("");
        if !["tts", "sfx"].contains(&kind) || command["output_format"] != "mp3_44100_128" {
            return Err("unsupported_generation_format_or_kind");
        }
        if kind == "tts"
            && (command["voice_id"].as_str().unwrap_or("").is_empty()
                || command.get("duration_ms").is_some()
                || command.get("loop").is_some()
                || command.get("prompt_influence").is_some())
        {
            return Err("invalid_tts_parameters");
        }
        if kind == "sfx"
            && (command.get("duration_ms").is_none()
                || ["voice_id", "voice_settings", "dialogue_id", "language"]
                    .iter()
                    .any(|k| command.get(*k).is_some()))
        {
            return Err("invalid_sfx_parameters");
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
