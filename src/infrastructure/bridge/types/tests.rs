use serde_json::json;

use super::*;

#[test]
fn bridge_message_serializes_with_tagged_type() {
    let message = BridgeMessage::OddsUpdate {
        platform: Platform::Betano,
        timestamp: 1_234,
        data: json!({"home": 2.0}),
    };

    let value = serde_json::to_value(&message).unwrap();

    assert_eq!("odds_update", value["type"]);
    assert_eq!("betano", value["platform"]);
    assert_eq!(1_234, value["timestamp"]);
    assert_eq!(2.0, value["data"]["home"]);
}

#[test]
fn bridge_message_deserializes_from_tagged_json() {
    let message: BridgeMessage =
        serde_json::from_value(json!({
            "type": "odds_update",
            "platform": "bwin",
            "timestamp": 42,
            "data": {"draw": 3.2}
        }))
        .unwrap();

    match message {
        BridgeMessage::OddsUpdate {
            platform,
            timestamp,
            data,
        } => {
            assert_eq!(Platform::Bwin, platform);
            assert_eq!(42, timestamp);
            assert_eq!(3.2, data["draw"]);
        }
    }
}
