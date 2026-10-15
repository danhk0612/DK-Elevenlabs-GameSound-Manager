use fs2::FileExt;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
fn command(db: &mut Connection, c: &Value) -> Result<Value, String> {
    match c["op"].as_str().unwrap_or("") {
        "status" => {
            let mut s = db
                .prepare("SELECT id,state,cost FROM job ORDER BY id")
                .map_err(|e| e.to_string())?;
            let rows = s.query_map([], |r|Ok(json!({"id":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"cost":r.get::<_,i64>(2)?}))).map_err(|e|e.to_string())?;
            let jobs: Vec<Value> = rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?;
            let used: i64 = db
                .query_row("SELECT COALESCE(sum(cost),0) FROM job", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            Ok(
                json!({"ok":true,"pid":std::process::id(),"jobs":jobs,"reserved":used,"fixture_limit":10}),
            )
        }
        "submit" => {
            let id = c["id"]
                .as_str()
                .filter(|s| {
                    !s.is_empty()
                        && s.len() <= 64
                        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
                })
                .ok_or("invalid fixture id")?;
            let delay = c["delay_ms"]
                .as_i64()
                .filter(|n| (0..=30000).contains(n))
                .ok_or("invalid delay")?;
            let cost = c["cost"]
                .as_i64()
                .filter(|n| (1..=10).contains(n))
                .ok_or("invalid cost")?;
            let tx = db.transaction().map_err(|e| e.to_string())?;
            let old: Result<(i64, i64), _> =
                tx.query_row("SELECT delay,cost FROM job WHERE id=?", [id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                });
            if let Ok((d, k)) = old {
                return if d == delay && k == cost {
                    Ok(json!({"ok":true,"id":id,"duplicate":true}))
                } else {
                    Err("id conflict".into())
                };
            }
            let used: i64 = tx
                .query_row("SELECT COALESCE(sum(cost),0) FROM job", [], |r| r.get(0))
                .map_err(|e| e.to_string())?;
            if used + cost > 10 {
                return Err("fixture budget exceeded".into());
            }
            tx.execute(
                "INSERT INTO job VALUES (?,'running',?,?,?)",
                params![id, now() + delay, cost, delay],
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
            Ok(json!({"ok":true,"id":id,"duplicate":false}))
        }
        "shutdown" => Ok(json!({"ok":true,"shutdown":true})),
        _ => Err("unknown operation".into()),
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = t00_spike::root()?;
    fs::create_dir_all(&root)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root.join("owner.lock"))?;
    lock.try_lock_exclusive()
        .map_err(|_| "engine already owns root")?;
    let mut db = Connection::open(root.join("spike.sqlite"))?;
    db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS job(id TEXT PRIMARY KEY,state TEXT NOT NULL,due INTEGER,cost INTEGER,delay INTEGER); UPDATE job SET state='interrupted' WHERE state='running';")?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let token = uuid::Uuid::new_v4().to_string();
    let endpoint = json!({"port":listener.local_addr()?.port(),"token":token});
    let path = root.join("endpoint.json");
    fs::write(&path, endpoint.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    loop {
        let mut s = db.prepare("SELECT id FROM job WHERE state='running' AND due<=?")?;
        let due = s
            .query_map([now()], |r| r.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        drop(s);
        for id in due {
            let tmp = root.join(format!("{id}.tmp"));
            let out = root.join(format!("{id}.fixture"));
            fs::write(&tmp, format!("fixture:{id}"))?;
            if std::env::var("T00_SPIKE_CRASH_POINT").as_deref() == Ok("after_temp") {
                std::process::exit(86);
            }
            fs::rename(&tmp, &out)?;
            if std::env::var("T00_SPIKE_CRASH_POINT").as_deref() == Ok("after_rename") {
                std::process::exit(86);
            }
            db.execute("UPDATE job SET state='succeeded' WHERE id=?", [id])?;
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Winsock accepted sockets inherit the listener nonblocking mode.
                stream.set_nonblocking(false)?;
                stream.set_read_timeout(Some(Duration::from_secs(2)))?;
                stream.set_write_timeout(Some(Duration::from_secs(2)))?;
                let mut line = String::new();
                let read = BufReader::new(&stream).take(65536).read_line(&mut line);
                let r = if read.is_err() {
                    json!({"ok":false,"error":"read failed"})
                } else {
                    let v: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
                    if v["token"] != token {
                        json!({"ok":false,"error":"unauthorized"})
                    } else {
                        command(&mut db, &v["command"])
                            .unwrap_or_else(|e| json!({"ok":false,"error":e}))
                    }
                };
                let _ = writeln!(stream, "{r}");
                if r["shutdown"] == true {
                    db.execute(
                        "UPDATE job SET state='interrupted' WHERE state='running'",
                        [],
                    )?;
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10))
            }
            Err(e) => return Err(e.into()),
        }
    }
    fs::remove_file(path)?;
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
