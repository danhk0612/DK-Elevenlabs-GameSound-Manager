//! Local audio processing; explicit output/inspection parameters, immutable candidates.
use crate::{
    audio_process,
    storage::{self, Error, Result, Store},
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::Duration,
};
const PCM_LIMIT: usize = 512 * 1024 * 1024;
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn n(v: &Value, key: &str) -> f64 {
    v[key].as_f64().unwrap_or(0.0)
}
fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|v| v.to_string()).collect()
}
pub fn migrate(db: &mut Connection) -> Result<()> {
    let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version == 2 {
        let tx = db.transaction()?;
        tx.execute_batch("CREATE TABLE audio_tools(id INTEGER PRIMARY KEY CHECK(id=1),settings TEXT NOT NULL);
CREATE TABLE audio_operations(id TEXT PRIMARY KEY,project_id TEXT NOT NULL REFERENCES projects(id),kind TEXT NOT NULL,snapshot TEXT NOT NULL,input_hash TEXT NOT NULL,state TEXT NOT NULL,result TEXT,error TEXT,extension TEXT,output_hash TEXT,asset_id TEXT NOT NULL,version_id TEXT NOT NULL);
CREATE UNIQUE INDEX audio_edit_target ON audio_operations(project_id,asset_id,version_id) WHERE kind='audio.edit';")?;
        tx.pragma_update(None, "user_version", 3)?;
        tx.commit()?;
    }

    Ok(())
}
fn exe(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    if !path.is_absolute() || !path.is_file() {
        return Err(Error("audio_tool_path_invalid"));
    }
    #[cfg(windows)]
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_none_or(|s| !s.eq_ignore_ascii_case("exe"))
    {
        return Err(Error("audio_tool_requires_exe"));
    }
    storage::no_link(path)?;
    Ok(path.canonicalize()?)
}
fn tool(path: &str) -> Result<Value> {
    let path = exe(path)?;
    let bytes = audio_process::run(
        &path,
        &strings(&["-version"]),
        None,
        64 * 1024,
        Duration::from_secs(5),
    )?;
    let text = String::from_utf8(bytes).map_err(|_| Error("audio_tool_version_invalid"))?;
    let version = text
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(160)
        .collect::<String>();
    if !(version.starts_with("ffmpeg version ") || version.starts_with("ffprobe version ")) {
        return Err(Error("audio_tool_version_invalid"));
    }
    Ok(json!({"path":path,"sha256":storage::hash_file(&path)?.0,"version":version}))
}
fn tools(store: &Store) -> Result<Value> {
    let text: String = store
        .catalog
        .query_row("SELECT settings FROM audio_tools WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()?
        .ok_or(Error("audio_tools_not_configured"))?;
    let v: Value = serde_json::from_str(&text).map_err(|_| Error("audio_tools_invalid"))?;
    verify_tools(&v)?;
    Ok(v)
}
pub(crate) fn verify_tools(v: &Value) -> Result<()> {
    for key in ["ffmpeg", "ffprobe"] {
        let path = exe(s(&v[key], "path"))?;
        if storage::hash_file(&path)?.0 != s(&v[key], "sha256") {
            return Err(Error("audio_tool_changed"));
        }
    }
    Ok(())
}
fn versions(tools: &Value) -> Value {
    json!({"ffmpeg":{"version":tools["ffmpeg"]["version"],"sha256":tools["ffmpeg"]["sha256"]},"ffprobe":{"version":tools["ffprobe"]["version"],"sha256":tools["ffprobe"]["sha256"]}})
}
pub(crate) fn source(store: &mut Store, c: &Value) -> Result<Value> {
    let command = json!({"protocol_version":1,"op":"asset.file","project_id":c["project_id"],"asset_id":c["asset_id"],"version_id":if c["op"]=="audio.edit"{&c["parent_version_id"]}else{&c["version_id"]}});
    let result = store.execute(&command);
    if result["ok"] != true {
        return Err(Error("audio_source_unavailable"));
    }
    Ok(result["data"].clone())
}
fn probe(tools: &Value, path: &Path, control: &crate::audio_queue::Control) -> Result<Value> {
    let mut args = strings(&[
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-format_whitelist",
        "wav,mp3,ogg,flac,mov,aac,aiff,asf",
        "-select_streams",
        "a:0",
        "-show_entries",
        "stream=codec_name,sample_rate,channels:format=format_name,duration",
        "-of",
        "json",
    ]);
    args.push(path.to_string_lossy().to_string());
    let bytes = audio_process::run_controlled(
        Path::new(s(&tools["ffprobe"], "path")),
        &args,
        None,
        128 * 1024,
        Duration::from_secs(15),
        &|| control.check(),
    )?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|_| Error("audio_probe_invalid"))?;
    let stream = v["streams"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or(Error("audio_stream_missing"))?;
    let rate = stream["sample_rate"]
        .as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|n| (8000..=192000).contains(n))
        .ok_or(Error("audio_sample_rate_unsupported"))?;
    let channels = stream["channels"]
        .as_u64()
        .filter(|n| (1..=8).contains(n))
        .ok_or(Error("audio_channels_unsupported"))?;
    Ok(
        json!({"codec":stream["codec_name"],"sample_rate":rate,"channels":channels,"format":v["format"]["format_name"]}),
    )
}
fn op_dir(store: &Store, id: &str) -> Result<PathBuf> {
    let parent = store.root.join("audio-operations");
    storage::directory(&parent)?;
    let dir = parent.join(id);
    storage::directory(&dir)?;
    Ok(dir)
}
fn analyze(
    store: &Store,
    c: &Value,
    input: &Value,
    tools: &Value,
    control: &crate::audio_queue::Control,
) -> Result<Value> {
    let path = Path::new(s(input, "path"));
    control.phase("probing")?;
    let info = probe(tools, path, control)?;
    let dir = op_dir(store, s(c, "operation_id"))?;
    let pcm = dir.join("decoded.f32");
    storage::no_link(&pcm)?;
    let output = OpenOptions::new().write(true).create_new(true).open(&pcm)?;
    let mut args = strings(&[
        "-nostdin",
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-format_whitelist",
        "wav,mp3,ogg,flac,mov,aac,aiff,asf",
        "-threads",
        "1",
        "-i",
    ]);
    args.push(path.to_string_lossy().into());
    args.extend(strings(&[
        "-map",
        "0:a:0",
        "-vn",
        "-sn",
        "-dn",
        "-c:a",
        "pcm_f32le",
        "-f",
        "f32le",
        "pipe:1",
    ]));
    control.phase("decoding")?;
    audio_process::run_controlled(
        Path::new(s(&tools["ffmpeg"], "path")),
        &args,
        Some(output),
        PCM_LIMIT,
        Duration::from_secs(90),
        &|| control.check(),
    )?;
    control.phase("measuring")?;
    let stats = statistics_controlled(
        &pcm,
        info["sample_rate"].as_u64().unwrap(),
        info["channels"].as_u64().unwrap(),
        c,
        &|| control.check(),
    )?;
    if storage::hash_file(path)?.0 != s(input, "sha256") {
        return Err(Error("audio_source_changed"));
    }
    fs::remove_file(&pcm)?;
    Ok(
        json!({"measurement":"decoded_first_audio_stream_sample_metrics","metadata":info,"metrics":stats,"tools":versions(tools),"conditions":c,"input_sha256":input["sha256"],"subjective_quality":"not_evaluated","approval":"not_granted"}),
    )
}
fn db(value: f64) -> Value {
    if value > 0.0 {
        json!(20.0 * value.log10())
    } else {
        Value::Null
    }
}
#[cfg(test)]
fn statistics(path: &Path, rate: u64, channels: u64, c: &Value) -> Result<Value> {
    statistics_controlled(path, rate, channels, c, &|| Ok(()))
}
fn statistics_controlled(
    path: &Path,
    rate: u64,
    channels: u64,
    c: &Value,
    check: &dyn Fn() -> Result<()>,
) -> Result<Value> {
    check()?;
    let file = File::open(path)?;
    let bytes = file.metadata()?.len();
    let stride = channels * 4;
    if bytes == 0 || bytes % stride != 0 {
        return Err(Error("audio_pcm_invalid"));
    }
    let frames = bytes / stride;
    let duration = frames as f64 * 1000.0 / rate as f64;
    if c.get("loop_end_ms").is_some() && n(c, "loop_end_ms") > duration {
        return Err(Error("audio_loop_out_of_range"));
    }
    let count = (c["waveform_bins"].as_u64().unwrap()).min(frames) as usize;
    let mut wave = vec![(f64::INFINITY, f64::NEG_INFINITY); count];
    let mut reader = BufReader::new(file);
    let mut sample = [0u8; 4];
    let mut peak = 0.0f64;
    let mut sum = 0.0;
    let mut clipped = 0u64;
    let mut quiet = 0u64;
    let mut quiet_start = None;
    let mut longest = 0u64;
    let mut regions = Vec::new();
    let mut region_count = 0u64;
    let silence = 10.0f64.powf(n(c, "silence_db") / 20.0);
    let clip = n(c, "clip_level");
    let loop_start = (n(c, "loop_start_ms") * rate as f64 / 1000.0).round() as u64;
    let loop_end = (n(c, "loop_end_ms") * rate as f64 / 1000.0).round() as u64;
    if c.get("loop_end_ms").is_some() && (loop_end <= loop_start || loop_end > frames) {
        return Err(Error("audio_loop_out_of_range"));
    }
    let mut starts = vec![0.0; channels as usize];
    let mut ends = starts.clone();
    let mut close_region = |start: u64, end: u64| {
        let ms = (end - start) as f64 * 1000.0 / rate as f64;
        if ms >= n(c, "silence_min_ms") {
            region_count += 1;
            if regions.len() < 32 {
                regions.push(json!({"start_ms":start as f64*1000.0/rate as f64,"end_ms":end as f64*1000.0/rate as f64}));
            }
        }
    };
    for frame in 0..frames {
        if frame % 4096 == 0 {
            check()?;
        }
        let index = (frame * count as u64 / frames) as usize;
        let mut frame_peak = 0.0f64;
        for ch in 0..channels {
            reader.read_exact(&mut sample)?;
            let v = f32::from_le_bytes(sample) as f64;
            if !v.is_finite() {
                return Err(Error("audio_pcm_nonfinite"));
            }
            peak = peak.max(v.abs());
            sum += v * v;
            frame_peak = frame_peak.max(v.abs());
            if v.abs() >= clip {
                clipped += 1;
            }
            wave[index].0 = wave[index].0.min(v);
            wave[index].1 = wave[index].1.max(v);
            if frame == loop_start {
                starts[ch as usize] = v;
            }
            if loop_end > 0 && frame == loop_end - 1 {
                ends[ch as usize] = v;
            }
        }
        if frame_peak <= silence {
            quiet += 1;
            quiet_start.get_or_insert(frame);
        } else if let Some(start) = quiet_start.take() {
            longest = longest.max(frame - start);
            close_region(start, frame);
        }
    }
    if let Some(start) = quiet_start {
        longest = longest.max(frames - start);
        close_region(start, frames);
    }
    let rms = (sum / (frames * channels) as f64).sqrt();
    let seam = if c.get("loop_end_ms").is_some() {
        Some(
            starts
                .iter()
                .zip(&ends)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max),
        )
    } else {
        None
    };
    Ok(
        json!({"frames":frames,"duration_ms":duration,"sample_peak":peak,"sample_peak_dbfs":db(peak),"rms":rms,"rms_dbfs":db(rms),"threshold_exceeding_samples":clipped,"samples":frames*channels,
    "quiet_ms":quiet as f64*1000.0/rate as f64,"longest_quiet_ms":longest as f64*1000.0/rate as f64,"silence_region_count":region_count,"silence_regions":regions,"silence_regions_truncated":region_count>32,"waveform":wave,"waveform_channels":"combined_min_max","loop_seam_sample_delta":seam,"true_peak_lufs":"not_measured"}),
    )
}
pub(crate) fn operation(store: &Store, project: &str, id: &str) -> Result<Value> {
    store.project(project)?;
    store.catalog.query_row("SELECT kind,state,result,error,input_hash,snapshot,cancel_requested,phase,tools_snapshot FROM audio_operations WHERE project_id=?1 AND id=?2",params![project,id],|r|{
        let result:Option<String>=r.get(2)?;let request:String=r.get(5)?;
        Ok(json!({"operation_id":id,"project_id":project,"kind":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"result":result.and_then(|s|serde_json::from_str::<Value>(&s).ok()),"error":r.get::<_,Option<String>>(3)?,"input_sha256":r.get::<_,String>(4)?,"request":serde_json::from_str::<Value>(&request).ok(),"cancel_requested":r.get::<_,bool>(6)?,"phase":r.get::<_,String>(7)?,"tool_snapshot":r.get::<_,Option<String>>(8)?.and_then(|t|serde_json::from_str::<Value>(&t).ok()).map(|v|versions(&v))}))
    }).optional()?.ok_or(Error("audio_operation_not_found"))
}
fn edit(
    store: &mut Store,
    c: &Value,
    input: &Value,
    tools: &Value,
    control: &crate::audio_queue::Control,
) -> Result<Value> {
    let path = Path::new(s(input, "path"));
    // Decode once to verify real frame count; container duration can be inaccurate.
    let mut measure = c.clone();
    measure["waveform_bins"] = json!(16);
    measure["silence_db"] = json!(-120);
    measure["silence_min_ms"] = json!(1);
    measure["clip_level"] = json!(1.0);
    // Edit has optional loop metadata in output coordinates, not source coordinates.
    measure.as_object_mut().unwrap().remove("loop_start_ms");
    measure.as_object_mut().unwrap().remove("loop_end_ms");
    let report = analyze(store, &measure, input, tools, control)?;
    if n(c, "end_ms") > n(&report["metrics"], "duration_ms") + 0.001 {
        return Err(Error("audio_trim_out_of_range"));
    }
    let duration = (n(c, "end_ms") - n(c, "start_ms")) / 1000.0;
    let mut filters = format!(
        "asetpts=PTS-STARTPTS,atrim=start={}:end={},asetpts=PTS-STARTPTS,volume={}dB",
        n(c, "start_ms") / 1000.0,
        n(c, "end_ms") / 1000.0,
        n(c, "gain_db")
    );
    if n(c, "fade_in_ms") > 0.0 {
        filters += &format!(
            ",afade=t=in:st=0:d={}:curve=tri",
            n(c, "fade_in_ms") / 1000.0
        );
    }
    if n(c, "fade_out_ms") > 0.0 {
        filters += &format!(
            ",afade=t=out:st={}:d={}:curve=tri",
            duration - n(c, "fade_out_ms") / 1000.0,
            n(c, "fade_out_ms") / 1000.0
        );
    }
    let (ext, codec) = match s(c, "output_format") {
        "wav_pcm_s16le" => ("wav", strings(&["-c:a", "pcm_s16le", "-f", "wav"])),
        "ogg_vorbis_q4" => (
            "ogg",
            strings(&["-c:a", "libvorbis", "-q:a", "4", "-f", "ogg"]),
        ),
        "mp3_128k" => (
            "mp3",
            strings(&["-c:a", "libmp3lame", "-b:a", "128k", "-f", "mp3"]),
        ),
        _ => return Err(Error("audio_output_format_unsupported")),
    };
    let dir = op_dir(store, s(c, "operation_id"))?;
    let file = dir.join(format!("output.{ext}"));
    storage::no_link(&file)?;
    let output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)?;
    let mut args = strings(&[
        "-nostdin",
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-format_whitelist",
        "wav,mp3,ogg,flac,mov,aac,aiff,asf",
        "-threads",
        "1",
        "-i",
    ]);
    args.push(path.to_string_lossy().into());
    args.extend(strings(&[
        "-map",
        "0:a:0",
        "-map_metadata",
        "-1",
        "-vn",
        "-sn",
        "-dn",
        "-af",
    ]));
    args.push(filters);
    args.extend(strings(&[
        "-ar",
        &c["sample_rate"].to_string(),
        "-ac",
        &c["channels"].to_string(),
    ]));
    args.extend(codec);
    args.push("pipe:1".into());
    control.phase("encoding")?;
    audio_process::run_controlled(
        Path::new(s(&tools["ffmpeg"], "path")),
        &args,
        Some(output),
        PCM_LIMIT,
        Duration::from_secs(90),
        &|| control.check(),
    )?;
    control.phase("checking_output")?;
    let output_info = probe(tools, &file, control)?;
    if output_info["sample_rate"] != c["sample_rate"] || output_info["channels"] != c["channels"] {
        return Err(Error("audio_output_mismatch"));
    }
    if storage::hash_file(path)?.0 != s(input, "sha256") {
        return Err(Error("audio_source_changed"));
    }
    let hash = storage::hash_file(&file)?.0;
    let details = json!({"tools":versions(tools),"output_metadata":output_info,"loop_start_ms":c["loop_start_ms"],"loop_end_ms":c["loop_end_ms"],"approval":"not_granted"});
    crate::audio_queue::publish_output(store, s(c, "operation_id"), ext, &hash, &details)?;
    attach(store, c, input, &details)?;
    operation(store, s(c, "project_id"), s(c, "operation_id"))
}
fn attach(store: &mut Store, c: &Value, input: &Value, details: &Value) -> Result<()> {
    let (ext, hash): (String, String) = store.catalog.query_row(
        "SELECT extension,output_hash FROM audio_operations WHERE id=?1 AND state='output_ready'",
        [s(c, "operation_id")],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if !["wav", "ogg", "mp3"].contains(&ext.as_str()) {
        return Err(Error("audio_output_invalid"));
    }
    let dir = op_dir(store, s(c, "operation_id"))?;
    let file = dir.join(format!("output.{ext}"));
    storage::no_link(&file)?;
    if storage::hash_file(&file)?.0 != hash {
        return Err(Error("audio_output_integrity_mismatch"));
    }
    let mut cmd = json!({"protocol_version":1,"op":"asset.import","project_id":c["project_id"],"asset_id":c["asset_id"],"version_id":c["version_id"],"parent_version_id":c["parent_version_id"],"name":input["name"],"kind":input["kind"],"source_path":file,"source_note":format!("local audio edit {}; candidate only",s(c,"operation_id"))});
    let (root, mut db) = store.project(s(c, "project_id"))?;
    // file_info intentionally omits asset name/kind: load scoped metadata here.
    let (name, kind): (String, String) = db.query_row(
        "SELECT name,kind FROM assets WHERE id=?1",
        [s(c, "asset_id")],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    cmd["name"] = json!(name);
    cmd["kind"] = json!(kind);
    let candidate = storage::import_generated(&root, &mut db, &cmd)?;
    let mut result = details.clone();
    result["candidate"] = candidate;
    store.catalog.execute(
        "UPDATE audio_operations SET state='succeeded',phase='completed',result=?2,error=NULL WHERE id=?1",
        params![s(c, "operation_id"), result.to_string()],
    )?;
    Ok(())
}
pub fn command(store: &mut Store, c: &Value) -> Result<Value> {
    let op = s(c, "op");
    if op == "audio.submit" {
        return crate::audio_queue::submit(store, c);
    }
    if op == "audio.cancel" {
        return crate::audio_queue::cancel(store, c);
    }
    if op == "audio.tools.set" {
        let ffmpeg = tool(s(c, "ffmpeg_path"))?;
        let ffprobe = tool(s(c, "ffprobe_path"))?;
        if !s(&ffmpeg, "version").starts_with("ffmpeg ")
            || !s(&ffprobe, "version").starts_with("ffprobe ")
        {
            return Err(Error("audio_tool_version_invalid"));
        }
        let settings = json!({"ffmpeg":ffmpeg,"ffprobe":ffprobe});
        store.catalog.execute("INSERT INTO audio_tools VALUES(1,?1) ON CONFLICT(id) DO UPDATE SET settings=excluded.settings",[settings.to_string()])?;
        return Ok(settings);
    }
    if op == "audio.tools.get" {
        return tools(store);
    }
    if op == "audio.list" {
        store.project(s(c, "project_id"))?;
        let limit = c["limit"].as_i64().unwrap_or(10);
        let mut statement=store.catalog.prepare("SELECT id,kind,state,asset_id,version_id,error,cancel_requested,phase FROM audio_operations WHERE project_id=?1 AND id>?2 ORDER BY id LIMIT ?3")?;
        let items=statement.query_map(params![s(c,"project_id"),s(c,"after"),limit],|r|Ok(json!({"operation_id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"state":r.get::<_,String>(2)?,"asset_id":r.get::<_,String>(3)?,"version_id":r.get::<_,String>(4)?,"error":r.get::<_,Option<String>>(5)?,"cancel_requested":r.get::<_,bool>(6)?,"phase":r.get::<_,String>(7)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
        return Ok(
            json!({"next_after":if items.len()==limit as usize{items.last().map(|v|v["operation_id"].clone())}else{None},"items":items}),
        );
    }
    if op == "audio.get" {
        return operation(store, s(c, "project_id"), s(c, "operation_id"));
    }
    if op == "audio.recover" {
        let current = operation(store, s(c, "project_id"), s(c, "operation_id"))?;
        if current["state"] != "output_ready" {
            return Err(Error("audio_output_not_recoverable"));
        }
        let original = &current["request"];
        let input = source(store, original)?;
        if input["sha256"] != current["input_sha256"] {
            return Err(Error("audio_source_changed"));
        }
        attach(store, original, &input, &current["result"])?;
        return operation(store, s(c, "project_id"), s(c, "operation_id"));
    }
    if !["audio.analyze", "audio.edit"].contains(&op) {
        return Err(Error("unsupported_operation"));
    }
    let id = s(c, "operation_id");
    if let Some((project, snapshot)) = store
        .catalog
        .query_row(
            "SELECT project_id,snapshot FROM audio_operations WHERE id=?1",
            [id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?
    {
        if project != s(c, "project_id") || snapshot != c.to_string() {
            return Err(Error("audio_operation_id_conflict"));
        }
        return operation(store, &project, id);
    }
    let input = source(store, c)?;
    let tools = tools(store)?;
    if op == "audio.edit" {
        let (_, db) = store.project(s(c, "project_id"))?;
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM versions WHERE asset_id=?1 AND id=?2)",
            params![s(c, "asset_id"), s(c, "version_id")],
            |r| r.get(0),
        )?;
        let reserved:bool=store.catalog.query_row("SELECT EXISTS(SELECT 1 FROM generation_jobs WHERE project_id=?1 AND asset_id=?2 AND version_id=?3)",params![s(c,"project_id"),s(c,"asset_id"),s(c,"version_id")],|r|r.get(0))?;
        let local_reserved:bool=store.catalog.query_row("SELECT EXISTS(SELECT 1 FROM audio_operations WHERE project_id=?1 AND asset_id=?2 AND version_id=?3 AND kind='audio.edit')",params![s(c,"project_id"),s(c,"asset_id"),s(c,"version_id")],|r|r.get(0))?;
        if exists || reserved || local_reserved {
            return Err(Error("audio_target_reserved"));
        }
    }
    store.catalog.execute("INSERT INTO audio_operations(id,project_id,kind,snapshot,input_hash,state,asset_id,version_id) VALUES(?1,?2,?3,?4,?5,'processing',?6,?7)",params![id,s(c,"project_id"),op,c.to_string(),s(&input,"sha256"),s(c,"asset_id"),s(c,"version_id")])?;
    store.catalog.execute(
        "UPDATE audio_operations SET phase='starting',tools_snapshot=?2 WHERE id=?1",
        params![id, tools.to_string()],
    )?;
    let control = crate::audio_queue::Control::new(&store.root, id)?;
    let result = run_registered(store, c, &input, &tools, &control);
    if let Err(error) = &result {
        crate::audio_queue::fail(store, id, error)?;
    }
    result
}
pub(crate) fn run_registered(
    store: &mut Store,
    c: &Value,
    input: &Value,
    tools: &Value,
    control: &crate::audio_queue::Control,
) -> Result<Value> {
    let id = s(c, "operation_id");
    control.check()?;
    if c["op"] == "audio.analyze" {
        let report = analyze(store, c, input, tools, control)?;
        if store.catalog.execute("UPDATE audio_operations SET state='succeeded',phase='completed',result=?2 WHERE id=?1 AND state='processing' AND cancel_requested=0",params![id,report.to_string()])?==0{return Err(Error("audio_cancelled"));}
        operation(store, s(c, "project_id"), id)
    } else {
        edit(store, c, input, tools, control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_metrics_handle_silence_stereo_loops_and_invalid_pcm() {
        let path = std::env::temp_dir().join(format!(
            "dk-audio-metrics-{}-{}.pcm",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let conditions = json!({"waveform_bins":16,"silence_db":-60,"silence_min_ms":1,"clip_level":1.0,"loop_start_ms":0,"loop_end_ms":2});
        let mut bytes = Vec::new();
        for frame in 0..16 {
            for v in [
                0f32,
                if frame == 0 {
                    1f32
                } else if frame == 15 {
                    -1f32
                } else {
                    0f32
                },
            ] {
                bytes.extend(v.to_le_bytes());
            }
        }
        fs::write(&path, &bytes).unwrap();
        let result = statistics(&path, 8000, 2, &conditions).unwrap();
        assert_eq!(result["threshold_exceeding_samples"], 2);
        assert_eq!(result["samples"], 32);
        assert_eq!(result["loop_seam_sample_delta"], 2.0);
        assert_eq!(result["silence_region_count"], 1);
        assert_eq!(result["quiet_ms"], 1.75);
        fs::write(&path, vec![0u8; 128]).unwrap();
        let result = statistics(&path, 8000, 2, &conditions).unwrap();
        assert_eq!(result["rms_dbfs"], Value::Null);
        assert_eq!(result["sample_peak_dbfs"], Value::Null);
        assert_eq!(result["quiet_ms"], 2.0);
        bytes[0..4].copy_from_slice(&f32::NAN.to_le_bytes());
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            statistics(&path, 8000, 2, &conditions).unwrap_err().0,
            "audio_pcm_nonfinite"
        );
        fs::write(&path, b"partial").unwrap();
        assert_eq!(
            statistics(&path, 8000, 2, &conditions).unwrap_err().0,
            "audio_pcm_invalid"
        );
        fs::remove_file(path).unwrap();
    }
}
