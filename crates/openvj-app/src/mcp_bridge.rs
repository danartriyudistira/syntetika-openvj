use serde_json::Value;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ControlRequest {
    pub action: String,
    pub args: Value,
    pub reply: Sender<Value>,
}

pub type ControlQueue = Arc<Mutex<VecDeque<ControlRequest>>>;

pub fn new_queue() -> ControlQueue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn start(queue: ControlQueue) {
    thread::Builder::new()
        .name("syntetika-control-bridge".into())
        .spawn(move || {
            let Ok(listener) = TcpListener::bind("127.0.0.1:47824") else {
                eprintln!("Syntetika control bridge: port 47824 is unavailable");
                return;
            };
            for stream in listener.incoming().flatten() {
                handle(stream, &queue);
            }
        })
        .expect("Syntetika control bridge thread");
}

fn handle(mut stream: TcpStream, queue: &ControlQueue) {
    let Ok(body) = read_body(&mut stream) else { return };
    let request: Value = serde_json::from_slice(&body).unwrap_or_else(|e| serde_json::json!({"error": e.to_string()}));
    let action = request.get("action").and_then(Value::as_str).unwrap_or("status").to_owned();
    let args = request.get("args").cloned().unwrap_or(Value::Null);
    let (tx, rx) = mpsc::channel();
    if let Ok(mut pending) = queue.lock() {
        pending.push_back(ControlRequest { action, args, reply: tx });
    }
    let response = rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap_or_else(|_| serde_json::json!({"error":"application did not respond"}));
    let bytes = serde_json::to_vec(&response).unwrap_or_else(|_| b"{\"error\":\"serialization failed\"}".to_vec());
    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&bytes);
}

fn read_body(stream: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buf = [0u8; 4096];
    let mut header_end = None;
    let mut length = 0usize;
    while header_end.is_none() && bytes.len() < 64 * 1024 {
        let n = stream.read(&mut buf)?;
        if n == 0 { break; }
        bytes.extend_from_slice(&buf[..n]);
        if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = Some(end + 4);
            let headers = String::from_utf8_lossy(&bytes[..end]);
            for line in headers.lines() {
                if let Some(value) = line.strip_prefix("Content-Length:").or_else(|| line.strip_prefix("content-length:")) {
                    length = value.trim().parse().unwrap_or(0);
                }
            }
        }
    }
    let start = header_end.unwrap_or(bytes.len());
    while bytes.len() < start + length {
        let n = stream.read(&mut buf)?;
        if n == 0 { break; }
        bytes.extend_from_slice(&buf[..n]);
    }
    Ok(bytes[start..std::cmp::min(bytes.len(), start + length)].to_vec())
}
