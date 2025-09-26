//! Types used to define the fields of [`crate::config::Config`].

// Note this file should generally be restricted to simple struct/enum
// definitions that do not contain business logic.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use wildmatch::WildMatchPattern;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::Error as SerdeError;

use crate::mcp_presets::find_mcp_server_preset;
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct McpServerConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,

    pub command: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// Startup timeout in seconds for initializing MCP server & initially listing tools.
    #[serde(
        default,
        with = "option_duration_secs",
        skip_serializing_if = "Option::is_none"
    )]
    pub startup_timeout_sec: Option<Duration>,

    /// Default timeout for MCP tool calls initiated via this server.
    #[serde(default, with = "option_duration_secs")]
    pub tool_timeout_sec: Option<Duration>,
}

impl<'de> Deserialize<'de> for McpServerConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMcpServerConfig {
            #[serde(default)]
            preset: Option<String>,
            #[serde(default)]
            command: Option<String>,
            #[serde(default)]
            args: Vec<String>,
            #[serde(default)]
            env: Option<HashMap<String, String>>,
            #[serde(default)]
            startup_timeout_sec: Option<f64>,
            #[serde(default)]
            startup_timeout_ms: Option<u64>,
            #[serde(default, with = "option_duration_secs")]
            tool_timeout_sec: Option<Duration>,
        }

        let raw = RawMcpServerConfig::deserialize(deserializer)?;

        let RawMcpServerConfig {
            preset,
            command,
            args,
            env,
            startup_timeout_sec: raw_startup_timeout_sec,
            startup_timeout_ms,
            tool_timeout_sec,
        } = raw;

        let startup_timeout_override = match (raw_startup_timeout_sec, startup_timeout_ms) {
            (Some(sec), _) => {
                let duration = Duration::try_from_secs_f64(sec).map_err(SerdeError::custom)?;
                Some(duration)
            }
            (None, Some(ms)) => Some(Duration::from_millis(ms)),
            (None, None) => None,
        };

        let preset_config = match preset.as_deref() {
            Some(preset_id) => {
                let preset = find_mcp_server_preset(preset_id).ok_or_else(|| {
                    SerdeError::custom(format!("unknown MCP server preset '{preset_id}'"))
                })?;
                Some(preset.to_config())
            }
            None => None,
        };

        let mut command_value = if let Some(cfg) = preset_config.as_ref() {
            cfg.command.clone()
        } else {
            command
                .clone()
                .ok_or_else(|| SerdeError::missing_field("command"))?
        };
        if let Some(cmd) = command {
            command_value = cmd;
        }

        let mut args_value = preset_config
            .as_ref()
            .map(|cfg| cfg.args.clone())
            .unwrap_or_default();
        if !args.is_empty() {
            args_value = args;
        }

        let mut env_value = preset_config
            .as_ref()
            .and_then(|cfg| cfg.env.clone())
            .unwrap_or_default();
        if let Some(env_override) = env {
            if env_value.is_empty() {
                env_value = env_override;
            } else {
                for (key, value) in env_override {
                    env_value.insert(key, value);
                }
            }
        }

        let mut startup_timeout_value = preset_config
            .as_ref()
            .and_then(|cfg| cfg.startup_timeout_sec);
        if let Some(duration) = startup_timeout_override {
            startup_timeout_value = Some(duration);
        }

        let mut tool_timeout_value = preset_config.as_ref().and_then(|cfg| cfg.tool_timeout_sec);
        if let Some(duration) = tool_timeout_sec {
            tool_timeout_value = Some(duration);
        }

        Ok(Self {
            preset,
            command: command_value,
            args: args_value,
            env: if env_value.is_empty() {
                None
            } else {
                Some(env_value)
            },
            startup_timeout_sec: startup_timeout_value,
            tool_timeout_sec: tool_timeout_value,
        })
    }
}

mod option_duration_secs {
    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;
    use std::time::Duration;

    pub fn serialize<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(duration) => serializer.serialize_some(&duration.as_secs_f64()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = Option::<f64>::deserialize(deserializer)?;
        secs.map(|secs| Duration::try_from_secs_f64(secs).map_err(serde::de::Error::custom))
            .transpose()
    }
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum UriBasedFileOpener {
    #[serde(rename = "vscode")]
    VsCode,

    #[serde(rename = "vscode-insiders")]
    VsCodeInsiders,

    #[serde(rename = "windsurf")]
    Windsurf,

    #[serde(rename = "cursor")]
    Cursor,

    /// Option to disable the URI-based file opener.
    #[serde(rename = "none")]
    None,
}

impl UriBasedFileOpener {
    pub fn get_scheme(&self) -> Option<&str> {
        match self {
            UriBasedFileOpener::VsCode => Some("vscode"),
            UriBasedFileOpener::VsCodeInsiders => Some("vscode-insiders"),
            UriBasedFileOpener::Windsurf => Some("windsurf"),
            UriBasedFileOpener::Cursor => Some("cursor"),
            UriBasedFileOpener::None => None,
        }
    }
}

/// Settings that govern if and what will be written to `~/.codex/history.jsonl`.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct History {
    /// If true, history entries will not be written to disk.
    pub persistence: HistoryPersistence,

    /// If set, the maximum size of the history file in bytes.
    /// TODO(mbolin): Not currently honored.
    pub max_bytes: Option<usize>,
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HistoryPersistence {
    /// Save all history entries to disk.
    #[default]
    SaveAll,
    /// Do not write history to disk.
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Notifications {
    Enabled(bool),
    Custom(Vec<String>),
}

impl Default for Notifications {
    fn default() -> Self {
        Self::Enabled(false)
    }
}

/// Collection of settings that are specific to the TUI.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Tui {
    /// Enable desktop notifications from the TUI when the terminal is unfocused.
    /// Defaults to `false`.
    #[serde(default)]
    pub notifications: Notifications,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct SandboxWorkspaceWrite {
    #[serde(default)]
    pub writable_roots: Vec<PathBuf>,
    #[serde(default)]
    pub network_access: bool,
    #[serde(default)]
    pub exclude_tmpdir_env_var: bool,
    #[serde(default)]
    pub exclude_slash_tmp: bool,
}

impl From<SandboxWorkspaceWrite> for codex_protocol::mcp_protocol::SandboxSettings {
    fn from(sandbox_workspace_write: SandboxWorkspaceWrite) -> Self {
        Self {
            writable_roots: sandbox_workspace_write.writable_roots,
            network_access: Some(sandbox_workspace_write.network_access),
            exclude_tmpdir_env_var: Some(sandbox_workspace_write.exclude_tmpdir_env_var),
            exclude_slash_tmp: Some(sandbox_workspace_write.exclude_slash_tmp),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ShellEnvironmentPolicyInherit {
    /// "Core" environment variables for the platform. On UNIX, this would
    /// include HOME, LOGNAME, PATH, SHELL, and USER, among others.
    Core,

    /// Inherits the full environment from the parent process.
    #[default]
    All,

    /// Do not inherit any environment variables from the parent process.
    None,
}

/// Policy for building the `env` when spawning a process via either the
/// `shell` or `local_shell` tool.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ShellEnvironmentPolicyToml {
    pub inherit: Option<ShellEnvironmentPolicyInherit>,

    pub ignore_default_excludes: Option<bool>,

    /// List of regular expressions.
    pub exclude: Option<Vec<String>>,

    pub r#set: Option<HashMap<String, String>>,

    /// List of regular expressions.
    pub include_only: Option<Vec<String>>,

    pub experimental_use_profile: Option<bool>,
}

pub type EnvironmentVariablePattern = WildMatchPattern<'*', '?'>;

/// Deriving the `env` based on this policy works as follows:
/// 1. Create an initial map based on the `inherit` policy.
/// 2. If `ignore_default_excludes` is false, filter the map using the default
///    exclude pattern(s), which are: `"*KEY*"` and `"*TOKEN*"`.
/// 3. If `exclude` is not empty, filter the map using the provided patterns.
/// 4. Insert any entries from `r#set` into the map.
/// 5. If non-empty, filter the map using the `include_only` patterns.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShellEnvironmentPolicy {
    /// Starting point when building the environment.
    pub inherit: ShellEnvironmentPolicyInherit,

    /// True to skip the check to exclude default environment variables that
    /// contain "KEY" or "TOKEN" in their name.
    pub ignore_default_excludes: bool,

    /// Environment variable names to exclude from the environment.
    pub exclude: Vec<EnvironmentVariablePattern>,

    /// (key, value) pairs to insert in the environment.
    pub r#set: HashMap<String, String>,

    /// Environment variable names to retain in the environment.
    pub include_only: Vec<EnvironmentVariablePattern>,

    /// If true, the shell profile will be used to run the command.
    pub use_profile: bool,
}

impl From<ShellEnvironmentPolicyToml> for ShellEnvironmentPolicy {
    fn from(toml: ShellEnvironmentPolicyToml) -> Self {
        // Default to inheriting the full environment when not specified.
        let inherit = toml.inherit.unwrap_or(ShellEnvironmentPolicyInherit::All);
        let ignore_default_excludes = toml.ignore_default_excludes.unwrap_or(false);
        let exclude = toml
            .exclude
            .unwrap_or_default()
            .into_iter()
            .map(|s| EnvironmentVariablePattern::new_case_insensitive(&s))
            .collect();
        let r#set = toml.r#set.unwrap_or_default();
        let include_only = toml
            .include_only
            .unwrap_or_default()
            .into_iter()
            .map(|s| EnvironmentVariablePattern::new_case_insensitive(&s))
            .collect();
        let use_profile = toml.experimental_use_profile.unwrap_or(false);

        Self {
            inherit,
            ignore_default_excludes,
            exclude,
            r#set,
            include_only,
            use_profile,
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ReasoningSummaryFormat {
    #[default]
    None,
    Experimental,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn preset_expands_defaults() {
        let cfg: McpServerConfig =
            toml::from_str("preset = \"chrome_devtools\"\n").expect("valid preset config");

        assert_eq!(cfg.preset.as_deref(), Some("chrome_devtools"));
        assert_eq!(cfg.command, "npx");
        assert_eq!(
            cfg.args,
            vec![
                "chrome-devtools-mcp@latest".to_string(),
                "--stdio".to_string()
            ]
        );
        assert_eq!(cfg.startup_timeout_sec, Some(Duration::from_secs(45)));
        assert_eq!(cfg.tool_timeout_sec, Some(Duration::from_secs(120)));
    }

    #[test]
    fn preset_respects_overrides() {
        let cfg: McpServerConfig = toml::from_str(
            "preset = \"chrome_devtools\"\ncommand = \"/custom/bin\"\nargs = [\"--foo\"]\nenv = { CUSTOM = \"1\" }\nstartup_timeout_sec = 5\ntool_timeout_sec = 7\n",
        )
        .expect("valid override config");

        assert_eq!(cfg.preset.as_deref(), Some("chrome_devtools"));
        assert_eq!(cfg.command, "/custom/bin");
        assert_eq!(cfg.args, vec!["--foo".to_string()]);
        let mut expected_env = HashMap::new();
        expected_env.insert("CUSTOM".to_string(), "1".to_string());
        assert_eq!(cfg.env, Some(expected_env));
        assert_eq!(cfg.startup_timeout_sec, Some(Duration::from_secs(5)));
        assert_eq!(cfg.tool_timeout_sec, Some(Duration::from_secs(7)));
    }
}
