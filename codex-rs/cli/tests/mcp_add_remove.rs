use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use codex_core::config::load_global_mcp_servers;
use predicates::str::contains;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::cargo_bin("codex")?;
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

#[test]
fn add_and_remove_server_updates_global_config() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut add_cmd = codex_command(codex_home.path())?;
    add_cmd
        .args(["mcp", "add", "docs", "--", "echo", "hello"])
        .assert()
        .success()
        .stdout(contains("Added global MCP server 'docs'."));

    let servers = load_global_mcp_servers(codex_home.path())?;
    assert_eq!(servers.len(), 1);
    let docs = servers.get("docs").expect("server should exist");
    assert!(docs.preset.is_none());
    assert_eq!(docs.command, "echo");
    assert_eq!(docs.args, vec!["hello".to_string()]);
    assert!(docs.env.is_none());

    let mut remove_cmd = codex_command(codex_home.path())?;
    remove_cmd
        .args(["mcp", "remove", "docs"])
        .assert()
        .success()
        .stdout(contains("Removed global MCP server 'docs'."));

    let servers = load_global_mcp_servers(codex_home.path())?;
    assert!(servers.is_empty());

    let mut remove_again_cmd = codex_command(codex_home.path())?;
    remove_again_cmd
        .args(["mcp", "remove", "docs"])
        .assert()
        .success()
        .stdout(contains("No MCP server named 'docs' found."));

    let servers = load_global_mcp_servers(codex_home.path())?;
    assert!(servers.is_empty());

    Ok(())
}

#[test]
fn add_with_env_preserves_key_order_and_values() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut add_cmd = codex_command(codex_home.path())?;
    add_cmd
        .args([
            "mcp",
            "add",
            "envy",
            "--env",
            "FOO=bar",
            "--env",
            "ALPHA=beta",
            "--",
            "python",
            "server.py",
        ])
        .assert()
        .success();

    let servers = load_global_mcp_servers(codex_home.path())?;
    let envy = servers.get("envy").expect("server should exist");
    let env = envy.env.as_ref().expect("env should be present");

    assert_eq!(env.len(), 2);
    assert_eq!(env.get("FOO"), Some(&"bar".to_string()));
    assert_eq!(env.get("ALPHA"), Some(&"beta".to_string()));

    Ok(())
}

#[test]
fn add_with_preset_uses_builtin_defaults() -> Result<()> {
    let codex_home = TempDir::new()?;

    let mut add_cmd = codex_command(codex_home.path())?;
    add_cmd
        .args(["mcp", "add", "chrome", "--preset", "chrome_devtools"])
        .assert()
        .success();

    let servers = load_global_mcp_servers(codex_home.path())?;
    let chrome = servers.get("chrome").expect("server should exist");
    assert_eq!(chrome.preset.as_deref(), Some("chrome_devtools"));
    assert_eq!(chrome.command, "npx");
    assert_eq!(
        chrome.args,
        vec![
            "chrome-devtools-mcp@latest".to_string(),
            "--stdio".to_string()
        ]
    );
    assert_eq!(chrome.startup_timeout_sec, Some(Duration::from_secs(45)));
    assert_eq!(chrome.tool_timeout_sec, Some(Duration::from_secs(120)));

    Ok(())
}
