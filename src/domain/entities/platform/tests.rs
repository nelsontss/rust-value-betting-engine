use crate::domain::entities::Platform;

#[test]
fn platform_variants_are_distinct() {
    assert_ne!(Platform::Betano, Platform::LeBull);
}

#[test]
fn platform_copy_and_clone_work() {
    let p = Platform::Betano;
    let q = p;
    assert_eq!(p, q);
}

#[test]
fn platform_serializes_to_lowercase() {
    let betano = serde_json::to_string(&Platform::Betano).unwrap();
    let lebull = serde_json::to_string(&Platform::LeBull).unwrap();
    assert_eq!(betano, "\"betano\"");
    assert_eq!(lebull, "\"lebull\"");
}

#[test]
fn platform_deserializes_from_lowercase() {
    let betano: Platform = serde_json::from_str("\"betano\"").unwrap();
    let lebull: Platform = serde_json::from_str("\"lebull\"").unwrap();
    assert_eq!(betano, Platform::Betano);
    assert_eq!(lebull, Platform::LeBull);
}
