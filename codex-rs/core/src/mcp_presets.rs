use std::collections::HashMap;
use std::time::Duration;

use crate::config_types::McpServerConfig;

/// Built-in definitions for MCP servers that can be referenced via `preset`.
#[derive(Debug, Clone, Copy)]
pub struct McpServerPreset {
    /// Stable identifier used from configuration files / CLI flags.
    pub id: &'static str,
    /// Short human readable label for help text.
    pub label: &'static str,
    /// Single line description explaining what the preset does.
    pub description: &'static str,
    /// Command to launch the MCP server.
    pub command: &'static str,
    /// Arguments appended to the command.
    pub args: &'static [&'static str],
    /// Environment variables exported when launching the server.
    pub env: &'static [(&'static str, &'static str)],
    /// Startup timeout applied while the server initializes and reports tools.
    pub startup_timeout: Option<Duration>,
    /// Default timeout for each tool call dispatched to this server.
    pub tool_timeout: Option<Duration>,
}

impl McpServerPreset {
    /// Convert the preset into a concrete [`McpServerConfig`].
    pub fn to_config(&self) -> McpServerConfig {
        let env_map = if self.env.is_empty() {
            None
        } else {
            let mut map = HashMap::new();
            for (key, value) in self.env {
                map.insert((*key).to_string(), (*value).to_string());
            }
            Some(map)
        };

        McpServerConfig {
            preset: Some(self.id.to_string()),
            command: self.command.to_string(),
            args: self.args.iter().map(|value| (*value).to_string()).collect(),
            env: env_map,
            startup_timeout_sec: self.startup_timeout,
            tool_timeout_sec: self.tool_timeout,
        }
    }
}

const CHROME_DEVTOOLS: McpServerPreset = McpServerPreset {
    id: "chrome_devtools",
    label: "Chrome DevTools",
    description: "Launch the Chrome DevTools MCP server via npx for debugging frontends.",
    command: "npx",
    args: &["chrome-devtools-mcp@latest", "--stdio"],
    env: &[],
    startup_timeout: Some(Duration::from_secs(45)),
    tool_timeout: Some(Duration::from_secs(120)),
};

const PRESETS: &[McpServerPreset] = &[CHROME_DEVTOOLS];

/// Return the list of built-in MCP presets.
pub fn builtin_mcp_server_presets() -> &'static [McpServerPreset] {
    PRESETS
}

/// Lookup a preset by its identifier.
pub fn find_mcp_server_preset(id: &str) -> Option<&'static McpServerPreset> {
    PRESETS.iter().find(|preset| preset.id == id)
}
