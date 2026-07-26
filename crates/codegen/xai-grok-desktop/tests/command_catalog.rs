use xai_grok_desktop::domain::command_catalog::{CommandKind, DesktopCommand};

#[test]
fn rename_becomes_a_form() {
    let item = DesktopCommand::from_slash("/rename", true, true);

    assert_eq!(item.kind, CommandKind::Form);
}

#[test]
fn quit_requires_confirmation() {
    assert!(DesktopCommand::from_slash("/quit", false, false).requires_confirmation);
}

#[test]
fn dynamic_commands_never_spawn_processes() {
    let item = DesktopCommand::from_slash("/plugin-action", true, false);

    assert_eq!(item.kind, CommandKind::PromptDispatch);
    assert!(!item.can_spawn_process);
}
