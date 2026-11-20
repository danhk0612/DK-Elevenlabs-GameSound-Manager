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
                json!({"tools":crate::catalog::tools()})
            }
            "tools/call" if self.ready => {
                let tool_name = message["params"]["name"].as_str().unwrap_or("");
                let definition = crate::catalog::COMMANDS.iter().find(|c| {
                    c.op != "shutdown" && format!("sound_{}", c.op.replace('.', "_")) == tool_name
                });
                let Some(definition) = definition else {
                    return Some(error(id, -32602, "Unknown tool"));
                };
                let arguments = message["params"]
                    .get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                let Some(mut arguments) = arguments.as_object().cloned() else {
                    return Some(error(id, -32602, "Tool arguments must be an object"));
                };
                if arguments.contains_key("op") || arguments.contains_key("protocol_version") {
                    return Some(error(id, -32602, "Reserved arguments"));
                }
                arguments.insert("op".to_string(), json!(definition.op));
                arguments.insert("protocol_version".to_string(), json!(1));
                let response = crate::request(Value::Object(arguments)).await;
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
            assert_eq!(session.handle(request("tools/list",json!({}))).await.unwrap()["result"]["tools"].as_array().unwrap().len(),crate::catalog::COMMANDS.len()-1);
            for params in [json!({"name":"sound_status","arguments":{"approved":true}}), json!({"name":"apply"})] {
                let response=session.handle(request("tools/call",params)).await.unwrap();
                assert!(response.get("error").is_some() || response["result"]["isError"]==true);
            }
        });
    }
}
