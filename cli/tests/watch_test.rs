//! Tests for the watch command functionality

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_watch_detects_yaml_changes() {
        // Create temporary directory with game files
        let temp_dir = TempDir::new().unwrap();
        let game_dir = temp_dir.path().join("test-game").join("1.0");
        fs::create_dir_all(&game_dir).unwrap();

        // Create initial YAML files
        let manifest_path = game_dir.join("manifest.yaml");
        fs::write(&manifest_path, r#"
gameId: test-game
version: "1.0"
specVersion: 1
metadata:
  name: "Test Game"
  players:
    min: 2
    max: 2
"#).unwrap();

        let actions_path = game_dir.join("actions.yaml");
        fs::write(&actions_path, r#"
- id: testAction
  uses: setState
  with:
    path: "/game/test"
    value: "initial"
"#).unwrap();

        // TODO: Implement actual watch testing once the watch command
        // is integrated with the server for live reload functionality
        
        // For now, verify files exist
        assert!(manifest_path.exists());
        assert!(actions_path.exists());
    }

    #[test]
    fn test_should_rebuild_file_filters() {
        use crate::watch::should_rebuild;
        use notify::{Event, EventKind};

        // Test YAML file changes
        let yaml_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("test.yaml")],
            attrs: Default::default(),
        };
        assert!(should_rebuild(&yaml_event));

        // Test YML file changes  
        let yml_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("test.yml")],
            attrs: Default::default(),
        };
        assert!(should_rebuild(&yml_event));

        // Test non-relevant file changes
        let txt_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("readme.txt")],
            attrs: Default::default(),
        };
        assert!(!should_rebuild(&txt_event));

        // Test directory changes (should not trigger)
        let dir_event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from("some_dir/")],
            attrs: Default::default(),
        };
        assert!(!should_rebuild(&dir_event));
    }

    #[test]
    fn test_watch_command_args() {
        use clap::Parser;
        use crate::cli::{Cli, Commands};

        // Test default arguments
        let cli = Cli::parse_from(&["bluefelt-cli", "watch"]);
        if let Commands::Watch { path, serve, open, port } = cli.command {
            assert_eq!(path, PathBuf::from("."));
            assert!(!serve);
            assert!(!open);
            assert_eq!(port, 8080);
        } else {
            panic!("Expected Watch command");
        }

        // Test with arguments
        let cli = Cli::parse_from(&[
            "bluefelt-cli", 
            "watch", 
            "games/my-game",
            "--serve",
            "--open",
            "--port", "3000"
        ]);
        if let Commands::Watch { path, serve, open, port } = cli.command {
            assert_eq!(path, PathBuf::from("games/my-game"));
            assert!(serve);
            assert!(open);
            assert_eq!(port, 3000);
        } else {
            panic!("Expected Watch command");
        }
    }
}