pub mod knowledge;

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

/// Runs the Highrise Docs & Rust SDK MCP server over stdio (JSON-RPC 2.0).
pub fn run_stdio_server() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("parse error: {e}") }
                });
                writeln!(out, "{err}")?;
                out.flush()?;
                continue;
            }
        };

        if let Some(resp) = handle_request(&req) {
            writeln!(out, "{resp}")?;
            out.flush()?;
        }
    }
    Ok(())
}

/// Processes a single JSON-RPC 2.0 MCP request; returns `None` for notifications.
pub fn handle_request(req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));

    if id.is_none() || method.starts_with("notifications/") {
        return None;
    }

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {}, "resources": {} },
            "serverInfo": { "name": "highrise-mcp", "version": env!("CARGO_PKG_VERSION") }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tool_definitions() })),
        "tools/call" => call_tool(&params),
        "resources/list" => Ok(json!({
            "resources": [
                { "uri": "highrise://colors", "name": "Safe Highrise Hex Palette", "mimeType": "text/plain" },
                { "uri": "highrise://quickstart", "name": "highrise-rs Quickstart", "mimeType": "text/rust" }
            ]
        })),
        "resources/read" => {
            let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
            let text = match uri {
                "highrise://colors" => knowledge::get_guide("colors"),
                "highrise://quickstart" => knowledge::get_guide("quickstart"),
                _ => return Some(json_err(id.unwrap(), -32602, "unknown resource uri")),
            };
            Ok(json!({ "contents": [{ "uri": uri, "mimeType": "text/plain", "text": text }] }))
        }
        _ => return Some(json_err(id.unwrap(), -32601, "method not found")),
    };

    Some(match result {
        Ok(res) => json!({ "jsonrpc": "2.0", "id": id.unwrap(), "result": res }),
        Err(msg) => json_err(id.unwrap(), -32602, &msg),
    })
}

fn json_err(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "search_highrise_docs",
            "description": "Search highrise-rs Rust SDK, official Highrise Bot WebSocket API, Web REST API, and formatting guides.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keywords (e.g. websocket, webapi, hex, 1006, events)" },
                    "section": { "type": "string", "enum": ["bot_api", "web_api", "guides"] }
                },
                "required": ["query"]
            }
        },
        {
            "name": "get_highrise_endpoint",
            "description": "Get wire schema and Rust SDK usage for a Highrise Bot WebSocket or Web API endpoint.",
            "inputSchema": {
                "type": "object",
                "properties": { "name": { "type": "string" } },
                "required": ["name"]
            }
        },
        {
            "name": "format_highrise_text",
            "description": "Format text with safe Highrise <#RRGGBB> color tags, Unicode bold, and <=250-char chunking.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "hex": { "type": "string", "description": "6-digit hex color like 38BDF8, 22C55E, FFC93D, EF4444" },
                    "bold": { "type": "boolean" }
                },
                "required": ["text"]
            }
        },
        {
            "name": "get_rust_sdk_guide",
            "description": "Get copy-pasteable highrise-rs guides: quickstart, webapi, colors, moderation, economy.",
            "inputSchema": {
                "type": "object",
                "properties": { "topic": { "type": "string" } },
                "required": ["topic"]
            }
        }
    ])
}

fn call_tool(params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing tool name".to_string())?;
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    let text = match name {
        "search_highrise_docs" => {
            let query = args.get("query").and_then(Value::as_str).unwrap_or("");
            let section = args.get("section").and_then(Value::as_str);
            knowledge::search_docs(query, section)
        }
        "get_highrise_endpoint" => {
            let ep = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing endpoint name".to_string())?;
            knowledge::get_endpoint(ep)
        }
        "format_highrise_text" => {
            let raw = args
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| "missing text".to_string())?;
            let hex = args.get("hex").and_then(Value::as_str);
            let bold = args.get("bold").and_then(Value::as_bool).unwrap_or(false);
            knowledge::format_highrise_text(raw, hex, bold).to_string()
        }
        "get_rust_sdk_guide" => {
            let topic = args.get("topic").and_then(Value::as_str).unwrap_or("quickstart");
            knowledge::get_guide(topic)
        }
        _ => return Err(format!("unknown tool: {name}")),
    };

    Ok(json!({ "content": [{ "type": "text", "text": text }] }))
}
