use super::*;

#[test]
fn new_creates_service() {
    let service = BookmakerScrapperService::new();
    let _ = service;
}
