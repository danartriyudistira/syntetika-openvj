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
        json!({"name":"set_tempo","description":"Set composition BPM.","inputSchema":{"type":"object","properties":{"bpm":{"type":"number","minimum":20,"maximum":300}},"required":["bpm"]}}),
        json!({"name":"trigger_clip","description":"Trigger a clip using 1-based layer and column.","inputSchema":{"type":"object","properties":{"layer":{"type":"integer"},"column":{"type":"integer"}},"required":["layer","column"]}}),
        json!({"name":"select_clip","description":"Select a clip using 1-based layer and column.","inputSchema":{"type":"object","properties":{"layer":{"type":"integer"},"column":{"type":"integer"}},"required":["layer","column"]}}),
        json!({"name":"add_midi_note","description":"Add a MIDI note to the selected clip. Start is in beats, velocity 1-127.","inputSchema":{"type":"object","properties":{"pitch":{"type":"integer","minimum":0,"maximum":127},"start":{"type":"number","minimum":0},"velocity":{"type":"integer","minimum":1,"maximum":127}},"required":["pitch","start"]}}),
        json!({"name":"set_midi_roll","description":"Configure MIDI Roll output of the selected clip: device, channel (0-15), enabled.","inputSchema":{"type":"object","properties":{"device":{"type":"string"},"channel":{"type":"integer","minimum":0,"maximum":15},"enabled":{"type":"boolean"}}}}),
        json!({"name":"clear_midi_roll","description":"Clear MIDI notes from the selected clip.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"get_midi_roll","description":"List notes of the selected clip with start/length in beats and selection state.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"marquee_select","description":"Select notes overlapping a beat/pitch rectangle.","inputSchema":{"type":"object","properties":{"beat_lo":{"type":"number"},"beat_hi":{"type":"number"},"pitch_lo":{"type":"integer"},"pitch_hi":{"type":"integer"},"additive":{"type":"boolean"}},"required":["beat_lo","beat_hi","pitch_lo","pitch_hi"]}}),
        json!({"name":"midi_edit","description":"Piano Roll edit ops: select_all, clear_selection, undo, redo, copy, paste, duplicate, velocity, fold, scale.","inputSchema":{"type":"object","properties":{"op":{"type":"string","enum":["select_all","clear_selection","undo","redo","copy","paste","duplicate","velocity","fold","scale"]},"beat":{"type":"number"},"velocity":{"type":"integer","minimum":1,"maximum":127},"root":{"type":"integer","minimum":0,"maximum":11},"kind":{"type":"integer","minimum":0,"maximum":6}},"required":["op"]}}),
        json!({"name":"panic_midi","description":"Send All Notes Off to MIDI outputs.","inputSchema":{"type":"object","properties":{}}}),
        json!({"name":"ui_tab","description":"Open a properties panel tab (0-4; 4 is Piano Roll).","inputSchema":{"type":"object","properties":{"tab":{"type":"integer","minimum":0,"maximum":4}},"required":["tab"]}}),
    ]
}

fn call_tool(params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or(Value::Null);
    let action = match name {
        "status" => "status",
        "performance" => "performance",
        "set_tempo" => "set_tempo",
        "trigger_clip" => "trigger_clip",
        "select_clip" => "select_clip",
        "add_midi_note" => "add_midi_note",
        "set_midi_roll" => "set_midi_roll",
        "clear_midi_roll" => "clear_midi_roll",
        "get_midi_roll" => "get_midi_roll",
        "marquee_select" => "marquee_select",
        "midi_edit" => "midi_edit",
        "panic_midi" => "panic_midi",
        "ui_tab" => "ui_tab",
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
