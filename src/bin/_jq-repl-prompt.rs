use clap::Parser;
use jq_repl::Prompt;

/// Transform a prompt for jq-repl.
///
/// This program expects the environment variable `FZF_PROMPT` to be set, e.g. "> ", "-n> ", or
/// "gron> ". The idea is that short flags passed to `jq` are shown chained together, and if a
/// rendering program is run, the name of it is appended.
///
/// If known flags are set, they are shown at the start with a leading '-', e.g. `--null-input`
/// (`-n`) is shown as "-n> ". If a program is set, the program name is added to the prompt, after a
/// space if there are flags (e.g. "braille> ", "-n braille> ").
///
/// There will always be a trailing "> ".
#[derive(Debug, Parser)]
#[command(name = "_jq-repl-prompt", version)]
struct PromptOpts {
    /// Set the prompt string to manipulate
    ///
    /// This is usually set by `fzf`, but in case you are using a different program, or testing, it
    /// can be useful to set this manually.
    #[arg(long, env = "FZF_PROMPT")]
    prompt: String,

    /// Add or remove a flag from the prompt string
    ///
    /// Expects a '+' or '-' character (to add or remove the flag respectively), followed by the
    /// single character to add or remove from the prompt. Some examples: "+n", "-c".
    #[arg(short, allow_hyphen_values = true)]
    flag: Option<String>,

    /// Add or remove the custom program name from the prompt
    ///
    /// Passing only the flag removes the program from the prompt. Some examples: "braille", "gron".
    #[arg(short)]
    #[allow(
        clippy::option_option,
        reason = "clap understands this means both the flag the argument are optional"
    )]
    program: Option<Option<String>>,
}

fn main() {
    let opts = PromptOpts::parse();
    let current_prompt = &opts.prompt;
    let mut prompt = current_prompt.parse::<Prompt>().unwrap();
    prompt.update(opts.flag, opts.program);

    println!("{prompt}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <PromptOpts as clap::CommandFactory>::command().debug_assert();
    }
}
