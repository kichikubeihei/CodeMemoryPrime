use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::tools;

#[derive(Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub fn handle_request(req: RpcRequest, rt: &Runtime) -> Value {
    match req.method.as_str() {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "CodeMemoryPrime",
                    "version": "0.1.0"
                }
            })
        }
        "notifications/initialized" | "initialized" => {
            Value::Null
        }
        "ping" => {
            json!({})
        }
        "tools/list" => {
            json!({
                "tools": tools::list_all_tools()
            })
        }
        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let name = params.get("name").and_then(|s| s.as_str()).unwrap_or("");
            let tool_params = params.get("arguments").cloned().unwrap_or(Value::Null);

            let result_text = tools::dispatch_tool_call(name, &tool_params, rt)
                .unwrap_or_else(|| format!("Unknown tool: '{}'", name));

            json!({
                "content": [
                    {
                        "type": "text",
                        "text": result_text
                    }
                ]
            })
        }
        _ => {
            json!({"error": json!({"code": -32601, "message": "Method not found"})})
        }
    }
}
