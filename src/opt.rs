use clap::ValueHint;
use std::path::PathBuf;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Parser)]
#[command(version, about)]
pub struct Opt {
    /// Executable to call
    #[arg(
        long,
        env = "JQ_BIN",
        default_value = "gojq",
        value_hint = ValueHint::CommandName,
    )]
    pub jq_bin: String,

    /// Override the path to fzf
    #[arg(
        long,
        env = "FZF_BIN",
        default_value = "fzf",
        value_hint = ValueHint::CommandName,
    )]
    pub fzf_bin: String,

    /// Path to a program that accepts the query string from stdin, and prints a number
    ///
    /// Useful for knowing how many characters you have left in the prompt.
    // https://github.com/junegunn/fzf/commit/4e5e925e39ead3c04865a1d9595715905ef276d2#diff-12758c127796978b99b1b8ccfc3c2092eb834f3b73c4c2e3fd9ebd5a9d7acda5R29
    // https://github.com/junegunn/fzf/blob/master/CHANGELOG.md#0603
    #[arg(
        long,
        env = "JQ_REPL_CHARCOUNTER_BIN",
        default_value = "charcounter",
        value_hint = ValueHint::CommandName,
    )]
    pub charcounter_bin: String,

    /// Arguments to pass to the "charcounter" binary
    ///
    /// For example, if you want to use `wc` as the counter, you could pass `-m`
    /// to have it return the character count.
    #[arg(long, allow_hyphen_values = true)]
    pub charcounter_options: Vec<String>,

    #[arg(
        long,
        env = "JQ_REPL_BRAILLE_BIN",
        default_value = "braille",
        value_hint = ValueHint::CommandName,
    )]
    pub braille_bin: String,

    /// Path to the history file (use ^P and ^N to navigate it)
    ///
    /// History is only recorded when query is accepted (enter is pressed).
    #[arg(
        long,
        env = "JQ_REPL_HISTORY",
        default_value = concat!(env!("HOME"), "/.jq_repl_history"),
        value_hint = ValueHint::DirPath,
    )]
    pub history_file: PathBuf,

    /// Usw `null` as input value
    ///
    /// This is the default when no file path was given and standard input is from an
    /// interactive terminal.
    #[arg(short, long)]
    pub null_input: bool,

    /// The flag to pass to `jq` inside fzf to indicate null input
    #[arg(long, default_value = "-n")]
    pub null_input_flag: String,

    /// Pass content to `jq` as standard input (e.g. `jq < /path/to/file`)
    #[arg(long)]
    pub pass_as_stdin: bool,

    /// Don't interpret input as JSON
    ///
    /// The flag passed to jq can be customized with `--raw-input-flag`.
    #[arg(short = 'R', long)]
    pub raw_input: bool,

    /// Print the fzf command that would be run to stdout and exit.
    #[arg(long, visible_alias = "print-fzf-command")]
    pub show_fzf_command: bool,

    /// Disable the default arguments
    #[arg(long = "no-default-args", action(clap::ArgAction::SetFalse))]
    pub use_default_args: bool,

    /// Path to jq library functions directory
    ///
    /// Should also have a file inside called `.jq`.
    #[arg(
        long,
        env,
        default_value = "~/.jq",
        value_hint = ValueHint::DirPath,
    )]
    pub jq_repl_lib: PathBuf,

    /// Disable including ".jq" as part of the default include paths
    #[arg(long, env)]
    pub no_default_include: bool,

    /// The flag passed to jq inside fzf to show color
    #[arg(long, allow_hyphen_values = true, default_value = "-C")]
    pub color_flag: String,

    /// The flag passed to jq inside fzf to disable color
    #[arg(long, allow_hyphen_values = true, default_value = "-M")]
    pub no_color_flag: String,

    /// The flag passed to jq inside fzf to use a compact format
    #[arg(long, allow_hyphen_values = true, default_value = "-c")]
    pub compact_flag: String,

    #[arg(long, allow_hyphen_values = true, default_value = "-R")]
    pub raw_input_flag: String,

    /// Editor to open inside fzf
    #[arg(
        long,
        env = "EDITOR",
        default_value = "nvim",
        value_hint = ValueHint::CommandName,
    )]
    pub editor: String,

    /// Arguments to pass to the editor
    ///
    /// This should accept reading from standard input, and should block until it quits.
    #[arg(long, allow_hyphen_values = true, default_value = "-c 'set ft=json' -")]
    pub editor_options: Vec<String>,

    /// JSON files to read from (defaults to standard input)
    ///
    /// If one of the files is "-", insert stdin at that point.
    pub files: Vec<PathBuf>,

    /// Pager to pipe output to
    #[arg(long, default_value = "less", value_hint = ValueHint::CommandName)]
    pub pager: String,

    /// Options to pass to the pager
    #[arg(long, allow_hyphen_values = true, default_value = "")]
    pub pager_options: Vec<String>,

    /// Show versions of all relevant executables
    #[arg(long)]
    pub version_verbose: bool,

    /// Pass additional arguments to `fzf`, terminated with a semicolon
    #[arg(long, num_args(1..), allow_hyphen_values = true, value_terminator = ";")]
    #[arg(verbatim_doc_comment)]
    pub fzf_args: Vec<String>,

    /// Additional args passed to `jq`
    #[arg(last = true)]
    pub jq_args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_sanity_check() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }
}
