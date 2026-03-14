use clap::Parser;
use jq_repl::{
    Prompt,
    transform::{TransformConfig, transform_actions},
};

/// Emit fzf actions to atomically update the prompt and preview for jq-repl.
///
/// Reads the current prompt from `FZF_PROMPT`, optionally applies a flag toggle or program switch,
/// then prints a `change-prompt(...)+change-preview(...)` action string to stdout.
///
/// This is intended to be used with fzf's `transform:` action binding. Using a single `transform:`
/// call keeps the prompt and preview update atomic — avoiding the state drift that would occur if
/// `transform-prompt` and `change-preview` were chained with `+`.
///
/// Static configuration (jq binary, argument prefix, color flags, renderer commands) is read from
/// environment variables set by `jq-repl` at startup:
///
/// | Name                    | Description                                               |
/// |-------------------------+-----------------------------------------------------------|
/// | `JQ_REPL_JQ_BIN`        | jq binary name (e.g. "gojq")                              |
/// | `JQ_REPL_JQ_ARG_PREFIX` | static jq arguments (library paths, `--raw-output`, etc.) |
/// | `JQ_REPL_COLOR_FLAG`    | flag to enable color (e.g. `-C`)                          |
/// | `JQ_REPL_NO_COLOR_FLAG` | flag to disable color (e.g. `-M`)                         |
/// | `JQ_REPL_GRON_CMD`      | pipe command when gron is active                          |
/// | `JQ_REPL_BRAILLE_CMD`   | pipe command when braille is active                       |
#[derive(Debug, Parser)]
#[command(name = "_jq-repl-transform", version, verbatim_doc_comment)]
struct TransformOpts {
    /// The current prompt string to transform
    ///
    /// Usually provided via the `FZF_PROMPT` environment variable set by fzf.
    #[arg(long, allow_hyphen_values = true, env = "FZF_PROMPT")]
    prompt: String,

    /// Add or remove a flag from the prompt string (e.g. "+c", "-c")
    #[arg(short, allow_hyphen_values = true)]
    flag: Option<String>,

    /// Set or clear the rendering program name in the prompt (e.g. "gron", "braille")
    #[arg(short)]
    #[allow(
        clippy::option_option,
        reason = "clap understands this means both the flag and the argument are optional"
    )]
    program: Option<Option<String>>,

    /// Input file paths to pass to jq (already shell-quoted)
    #[arg(trailing_var_arg = true)]
    input_file_paths: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = TransformOpts::parse();

    let mut prompt = opts.prompt.parse::<Prompt>().unwrap();
    prompt.transform(opts.flag, opts.program);

    let config = TransformConfig::from_env(opts.input_file_paths.join(" "))?;

    println!("{}", transform_actions(&prompt, &config));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <TransformOpts as clap::CommandFactory>::command().debug_assert();
    }
}
