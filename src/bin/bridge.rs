use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use rust_value_betting_engine::infrastructure::config::BridgeConfig;

fn log_msg(msg: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let line = format!("[{}] {}\n", ts, msg);

    let path = BridgeConfig::LOG_PATH;

    if fs::metadata(path).map_or(false, |m| m.len() > 0) {
        if let Ok(content) = fs::read_to_string(path) {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() >= BridgeConfig::MAX_LOG_LINES {
                let keep: String = lines
                    .iter()
                    .skip(lines.len().saturating_sub(BridgeConfig::TRIM_LOG_TO))
                    .chain(std::iter::once(&&line[..]))
                    .map(|l| format!("{}\n", l))
                    .collect();
                let _ = fs::write(path, keep);
                return;
            }
        }
    }

    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn read_message() -> io::Result<Option<String>> {
    let mut len_buf = [0u8; 4];
    if io::stdin().read_exact(&mut len_buf).is_err() {
        return Ok(None);
    }
    let len = u32::from_le_bytes(len_buf) as usize;
    log_msg(&format!("reading {} bytes", len));
    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
}

fn send_response(payload: &str) {
    let bytes = payload.as_bytes();
    let len = (bytes.len() as u32).to_le_bytes();
    let mut out = io::stdout().lock();
    let _ = out.write_all(&len);
    let _ = out.write_all(bytes);
    let _ = out.flush();
}

fn send_to_stream(stream: &mut UnixStream, msg: &str) -> bool {
    let bytes = msg.as_bytes();
    let len = (bytes.len() as u32).to_le_bytes();
    stream.write_all(&len).is_ok() && stream.write_all(bytes).is_ok() && stream.flush().is_ok()
}

fn main() {
    let _ = fs::remove_file(BridgeConfig::SOCKET_PATH);
    let listener = UnixListener::bind(BridgeConfig::SOCKET_PATH).expect("Cannot create socket");
    log_msg("bridge started");

    let streams: Arc<Mutex<Vec<UnixStream>>> = Arc::new(Mutex::new(Vec::new()));

    let streams_clone = streams.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    log_msg("new client connected");
                    streams_clone.lock().unwrap().push(stream);
                }
                Err(e) => log_msg(&format!("accept error: {}", e)),
            }
        }
    });

    loop {
        match read_message() {
            Ok(Some(msg)) => {
                log_msg(&format!(
                    "received {} bytes: {}",
                    msg.len(),
                    &msg[..msg.len().min(500)]
                ));
                send_response(&format!(
                    "{{\"status\":\"ok\",\"received_bytes\":{}}}",
                    msg.len()
                ));

                let mut streams = streams.lock().unwrap();
                streams.retain(|s| send_to_stream(&mut s.try_clone().unwrap(), &msg));
            }
            Ok(None) => {
                log_msg("stdin closed, exiting");
                break;
            }
            Err(e) => {
                log_msg(&format!("read error: {e}"));
                send_response(&format!("{{\"status\":\"error\",\"message\":\"{e}\"}}"));
                break;
            }
        }
    }
    log_msg("bridge exiting");
}
