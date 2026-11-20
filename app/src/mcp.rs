use serde_json::{json, Value};

pub const VERSION: &str = "2025-11-25";
#[derive(Default)]
pub struct Session {
    negotiated: bool,
    ready: bool,
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

impl Session {
    pub async fn handle(&mut self, message: Value) -> Option<Value> {
        let id = message.get("id").cloned();
        if !message.is_object()
            || message["jsonrpc"] != "2.0"
            || !message["method"].is_string()
            || id
                .as_ref()
                .is_some_and(|v| !(v.is_null() || v.is_string() || v.is_i64() || v.is_u64()))
        {
            return Some(error(Value::Null, -32600, "Invalid request"));
        }
        let method = message["method"].as_str().unwrap();
        if id.is_none() {
            if method == "notifications/initialized" && self.negotiated {
                self.ready = true;
            }
            return None;
        }
        let id = id.unwrap();
        let result = match method {
            "initialize" if !self.negotiated => {
                if message["params"]["protocolVersion"] != VERSION {
                    return Some(error(id, -32602, "Only protocol 2025-11-25 is supported"));
                }
                self.negotiated = true;
                json!({"protocolVersion":VERSION,"capabilities":{"tools":{}},
                    "serverInfo":{"name":"dk-gamesound","version":env!("CARGO_PKG_VERSION")}})
            }
            "ping" => json!({}),
            "tools/list" if self.ready => {
                json!({"tools":[tool("sound_version"), tool("sound_status")]})
            }
            "tools/call" if self.ready => {
                let op = match message["params"]["name"].as_str() {
                    Some("sound_version") => "version",
                    Some("sound_status") => "status",
                    _ => return Some(error(id, -32602, "Unknown tool")),
                };
                if let Some(arguments) = message["params"].get("arguments") {
                    if !arguments.as_object().is_some_and(|v| v.is_empty()) {
                        return Some(error(id, -32602, "Tool arguments must be an empty object"));
                    }
                }
                let response = crate::request(json!({"protocol_version":1,"op":op})).await;
                json!({"content":[{"type":"text","text":response.to_string()}],
                    "structuredContent":response,"isError":response["ok"]!=true})
            }
            "tools/list" | "tools/call" => {
                return Some(error(id, -32002, "Initialize session first"))
            }
            _ => return Some(error(id, -32601, "Method not found")),
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }
}

fn tool(name: &str) -> Value {
    json!({"name":name,"description":"Query the local DK GameSound engine; starts it if needed. No sound generation or game writes.",
        "inputSchema":{"type":"object","properties":{},"additionalProperties":false},
        "annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})
}

pub fn parse_error() -> Value {
    error(Value::Null, -32700, "Parse error")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handshake_notifications_and_tool_arguments() {
        crate::runtime().unwrap().block_on(async {
            let mut session = Session::default();
            let request = |method: &str, params: Value| json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
            assert_eq!(session.handle(request("tools/list",json!({}))).await.unwrap()["error"]["code"],-32002);
            assert_eq!(session.handle(request("initialize",json!({"protocolVersion":"unknown"}))).await.unwrap()["error"]["code"],-32602);
            assert_eq!(session.handle(request("initialize",json!({"protocolVersion":VERSION}))).await.unwrap()["result"]["protocolVersion"],VERSION);
            assert!(session.handle(json!({"jsonrpc":"2.0","method":"notifications/initialized"})).await.is_none());
            assert_eq!(session.handle(request("tools/list",json!({}))).await.unwrap()["result"]["tools"].as_array().unwrap().len(),2);
            for params in [json!({"name":"sound_status","arguments":{"approved":true}}), json!({"name":"apply"})] {
                assert_eq!(session.handle(request("tools/call",params)).await.unwrap()["error"]["code"],-32602);
            }
        });
    }
}
