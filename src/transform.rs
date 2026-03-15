use crate::Prompt;
use std::collections::HashMap;

/// Configuration for building fzf transform actions.
///
/// All fields are sourced from environment variables set by `jq-repl` at startup, so they are
/// available to `_jq-repl-transform` without being repeated in every binding string.
pub struct TransformConfig {
    pub jq_bin: String,
    pub jq_arg_prefix: String,
    pub color_flag: String,
    pub no_color_flag: String,
    /// Lens commands keyed by name, sourced from `JQ_REPL_LENS_<NAME>` env vars.
    pub lenses: HashMap<String, String>,
    pub input_file_paths: String,
}

impl TransformConfig {
    /// Read config from the `JQ_REPL_*` environment variables set by `jq-repl`.
    pub fn from_env(input_file_paths: String) -> Result<Self, std::env::VarError> {
        // Collect all JQ_REPL_LENS_* env vars into a name -> command map. The name is lowercased so
        // lookups against the prompt label are case-insensitive.
        let lenses = std::env::vars()
            .filter_map(|(key, value)| {
                key.strip_prefix("JQ_REPL_LENS_")
                    .map(|name| (name.to_lowercase(), value))
            })
            .collect();

        Ok(Self {
            jq_bin: std::env::var("JQ_REPL_JQ_BIN")?,
            jq_arg_prefix: std::env::var("JQ_REPL_JQ_ARG_PREFIX").unwrap_or_default(),
            color_flag: std::env::var("JQ_REPL_COLOR_FLAG").unwrap_or_default(),
            no_color_flag: std::env::var("JQ_REPL_NO_COLOR_FLAG").unwrap_or_default(),
            lenses,
            input_file_paths,
        })
    }
}

/// Build the fzf action string for the given prompt state and config.
///
/// Returns a `change-prompt(...)+change-preview(...)` string suitable for use as the output of
/// fzf's `transform:` action.
#[must_use]
pub fn transform_actions(prompt: &Prompt, config: &TransformConfig) -> String {
    let pipe: Option<&str> = prompt
        .program()
        .and_then(|name| config.lenses.get(name).map(String::as_str));

    let color_flag = if pipe.is_some() {
        config.no_color_flag.as_str()
    } else {
        config.color_flag.as_str()
    };

    let jq_flags = prompt.jq_flags();
    let files = config.input_file_paths.as_str();
    let parts: Vec<&str> = [
        config.jq_arg_prefix.trim(),
        color_flag,
        jq_flags.as_str(),
        "{q}",
        files,
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect();

    let jq_bin = &config.jq_bin;
    let jq_params = parts.join(" ");
    let jq_cmd = match pipe {
        Some(p) => format!("{jq_bin} {jq_params} | {p}"),
        None => format!("{jq_bin} {jq_params}"),
    };

    format!("change-prompt({prompt})+change-preview({jq_cmd})")
}
