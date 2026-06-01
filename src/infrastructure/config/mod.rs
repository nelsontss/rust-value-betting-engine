pub struct BridgeConfig;

impl BridgeConfig {
    pub const SOCKET_PATH: &str = "/tmp/odds-bridge.sock";
    pub const LOG_PATH: &str = "/Users/nelsonsousa/rust-value-betting-engine/bridge.log";
    pub const MAX_LOG_LINES: usize = 1000;
    pub const TRIM_LOG_TO: usize = 500;
}
