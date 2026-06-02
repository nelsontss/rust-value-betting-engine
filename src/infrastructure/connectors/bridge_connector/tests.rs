use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;

use crate::application::services::bookmaker_scrapper_service::BookmakerEvent;
use crate::infrastructure::connectors::bridge_connector::BridgeConnector;

fn setup_socket(
    path: &str,
) -> (
    UnixListener,
    mpsc::Receiver<BookmakerEvent>,
    thread::JoinHandle<()>,
) {
    let _ = std::fs::remove_file(path);
    let listener = UnixListener::bind(path).unwrap();
    let connector = BridgeConnector::new();
    let (tx, rx) = mpsc::channel();
    let path_owned = path.to_string();
    let handle = thread::spawn(move || {
        let _ = connector.start_at(tx, &path_owned);
    });
    (listener, rx, handle)
}

fn send_message(stream: &mut std::os::unix::net::UnixStream, payload: &str) {
    let len = payload.len() as u32;
    let mut buf = len.to_le_bytes().to_vec();
    buf.extend_from_slice(payload.as_bytes());
    stream.write_all(&buf).unwrap();
}

fn cleanup(path: &str, listener: UnixListener, handle: thread::JoinHandle<()>) {
    drop(listener);
    let _ = handle.join();
    let _ = std::fs::remove_file(path);
}

fn send_raw(stream: &mut std::os::unix::net::UnixStream, bytes: &[u8]) {
    stream.write_all(bytes).unwrap();
}

#[test]
fn new_creates_bridge_connector() {
    let connector = BridgeConnector::new();
    let _ = connector;
}

#[test]
fn start_returns_gracefully_when_no_socket() {
    let connector = BridgeConnector::new();
    let (tx, _rx) = mpsc::channel();
    let _ = connector.start_at(tx, "/tmp/nonexistent-test-socket.sock");
}

#[test]
fn start_reads_and_forwards_events_from_socket() {
    let path = "/tmp/test-bridge-connector.sock";
    let (listener, rx, handle) = setup_socket(path);
    let (mut stream, _) = listener.accept().unwrap();
    send_message(
        &mut stream,
        r#"{"type":"odds_update","platform":"betano","timestamp":1717000000,"data":{}}"#,
    );
    drop(stream);

    let event = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(matches!(event, BookmakerEvent::InsertGames(_)));
    cleanup(path, listener, handle);
}

#[test]
fn start_forwards_multiple_messages() {
    let path = "/tmp/test-bridge-connector-multi.sock";
    let (listener, rx, handle) = setup_socket(path);
    let (mut stream, _) = listener.accept().unwrap();

    for _ in 0..3 {
        let payload =
            r#"{"type":"odds_update","platform":"betano","timestamp":1717000000,"data":{}}"#;
        send_message(&mut stream, payload);
    }
    drop(stream);

    for _ in 0..3 {
        let event = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
        assert!(matches!(event, BookmakerEvent::InsertGames(_)));
    }
    cleanup(path, listener, handle);
}

#[test]
fn start_handles_invalid_json_gracefully() {
    let path = "/tmp/test-bridge-connector-invalid.sock";
    let (listener, rx, handle) = setup_socket(path);
    let (mut stream, _) = listener.accept().unwrap();
    send_message(&mut stream, "not valid json");
    drop(stream);

    let result = rx.recv_timeout(std::time::Duration::from_millis(500));
    assert!(result.is_err());
    cleanup(path, listener, handle);
}

#[test]
fn start_recovers_after_invalid_message_and_processes_next() {
    let path = "/tmp/test-bridge-connector-recovery.sock";
    let (listener, rx, handle) = setup_socket(path);
    let (mut stream, _) = listener.accept().unwrap();

    send_message(&mut stream, "bad payload");
    send_message(
        &mut stream,
        r#"{"type":"odds_update","platform":"betano","timestamp":100,"data":{}}"#,
    );
    drop(stream);

    let event = rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(matches!(event, BookmakerEvent::InsertGames(_)));
    cleanup(path, listener, handle);
}

#[test]
fn start_exits_when_socket_closes() {
    let path = "/tmp/test-bridge-connector-exit.sock";
    let (listener, _rx, handle) = setup_socket(path);
    let (stream, _) = listener.accept().unwrap();
    drop(stream);

    let result = handle.join();
    assert!(
        result.is_ok(),
        "start should exit cleanly when socket closes"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn start_handles_truncated_body_gracefully() {
    let path = "/tmp/test-bridge-connector-trunc.sock";
    let (listener, _rx, handle) = setup_socket(path);
    let (mut stream, _) = listener.accept().unwrap();
    let mut buf = 100u32.to_le_bytes().to_vec();
    buf.extend_from_slice(b"only 10 bytes");
    send_raw(&mut stream, &buf);
    drop(stream);

    let result = handle.join();
    assert!(result.is_ok(), "start should handle truncated body");
    let _ = std::fs::remove_file(path);
}
