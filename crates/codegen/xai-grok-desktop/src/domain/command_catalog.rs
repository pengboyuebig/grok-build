use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandKind {
    Action,
    Form,
    PromptDispatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandArgument {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesktopCommand {
    pub slash: String,
    pub kind: CommandKind,
    pub requires_confirmation: bool,
    pub can_spawn_process: bool,
    pub arguments: Vec<CommandArgument>,
}

impl DesktopCommand {
    pub fn from_slash(slash: &str, takes_arguments: bool, arguments_required: bool) -> Self {
        let normalized = normalize_slash(slash);
        let is_known = matches!(
            normalized.as_str(),
            "/rename" | "/model" | "/effort" | "/new" | "/clear" | "/quit" | "/exit"
        );
        let kind = if !is_known {
            CommandKind::PromptDispatch
        } else if takes_arguments {
            CommandKind::Form
        } else {
            CommandKind::Action
        };

        Self {
            slash: normalized.clone(),
            kind,
            requires_confirmation: requires_confirmation(&normalized),
            can_spawn_process: false,
            arguments: takes_arguments
                .then(|| CommandArgument {
                    name: argument_name(&normalized).to_owned(),
                    required: arguments_required,
                })
                .into_iter()
                .collect(),
        }
    }
}

fn normalize_slash(slash: &str) -> String {
    let trimmed = slash.trim().to_ascii_lowercase();
    if trimmed.starts_with('/') {
        trimmed
    } else {
        format!("/{trimmed}")
    }
}

fn requires_confirmation(slash: &str) -> bool {
    matches!(
        slash,
        "/quit"
            | "/exit"
            | "/new"
            | "/clear"
            | "/rewind"
            | "/logout"
            | "/memory clear"
            | "/worktree rm"
    )
}

fn argument_name(slash: &str) -> &'static str {
    match slash {
        "/rename" => "session_name",
        "/model" => "model",
        "/effort" => "effort",
        _ => "argument",
    }
}
