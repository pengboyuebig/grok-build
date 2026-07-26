use xai_grok_desktop::commands::desktop_invoke_schema;

#[test]
fn ipc_schema_excludes_raw_process_inputs() {
    let schema = desktop_invoke_schema();

    assert!(!schema.iter().any(|name| *name == "rawArgs"));
    assert!(!schema.iter().any(|name| *name == "program"));
}

#[test]
fn ipc_schema_lists_only_approved_operations() {
    let schema = desktop_invoke_schema();

    assert_eq!(
        schema.as_slice(),
        &[
            "get_command_catalog",
            "start_session",
            "send_message",
            "respond_to_approval",
            "launch_terminal_session",
            "list_sessions",
            "open_workspace_dialog",
            "set_preference",
        ]
    );
}
