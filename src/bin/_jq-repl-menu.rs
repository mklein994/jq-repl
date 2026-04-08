#[macro_use]
extern crate log;

use clap::Parser;
use jq_repl::Prompt;
use jq_repl::menu::{self, MenuState};
use std::path::PathBuf;

/// Open or close the jq-repl menu, emitting fzf transform actions.
///
/// When opening, captures the current fzf state (prompt, query, preview command, preview window)
/// into a JSON state file, then emits actions to switch fzf into menu mode. When closing, reads
/// the state file back and emits actions to restore the previous state.
///
/// Whether to open or close is determined by whether the state file exists: if it does, the menu
/// is currently open and should be closed; if it doesn't, the menu should be opened.
///
/// Static configuration is read from environment variables set by `jq-repl` at startup:
///
/// | Name                       | Description                                              |
/// |----------------------------+----------------------------------------------------------|
/// | `JQ_REPL_JQ_BIN`           | jq binary name (e.g. "gojq")                            |
/// | `JQ_REPL_JQ_ARG_PREFIX`    | static jq arguments (library paths, `--raw-output`, etc.)|
/// | `JQ_REPL_COLOR_FLAG`       | flag to enable color (e.g. `-C`)                         |
/// | `JQ_REPL_INPUT_FILE_PATHS` | shell-quoted input file paths                            |
/// | `JQ_REPL_MENU_STATE_FILE`  | path to write/read the JSON state snapshot               |
/// | `JQ_REPL_MENU_FILE`        | path to the file containing menu items (tab-delimited)   |
#[derive(Debug, Parser)]
#[command(name = "_jq-repl-menu", version, verbatim_doc_comment)]
struct MenuOpts {
    /// The current prompt string
    #[arg(long, allow_hyphen_values = true, env = "FZF_PROMPT")]
    prompt: String,

    /// The current query string
    #[arg(long, env = "FZF_QUERY")]
    query: String,

    #[arg(long)]
    accept: Option<String>,
}

fn run() -> anyhow::Result<()> {
    let opts = MenuOpts::parse();

    let state_path = PathBuf::from(std::env::var("JQ_REPL_STATE_PATH")?);
    let menu_path = PathBuf::from(std::env::var("JQ_REPL_MENU_PATH")?);
    let keys_to_unbind = std::env::var("JQ_REPL_MENU_KEYS_TO_UNBIND")?;
    let menu_height = menu::calculate_menu_height(5 + 1)?; // target + header lines

    let input_file_paths = std::env::var("JQ_REPL_INPUT_FILE_PATHS").unwrap_or_default();

    debug!("current prompt: {}", opts.prompt);

    if opts.prompt.starts_with('[') {
        // Menu is open — close it and restore saved state
        let state = MenuState::load(&state_path)?;
        std::fs::remove_file(&state_path)?;

        if let Some(action) = &opts.accept {
            print!("{}", menu::accept_actions(&state, action)?);
        } else {
            print!("{}", menu::close_actions(&state));
        }
    } else {
        // Menu is closed — open it and save current state
        let jq_bin = std::env::var("JQ_REPL_JQ_BIN")?;
        let jq_arg_prefix = std::env::var("JQ_REPL_JQ_ARG_PREFIX").unwrap_or_default();
        let color_flag = std::env::var("JQ_REPL_COLOR_FLAG").unwrap_or_default();
        let transform_bin = std::env::var("JQ_REPL_TRANSFORM_BIN")?;

        // Reconstruct the jq flags from the current prompt state
        let prompt: Prompt = opts.prompt.parse().unwrap();
        let jq_flags = prompt.jq_flags();

        // Assemble the frozen preview command, embedding the query directly instead of {q}
        // so it doesn't update as the user types to filter the menu
        let frozen_query = jq_repl::bash_quote(&opts.query);
        let frozen_parts: Vec<&str> = [
            jq_arg_prefix.trim(),
            color_flag.as_str(),
            jq_flags.as_str(),
            &frozen_query,
            input_file_paths.trim(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
        let frozen_preview = format!("{jq_bin} {}", frozen_parts.join(" "));

        let parts: Vec<&str> = [
            jq_arg_prefix.trim(),
            color_flag.as_str(),
            jq_flags.as_str(),
            "{q}",
            input_file_paths.trim(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
        let preview = format!("{jq_bin} {}", parts.join(" "));

        let state = MenuState {
            key_bindings: keys_to_unbind,
            prompt: opts.prompt,
            query: opts.query,
            frozen_preview,
            preview,
            transform_bin,
            reset_key: std::env::var("JQ_REPL_RESET_KEY")?,
            menu_height,
        };

        state.save(&state_path)?;
        println!("{}", menu::open_actions(&state, &menu_path)?);
    }

    Ok(())
}

fn main() {
    jq_repl::setup_logging("debug.log", &format!("menu: pid {}", std::process::id()))
        .expect("failed to write to debug log");

    if let Err(err) = run() {
        // Change the header to "ERROR" in bold, bright red, and show the error message on the
        // preview window
        println!(
            "change-header(\x1b[1;91mERROR\x1b[22;39m)+change-preview:echo {}",
            jq_repl::bash_quote(format!("{err}")),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <MenuOpts as clap::CommandFactory>::command().debug_assert();
    }
}
