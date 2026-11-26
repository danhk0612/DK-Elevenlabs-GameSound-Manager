use dk_gamesound::{protocol, request, runtime};
use serde_json::{json, Value};
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--help"] {
        println!("dk-sound-cli version|status|shutdown\ndk-sound-cli request < request.json\ndk-sound-cli schema\ndk-sound-cli validate < request.json (offline syntax/fields only)\nJSON stdout; exit 0 success, 1 request/runtime error. Windows; Linux headless requires DK_SOUND_ROOT.");
        return;
    }
    if args == ["schema"] {
        println!(
            "{}",
            json!({"protocol_version":1,"tools":dk_gamesound::catalog::tools()})
        );
        return;
    }
    let command: Result<Value, &'static str> = match args.as_slice() {
        [op] if ["version", "status", "shutdown"].contains(&op.as_str()) => {
            Ok(json!({"protocol_version":1,"op":op}))
        }
        [op] if op == "request" || op == "validate" => {
            let mut bytes = Vec::new();
            match io::stdin()
                .take(protocol::MAX_FRAME as u64 + 1)
                .read_to_end(&mut bytes)
            {
                Ok(_) if bytes.len() <= protocol::MAX_FRAME => serde_json::from_slice(
                    bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes),
                )
                .map_err(|_| "invalid_json"),
                _ => Err("invalid_input_size_or_io"),
            }
        }
        _ => Err("invalid_arguments"),
    };
    let response = match command {
        Ok(command) if args == ["validate"] => match protocol::validate(&command) {
            Ok(()) => protocol::success(
                json!({"valid":true,"op":command["op"],"scope":"syntax_and_fields_only","engine_started":false}),
            ),
            Err(code) => protocol::error(code),
        },
        Ok(command) => match runtime() {
            Ok(runtime) => runtime.block_on(request(command)),
            Err(_) => protocol::error("runtime_unavailable"),
        },
        Err(code) => protocol::error(code),
    };
    println!("{response}");
    if response["ok"] != true {
        std::process::exit(1);
    }
}
