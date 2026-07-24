use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionMode {
    Ask,
    Auto,
    AlwaysApprove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    pub cwd: PathBuf,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub permission_mode: PermissionMode,
}

impl LaunchRequest {
    pub fn new(
        cwd: impl Into<PathBuf>,
        model: Option<&str>,
        effort: Option<&str>,
        permission_mode: PermissionMode,
    ) -> Self {
        Self {
            cwd: cwd.into(),
            model: model.map(str::to_owned),
            effort: effort.map(str::to_owned),
            permission_mode,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaunchSpec {
    pub program: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<String>,
}
