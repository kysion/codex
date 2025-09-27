use std::option_env;

/// The current Codex CLI version as embedded at compile time.
pub const CODEX_CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const KYSION_BUILD_VERSION: &str = match option_env!("KYSION_BUILD_VERSION") {
    Some(version) => version,
    None => CODEX_CLI_VERSION,
};

pub const OFFICIAL_BASE_VERSION: &str = match option_env!("KYSION_BASE_VERSION") {
    Some(version) => version,
    None => "0.41.0",
};
