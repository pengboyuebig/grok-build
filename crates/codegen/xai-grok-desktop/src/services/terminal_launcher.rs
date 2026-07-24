use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::domain::terminal_launch::{LaunchRequest, PermissionMode, TerminalLaunchSpec};

pub fn build_launch_spec(request: LaunchRequest) -> Result<TerminalLaunchSpec> {
    let cwd = dunce::canonicalize(&request.cwd)
        .with_context(|| format!("工作目录不存在或无法访问：{}", request.cwd.display()))?;
    if !cwd.is_dir() {
        bail!("工作目录不是文件夹：{}", cwd.display());
    }

    let program = sibling_terminal_binary()?;
    let mut args = vec!["--cwd".to_owned(), cwd.to_string_lossy().into_owned()];

    if let Some(model) = request.model.filter(|value| !value.trim().is_empty()) {
        args.extend(["--model".to_owned(), model]);
    }
    if let Some(effort) = request.effort.filter(|value| !value.trim().is_empty()) {
        args.extend(["--reasoning-effort".to_owned(), effort]);
    }
    if matches!(request.permission_mode, PermissionMode::AlwaysApprove) {
        args.push("--always-approve".to_owned());
    }

    Ok(TerminalLaunchSpec { program, cwd, args })
}

pub fn launch_terminal_session(spec: &TerminalLaunchSpec) -> Result<()> {
    std::process::Command::new(&spec.program)
        .args(&spec.args)
        .spawn()
        .with_context(|| format!("无法启动终端会话：{}", spec.program.display()))?;
    Ok(())
}

fn sibling_terminal_binary() -> Result<PathBuf> {
    let desktop_exe = std::env::current_exe().context("无法定位桌面程序路径")?;
    let directory = desktop_exe.parent().context("桌面程序路径不包含目录")?;
    let extension = if cfg!(windows) { "exe" } else { "" };
    let name = if extension.is_empty() {
        "xai-grok-pager"
    } else {
        "xai-grok-pager.exe"
    };
    Ok(directory.join(name))
}
