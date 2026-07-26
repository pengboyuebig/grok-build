use std::path::PathBuf;

use xai_grok_desktop::domain::terminal_launch::{LaunchRequest, PermissionMode};
use xai_grok_desktop::services::terminal_launcher::build_launch_spec;

#[test]
fn creates_argument_vector_not_shell_text() {
    let cwd = std::env::current_dir().unwrap();
    let request = LaunchRequest::new(cwd, Some("grok-build"), Some("high"), PermissionMode::Ask);

    let spec = build_launch_spec(request).unwrap();

    assert_eq!(
        spec.args,
        vec![
            "--cwd",
            spec.cwd.to_string_lossy().as_ref(),
            "--model",
            "grok-build",
            "--reasoning-effort",
            "high",
        ]
    );
    assert_ne!(spec.program, PathBuf::from("cmd.exe"));
}

#[test]
fn invalid_directory_is_rejected() {
    let request = LaunchRequest::new(
        PathBuf::from("Z:/grok-desktop-missing-directory"),
        None,
        None,
        PermissionMode::Ask,
    );

    assert!(build_launch_spec(request).is_err());
}

#[test]
fn always_approve_is_an_explicit_flag() {
    let cwd = std::env::current_dir().unwrap();
    let request = LaunchRequest::new(cwd, None, None, PermissionMode::AlwaysApprove);

    let spec = build_launch_spec(request).unwrap();

    assert_eq!(spec.args.last(), Some(&"--always-approve".to_owned()));
}
