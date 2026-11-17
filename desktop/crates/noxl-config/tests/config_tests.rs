//! Unit tests for the NoxL config model.
//!
//! These run without a Wayland session (`cargo test -p noxl-config`).

#[cfg(test)]
mod tests {
    use noxl_config::*;

    #[test]
    fn defaults_are_obsidian() {
        let config = NoxlConfig::default();
        assert_eq!(config.theme.name, "OBSIDIAN");
        assert_eq!(config.theme.colors.bg_primary, "#09090D");
        assert_eq!(config.theme.colors.accent_primary, "#B44CFF");
        assert_eq!(config.theme.colors.text_primary, "#F2EFF7");
    }

    #[test]
    fn hex_color_parsing() {
        let (r, g, b, a) = Colors::parse_hex("#B44CFF").unwrap();
        assert!((r - 0.706).abs() < 0.01, "r = {}", r);
        assert!((g - 0.298).abs() < 0.01, "g = {}", g);
        assert!((b - 1.0).abs() < 0.01, "b = {}", b);
        assert!((a - 1.0).abs() < 0.01, "a = {}", a);
    }

    #[test]
    fn hex_color_with_alpha() {
        let (_, _, _, a) = Colors::parse_hex("#09090D80").unwrap();
        assert!((a - 0.502).abs() < 0.01, "a = {}", a);
    }

    #[test]
    fn hex_color_invalid() {
        assert!(Colors::parse_hex("not-a-color").is_none());
        assert!(Colors::parse_hex("#FFF").is_none());
    }

    #[test]
    fn config_toml_roundtrip() {
        let config = NoxlConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: NoxlConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.theme.name, config.theme.name);
        assert_eq!(parsed.panel.size, config.panel.size);
        assert_eq!(parsed.workspaces.count, config.workspaces.count);
    }

    #[test]
    fn partial_config_uses_defaults() {
        let toml_str = r#"
            [theme]
            name = "Custom"
        "#;
        let config: NoxlConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.theme.name, "Custom");
        assert_eq!(config.panel.size, 48);
        assert_eq!(config.theme.colors.accent_primary, "#B44CFF");
    }

    #[test]
    fn keybindings_have_defaults() {
        let kb = Keybindings::default();
        assert_eq!(kb.get("launcher.toggle"), Some("Super"));
        assert_eq!(kb.get("terminal.spawn"), Some("Super+Return"));
        assert_eq!(kb.get("window.close"), Some("Super+Q"));
    }

    #[test]
    fn keybinding_set_and_get() {
        let mut kb = Keybindings::default();
        kb.set("custom.action", "Super+X");
        assert_eq!(kb.get("custom.action"), Some("Super+X"));
    }

    #[test]
    fn widgets_upsert_and_remove() {
        let mut widgets = WidgetsConfig::default();
        let w = WidgetInstance {
            widget_type: "system.cpu".into(),
            x: 10,
            y: 20,
            ..Default::default()
        };
        widgets.upsert(w);
        assert_eq!(widgets.widgets.len(), 1);

        let w2 = WidgetInstance {
            widget_type: "system.cpu".into(),
            x: 10,
            y: 20,
            width: 300,
            ..Default::default()
        };
        widgets.upsert(w2);
        assert_eq!(widgets.widgets.len(), 1);
        assert_eq!(widgets.widgets[0].width, 300);

        widgets.remove("system.cpu", 10, 20);
        assert_eq!(widgets.widgets.len(), 0);
    }

    #[test]
    fn wallpaper_default_is_obsidian() {
        match &WallpaperConfig::default() {
            WallpaperConfig::Static { path, .. } => {
                assert!(path.contains("obsidian-default.png"));
            }
            _ => panic!("default wallpaper should be static"),
        }
    }

    #[test]
    fn panel_default_plugins() {
        let panel = PanelConfig::default();
        assert!(panel.plugins.contains(&PanelPlugin::Launcher));
        assert!(panel.plugins.contains(&PanelPlugin::Taskbar));
        assert!(panel.plugins.contains(&PanelPlugin::Clock));
    }
}
