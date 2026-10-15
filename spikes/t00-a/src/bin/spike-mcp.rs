use serde_json::{json, Value};
use std::io::{self, BufRead};
fn main() {
    let mut initialized = false;
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                println!(
                    "{}",
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}})
                );
                continue;
            }
        };
        if v["method"] == "notifications/initialized" {
            initialized = true;
            continue;
        }
        if v.get("id").is_none() {
            continue;
        }
        let method = v["method"].as_str().unwrap_or("");
        let result = match method {
            "initialize" => Ok(
                json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"t00-spike","version":"0.0.0"}}),
            ),
            "ping" => Ok(json!({})),
            "tools/list" if initialized => Ok(
                json!({"tools":[{"name":"spike_command","description":"Isolated fixture commands only","inputSchema":{"type":"object","properties":{"op":{"type":"string","enum":["status","submit"]},"id":{"type":"string"},"delay_ms":{"type":"integer"},"cost":{"type":"integer"}},"required":["op"],"additionalProperties":false}}]}),
            ),
            "tools/call" if initialized && v["params"]["name"] == "spike_command" => {
                let c = v["params"]["arguments"].clone();
                let r = if c["op"] == "status" || c["op"] == "submit" {
                    t00_spike::response(c)
                } else {
                    json!({"ok":false,"error":"unsupported tool operation"})
                };
                Ok(
                    json!({"content":[{"type":"text","text":r.to_string()}],"isError":r["ok"]!=true}),
                )
            }
            _ => Err(json!({"code":-32601,"message":"Unsupported method or not initialized"})),
        };
        let r = match result {
            Ok(r) => json!({"jsonrpc":"2.0","id":v["id"],"result":r}),
            Err(e) => json!({"jsonrpc":"2.0","id":v["id"],"error":e}),
        };
        println!("{r}");
    }
}
