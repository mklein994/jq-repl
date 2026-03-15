use std::collections::BTreeMap;
use std::path::Path;

/// Top-level configuration, deserialized from `config.toml`.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub keybinds: Keybinds,
    pub lens: BTreeMap<String, Lens>,
    pub external: BTreeMap<String, External>,
}

/// Global key bindings not tied to a specific lens or external tool.
#[derive(Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Keybinds {
    pub reset_lens: String,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            reset_lens: "alt-G".to_string(),
        }
    }
}

/// A lens: pipes jq output through a command for alternative display.
///
/// Color is always suppressed on the jq side before piping, since the command is expected to handle
/// its own coloring.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lens {
    pub command: String,
    pub key: String,
}

/// An external tool: opens jq output in another program.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct External {
    pub command: String,
    pub key: String,
    /// Extra flags passed to jq before piping to the command (e.g. `["-c"]`).
    #[serde(default)]
    pub jq_flags: Vec<String>,
}

impl Config {
    /// Load config from the given path.
    ///
    /// Returns `Ok(None)` if the file does not exist.
    pub fn load(path: &Path) -> Result<Option<Self>, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => Ok(Some(toml::from_str(&contents)?)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}
