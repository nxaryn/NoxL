//! Tests for notification history persistence and unread tracking.

use noxl_notifications::history::*;

#[test]
fn add_assigns_incrementing_ids() {
    let mut h = History::default();
    let id1 = h.add("app1".into(), "icon".into(), "Summary 1".into(), "Body".into());
    let id2 = h.add("app2".into(), "icon".into(), "Summary 2".into(), "Body".into());
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
    assert_eq!(h.notifications.len(), 2);
}

#[test]
fn unread_tracks_read_state() {
    let mut h = History::default();
    let id = h.add("app".into(), "icon".into(), "S".into(), "B".into());
    assert_eq!(h.unread().len(), 1);

    h.mark_read(id);
    assert_eq!(h.unread().len(), 0);
}

#[test]
fn remove_deletes_notification() {
    let mut h = History::default();
    let id = h.add("app".into(), "icon".into(), "S".into(), "B".into());
    assert_eq!(h.notifications.len(), 1);
    h.remove(id);
    assert_eq!(h.notifications.len(), 0);
}

#[test]
fn mark_read_unknown_id_is_noop() {
    let mut h = History::default();
    h.mark_read(999); // should not panic
    assert_eq!(h.notifications.len(), 0);
}

#[test]
fn remove_unknown_id_is_noop() {
    let mut h = History::default();
    h.remove(999); // should not panic
}

#[test]
fn history_json_roundtrip() {
    let mut h = History::default();
    h.add("app".into(), "icon".into(), "Summary".into(), "Body".into());
    h.next_id = 5;

    let json = serde_json::to_string(&h).unwrap();
    let parsed: History = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.notifications.len(), 1);
    assert_eq!(parsed.next_id, 5);
    assert_eq!(parsed.notifications[0].summary, "Summary");
}

#[test]
fn timestamps_are_set() {
    let mut h = History::default();
    h.add("app".into(), "icon".into(), "S".into(), "B".into());
    // Timestamp should be a plausible Unix time (after 2020-01-01).
    assert!(h.notifications[0].timestamp > 1_577_836_800);
}
