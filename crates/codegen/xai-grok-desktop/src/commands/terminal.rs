use anyhow::Result;

use crate::domain::terminal_launch::LaunchRequest;
use crate::services::terminal_launcher::{
    build_launch_spec, launch_terminal_session as spawn_terminal_session,
};

pub fn launch_terminal_session(request: LaunchRequest) -> Result<()> {
    let spec = build_launch_spec(request)?;
    spawn_terminal_session(&spec)
}
