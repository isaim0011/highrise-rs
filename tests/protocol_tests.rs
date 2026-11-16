use highrise_rs::error::is_fatal_server_error;
use highrise_rs::protocol::{WsEvent, WsRequest};
use highrise_rs::types::{EntityPosition, Facing, GoldBar, Reaction, RoomPermissions};

#[test]
fn parses_session_metadata_with_null_sdk_version() {
    // Exact frame sent by the live Highrise server (sdk_version is null).
    let raw = r#"{"_type":"SessionMetadata","connection_id":"cd7af825-4032-4844-880a-73bbe1d79130","rate_limits":{"client":[10,0.1],"socials":[20,0.2]},"room_info":{"owner_id":"6a86225f4f0a10f17516ff82","room_name":"HR_COLLEGE"},"sdk_version":null,"user_id":"6af889e574eebd877faa5f3c"}"#;
    let evt: WsEvent = serde_json::from_str(raw).expect("null sdk_version must not drop SessionMetadata");
    match evt {
        WsEvent::SessionMetadata { user_id, sdk_version, room_info, .. } => {
            assert_eq!(user_id, "6af889e574eebd877faa5f3c");
            assert_eq!(sdk_version, "");
            assert_eq!(room_info.room_name, "HR_COLLEGE");
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[test]
fn parses_session_metadata_and_chat_events() {
    let raw_meta = r#"{
        "_type": "SessionMetadata",
        "user_id": "6af889e574eebd877faa5f3c",
        "room_info": { "room_name": "Chill", "owner_id": "6a86225f4f0a10f17516ff82" },
        "connection_id": "conn-1",
        "sdk_version": "0.1.0"
    }"#;
    let evt: WsEvent = serde_json::from_str(raw_meta).expect("session metadata should parse");
    match evt {
        WsEvent::SessionMetadata { user_id, room_info, .. } => {
            assert_eq!(user_id, "6af889e574eebd877faa5f3c");
            assert_eq!(room_info.room_name, "Chill");
        }
        other => panic!("unexpected variant: {other:?}"),
    }

    let raw_whisper = r#"{
        "_type": "ChatEvent",
        "user": { "id": "u1", "username": "iBimo_o" },
        "message": "!play test",
        "whisper": true
    }"#;
    let evt: WsEvent = serde_json::from_str(raw_whisper).expect("chat event should parse");
    match evt {
        WsEvent::ChatEvent { user, message, whisper } => {
            assert_eq!(user.username, "iBimo_o");
            assert_eq!(message, "!play test");
            assert!(whisper);
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[test]
fn parses_reaction_and_voice_events_missing_in_js_sdk() {
    let raw_reaction = r#"{
        "_type": "ReactionEvent",
        "user": { "id": "u1", "username": "alice" },
        "reaction": "heart",
        "receiver": { "id": "u2", "username": "bob" }
    }"#;
    let evt: WsEvent = serde_json::from_str(raw_reaction).expect("reaction event should parse");
    match evt {
        WsEvent::ReactionEvent { reaction, receiver, .. } => {
            assert_eq!(reaction, Reaction::Heart);
            assert_eq!(receiver.username, "bob");
        }
        other => panic!("unexpected variant: {other:?}"),
    }

    let raw_voice = r#"{
        "_type": "VoiceEvent",
        "users": [[{ "id": "u1", "username": "alice" }, "speaking"]],
        "seconds_left": 120
    }"#;
    let evt: WsEvent = serde_json::from_str(raw_voice).expect("voice event should parse");
    match evt {
        WsEvent::VoiceEvent { users, seconds_left } => {
            assert_eq!(users.len(), 1);
            assert_eq!(users[0].1, "speaking");
            assert_eq!(seconds_left, 120);
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

#[test]
fn parses_floor_and_anchor_positions() {
    let floor: EntityPosition =
        serde_json::from_str(r#"{"x": 4.0, "y": 0.0, "z": 29.0, "facing": "FrontRight"}"#)
            .expect("floor pos");
    assert!(matches!(floor, EntityPosition::Floor(p) if p.facing == Facing::FrontRight));

    let anchor: EntityPosition =
        serde_json::from_str(r#"{"entity_id": "chair_1", "anchor_ix": 2}"#)
            .expect("anchor pos");
    assert!(matches!(anchor, EntityPosition::Anchor(a) if a.anchor_ix == 2));
}

#[test]
fn serializes_requests_with_type_tag() {
    let req = WsRequest::ChangeRoomPrivilegeRequest {
        user_id: "u1".into(),
        permissions: RoomPermissions {
            moderator: true,
            designer: false,
        },
    };
    let v = serde_json::to_value(&req).expect("serialize request");
    assert_eq!(v["_type"], "ChangeRoomPrivilegeRequest");
    assert_eq!(v["permissions"]["moderator"], true);

    assert_eq!(GoldBar::G1000.as_str(), "gold_bar_1k");
    assert!(is_fatal_server_error("Bots must have designer rights"));
    assert!(!is_fatal_server_error("Request timed out"));
}

#[test]
fn parses_webapi_user_with_literal_nulls() {
    let raw = r#"{
        "user": {
            "user_id": "u1",
            "username": "iBimo_o",
            "bio": null,
            "discord_id": null,
            "outfit": null,
            "active_room": null,
            "crew": null
        }
    }"#;
    let resp: highrise_rs::webapi_models::GetUserResponse =
        serde_json::from_str(raw).expect("null fields should deserialize cleanly");
    assert_eq!(resp.user.username, "iBimo_o");
    assert!(resp.user.outfit.is_empty());
    assert!(resp.user.active_room.is_none());
}
