use highrise_rs::mcp::{handle_request, knowledge};
use serde_json::json;

#[test]
fn mcp_initialize_and_tools_list_work() {
    let init = handle_request(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }))
        .expect("initialize responds");
    assert_eq!(init["result"]["serverInfo"]["name"], "highrise-mcp");

    let list = handle_request(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
        .expect("tools/list responds");
    let tools = list["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 4);
}

#[test]
fn mcp_format_highrise_text_applies_safe_hex_bold_and_chunking() {
    let resp = handle_request(&json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "format_highrise_text",
            "arguments": { "text": "Hello", "hex": "#38bdf8", "bold": true }
        }
    }))
    .expect("tools/call responds");

    let body_str = resp["result"]["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(body_str).unwrap();
    assert_eq!(parsed["hex_valid"], true);
    assert!(parsed["formatted"].as_str().unwrap().starts_with("<#38BDF8>"));
    assert_eq!(knowledge::to_highrise_bold("Hi"), "𝗛𝗶");
}
