//! Unit tests for the NoxL IPC protocol.
//!
//! These run without a Wayland session (`cargo test -p noxl-ipc`).

#[cfg(test)]
mod tests {
    use noxl_ipc::*;

    #[test]
    fn frame_encode_decode_roundtrip() {
        let cmd = Command::Ping;
        let frame = Frame::new(serde_json::to_value(&cmd).unwrap());
        let encoded = frame.encode();

        assert!(encoded.ends_with('\n'));
        assert_eq!(encoded.matches('\n').count(), 1);

        let decoded = Frame::decode(encoded.trim()).unwrap();
        assert_eq!(decoded.version, PROTOCOL_VERSION);
        let cmd2: Command = serde_json::from_value(decoded.payload).unwrap();
        assert!(matches!(cmd2, Command::Ping));
    }

    #[test]
    fn spawn_command_roundtrip() {
        let cmd = Command::Spawn { app: "firefox.desktop".into() };
        let frame = Frame::new(serde_json::to_value(&cmd).unwrap());
        let decoded = Frame::decode(frame.encode().trim()).unwrap();
        let cmd2: Command = serde_json::from_value(decoded.payload).unwrap();
        match cmd2 {
            Command::Spawn { app } => assert_eq!(app, "firefox.desktop"),
            _ => panic!("wrong command type"),
        }
    }

    #[test]
    fn workspace_commands_roundtrip() {
        for cmd in [
            Command::WorkspaceNext,
            Command::WorkspacePrev,
            Command::WorkspaceFocus { index: 2 },
        ] {
            let frame = Frame::new(serde_json::to_value(&cmd).unwrap());
            let decoded = Frame::decode(frame.encode().trim()).unwrap();
            let _: Command = serde_json::from_value(decoded.payload).unwrap();
        }
    }

    #[test]
    fn event_roundtrip() {
        let event = Event::WorkspaceChanged { index: 3 };
        let frame = Frame::new(serde_json::to_value(&event).unwrap());
        let decoded = Frame::decode(frame.encode().trim()).unwrap();
        let event2: Event = serde_json::from_value(decoded.payload).unwrap();
        match event2 {
            Event::WorkspaceChanged { index } => assert_eq!(index, 3),
            _ => panic!("wrong event type"),
        }
    }

    #[test]
    fn notify_command_roundtrip() {
        let cmd = Command::Notify {
            summary: "Test".into(),
            body: "Body text".into(),
            app_icon: Some("firefox".into()),
        };
        let frame = Frame::new(serde_json::to_value(&cmd).unwrap());
        let decoded = Frame::decode(frame.encode().trim()).unwrap();
        let cmd2: Command = serde_json::from_value(decoded.payload).unwrap();
        match cmd2 {
            Command::Notify { summary, body, app_icon } => {
                assert_eq!(summary, "Test");
                assert_eq!(body, "Body text");
                assert_eq!(app_icon.as_deref(), Some("firefox"));
            }
            _ => panic!("wrong command type"),
        }
    }

    #[test]
    fn control_socket_path_is_under_runtime_dir() {
        let path = control_socket_path();
        assert!(path.to_string_lossy().contains("noxl"));
        assert!(path.to_string_lossy().ends_with("control.sock"));
    }

    #[test]
    fn invalid_json_is_rejected() {
        assert!(Frame::decode("not json").is_err());
        assert!(Frame::decode("").is_err());
    }

    #[test]
    fn protocol_version_is_set() {
        assert!(PROTOCOL_VERSION >= 1);
    }
}
