use super::*;

#[test]
fn new_creates_service() {
    let service = BookmakerScrapperService::new(Arc::new(RwLock::new(ClusterService::new())));
    let _ = service;
}
