//! `grok models` subcommand.

use anyhow::{Context, Result};
use tokio_util::sync::CancellationToken;
use xai_grok_shell::agent::config::Config as AgentConfig;
use xai_grok_shell::cli_models::{AuthStatus, list_models};

use crate::client_identity::{PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION};

pub async fn list_available_models(agent_config: &AgentConfig) -> Result<()> {
    match AuthStatus::resolve(agent_config) {
        AuthStatus::ApiKey => println!("You are using XAI_API_KEY."),
        AuthStatus::LoggedIn(host) => println!("You are logged in with {}.", host),
        AuthStatus::ModelCredentials(model) => {
            println!("Model '{model}' is using its own API key.");
        }
        AuthStatus::DeploymentKey => println!("You are authenticated via deployment key."),
        AuthStatus::NotAuthenticated => println!("You are not authenticated."),
    }
    println!();

    let cancel = CancellationToken::new();
    let spawned = crate::acp::spawn::spawn_grok_shell(agent_config.clone(), &cancel, None).await?;

    let state = list_models(&spawned.channel.tx, PAGER_CLIENT_TYPE, PAGER_CLIENT_VERSION).await?;

    println!("Default model: {}", state.current_model_id.0);
    println!();
    println!("Available models:");
    for m in state.available_models {
        if m.model_id == state.current_model_id {
            println!("  * {} (default)", m.model_id.0);
        } else {
            println!("  - {}", m.model_id.0);
        }
    }

    cancel.cancel();
    Ok(())
}

/// Add or update a `[model.<name>]` entry in `~/.grok/config.toml`.
pub async fn add_model(
    name: String,
    model: String,
    base_url: String,
    api_key: Option<String>,
    env_key: Option<String>,
    display_name: Option<String>,
    context_window: u64,
    api_backend: Option<String>,
    auth_scheme: Option<String>,
    extra_headers: Vec<String>,
) -> Result<()> {
    let path = xai_grok_shell::util::config::user_config_path();

    // Read existing config, preserving everything we don't touch.
    let mut root: toml::Value = match tokio::fs::read_to_string(&path).await {
        Ok(s) => toml::from_str(&s).with_context(|| {
            format!(
                "failed to parse existing config at {}; fix the syntax error and retry",
                path.display()
            )
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            toml::Value::Table(toml::map::Map::new())
        }
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };

    let table = root.as_table_mut().with_context(|| "config root is not a table")?;

    // Build a clean [model.<name>] table, writing only the fields the user supplied.
    let mut model_table = toml::map::Map::new();
    model_table.insert("model".to_string(), toml::Value::String(model));
    model_table.insert("base_url".to_string(), toml::Value::String(base_url));
    model_table.insert(
        "context_window".to_string(),
        toml::Value::Integer(context_window as i64),
    );
    if let Some(display_name) = display_name {
        model_table.insert("name".to_string(), toml::Value::String(display_name));
    }
    if let Some(api_key) = api_key {
        model_table.insert("api_key".to_string(), toml::Value::String(api_key));
    }
    if let Some(env_key) = env_key {
        model_table.insert("env_key".to_string(), toml::Value::String(env_key));
    }
    if let Some(api_backend) = api_backend {
        model_table.insert("api_backend".to_string(), toml::Value::String(api_backend));
    }
    if let Some(auth_scheme) = auth_scheme {
        model_table.insert("auth_scheme".to_string(), toml::Value::String(auth_scheme));
    }
    if !extra_headers.is_empty() {
        let mut headers = toml::map::Map::new();
        for h in extra_headers {
            let (k, v) = parse_header(&h).with_context(|| format!("invalid header format: {h}"))?;
            headers.insert(k, toml::Value::String(v));
        }
        model_table.insert("extra_headers".to_string(), toml::Value::Table(headers));
    }

    let model_section = table
        .entry("model".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let model_section = model_section
        .as_table_mut()
        .with_context(|| "[model] section in config is not a table")?;
    model_section.insert(name.clone(), toml::Value::Table(model_table));

    let toml_str = toml::to_string_pretty(&root).context("failed to serialize config")?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    // Atomic write via temp file + rename.
    let suffix = format!(
        "toml.tmp.{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let tmp = path.with_extension(suffix);
    tokio::fs::write(&tmp, toml_str)
        .await
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("failed to rename {} to {}", tmp.display(), path.display()))?;

    println!("Added model '{}' to {}.", name, path.display());
    println!();
    println!("You can now use it with:");
    println!("  grok -m {name} \"<prompt>\"");
    println!("or set it as the default with:");
    println!("  grok models default {name}");

    Ok(())
}

/// Parse a header string in "Key: Value" or "Key=Value" form.
fn parse_header(s: &str) -> Result<(String, String)> {
    let s = s.trim();
    let split_at = s
        .find(':')
        .or_else(|| s.find('='))
        .ok_or_else(|| anyhow::anyhow!("header must be in 'Key: Value' or 'Key=Value' form"))?;
    let (k, v) = s.split_at(split_at);
    let k = k.trim();
    let v = v[1..].trim_start_matches(|c| c == ':' || c == '=').trim();
    if k.is_empty() || v.is_empty() {
        anyhow::bail!("header key and value must be non-empty");
    }
    Ok((k.to_string(), v.to_string()))
}

/// Persist the default model to `[models].default` in `~/.grok/config.toml`.
pub async fn set_default_model(model: String) -> Result<()> {
    let path = xai_grok_shell::util::config::user_config_path();

    // Read existing config, preserving everything we don't touch.
    let mut root: toml::Value = match tokio::fs::read_to_string(&path).await {
        Ok(s) => toml::from_str(&s).with_context(|| {
            format!(
                "failed to parse existing config at {}; fix the syntax error and retry",
                path.display()
            )
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            toml::Value::Table(toml::map::Map::new())
        }
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };

    let table = root.as_table_mut().with_context(|| "config root is not a table")?;
    let models_section = table
        .entry("models".to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let models_section = models_section
        .as_table_mut()
        .with_context(|| "[models] section in config is not a table")?;
    models_section.insert("default".to_string(), toml::Value::String(model.clone()));

    let toml_str = toml::to_string_pretty(&root).context("failed to serialize config")?;

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let suffix = format!(
        "toml.tmp.{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let tmp = path.with_extension(suffix);
    tokio::fs::write(&tmp, toml_str)
        .await
        .with_context(|| format!("failed to write {}", tmp.display()))?;
    tokio::fs::rename(&tmp, &path)
        .await
        .with_context(|| format!("failed to rename {} to {}", tmp.display(), path.display()))?;

    println!("Set default model to '{}'.", model);
    println!();
    println!("New sessions will use this model unless you pass -m/--model.");
    Ok(())
}
