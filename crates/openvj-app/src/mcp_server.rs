use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::net::TcpStream;

fn main() {
    for line in io::stdin().lock().lines().flatten() {
        let Ok(request) = serde_json::from_str::<Value>(&line) else { continue };
        let method = request.get("method").and_then(Value::as_str).unwrap_or_default();
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        if method == "notifications/initialized" { continue; }
        let result = match method {
            "initialize" => json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"syntetika","version":"0.1.0"}}),
            "tools/list" => json!({"tools": tools()}),
            "tools/call" => call_tool(request.get("params").unwrap_or(&Value::Null)),
            _ => json!({"error":{"code":-32601,"message":"method not found"}}),
        };
        let response = if result.get("error").is_some() { json!({"jsonrpc":"2.0","id":id,"error":result["error"]}) } else { json!({"jsonrpc":"2.0","id":id,"result":result}) };
        println!("{}", response);
        let _ = io::stdout().flush();
    }
}

fn tools() -> Vec<Value> {
    vec![
        json!({"name":"status","description":"Get Syntetika status and selected clip.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"performance","description":"Get FPS and render telemetry.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"trigger_clip","description":"Trigger a clip using 1-based layer and column.","inputSchema":{"type":"object","properties":{"layer":{"type":"integer"},"column":{"type":"integer"}},"required":["layer","column"]}}),
        json!({"name":"select_clip","description":"Select a clip using 1-based layer and column.","inputSchema":{"type":"object","properties":{"layer":{"type":"integer"},"column":{"type":"integer"}},"required":["layer","column"]}}),
        json!({"name":"add_midi_note","description":"Add a MIDI note to the selected clip. Start is in beats.","inputSchema":{"type":"object","properties":{"pitch":{"type":"integer","minimum":0,"maximum":127},"start":{"type":"number","minimum":0}},"required":["pitch","start"]}}),
        json!({"name":"clear_midi_roll","description":"Clear MIDI notes from the selected clip.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"panic_midi","description":"Send All Notes Off to MIDI outputs.","inputSchema":{"type":"object","properties":{}}}),
    ]
}

fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);
    let action = match name {
        "status" => "status",
        "performance" => "performance",
        "trigger_clip" => "trigger_clip",
        "select_clip" => "select_clip",
        "add_midi_note" => "add_midi_note",
        "clear_midi_roll" => "clear_midi_roll",
        "panic_midi" => "panic_midi",
        _ => return json!({"error":{"code":-32602,"message":"unknown tool"}}),
    };
    match post(json!({"action":action,"args":args})) {
        Ok(value) => json!({"content":[{"type":"text","text":value.to_string()}]}),
        Err(error) => json!({"isError":true,"content":[{"type":"text","text":error}]}),
    }
}

fn post(body: Value) -> Result<Value, String> {
    let mut stream = TcpStream::connect("127.0.0.1:47824").map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;
    let header = format!("POST /control HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
    use std::io::{Read, Write};
    stream.write_all(header.as_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&bytes).map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| e.to_string())?;
    let start = response.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4).ok_or("invalid HTTP response")?;
    let body = &response[start..];
    serde_json::from_slice(&body).map_err(|e| e.to_string())
}
