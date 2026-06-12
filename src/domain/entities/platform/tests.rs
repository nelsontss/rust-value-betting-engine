use strum::IntoEnumIterator;

use crate::domain::entities::Platform;

#[test]
fn platform_variants_are_distinct() {
    assert_ne!(Platform::Betano, Platform::LeBull);
    assert_ne!(Platform::Betano, Platform::Bwin);
    assert_ne!(Platform::LeBull, Platform::Bwin);
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
    let bwin = serde_json::to_string(&Platform::Bwin).unwrap();
    assert_eq!(betano, "\"betano\"");
    assert_eq!(lebull, "\"lebull\"");
    assert_eq!(bwin, "\"bwin\"");
}

#[test]
fn platform_deserializes_from_lowercase() {
    let betano: Platform = serde_json::from_str("\"betano\"").unwrap();
    let lebull: Platform = serde_json::from_str("\"lebull\"").unwrap();
    let bwin: Platform = serde_json::from_str("\"bwin\"").unwrap();
    assert_eq!(betano, Platform::Betano);
    assert_eq!(lebull, Platform::LeBull);
    assert_eq!(bwin, Platform::Bwin);
}

#[test]
fn platform_iter_returns_all_variants() {
    let variants: Vec<Platform> = Platform::iter().collect();
    assert_eq!(variants.len(), 3);
    assert!(variants.contains(&Platform::Betano));
    assert!(variants.contains(&Platform::LeBull));
    assert!(variants.contains(&Platform::Bwin));
}
