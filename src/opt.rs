use std::path::PathBuf;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Parser)]
#[command(version, about)]
pub struct Opt {
    /// Executable to call
    #[arg(long, env = "JQ_BIN", default_value = "gojq")]
    pub bin: String,

    /// Path to the history file (use ^P and ^N to navigate it)
    ///
    /// History is only recorded when query is accepted (enter is pressed).
    #[arg(long, env = "JQ_REPL_HISTORY", default_value = concat!(env!("HOME"), "/.jq_repl_history"))]
    pub history_file: PathBuf,

    /// Usw `null` as input value
    ///
    /// This is the default when no file path was given and standard input is from an
    /// interactive terminal.
    #[arg(short, long)]
    pub null_input: bool,

    /// Print the fzf command that would be run to stdout and exit.
    #[arg(long)]
    pub show_fzf_command: bool,

    /// Disable the default arguments
    #[arg(long = "no-default-args", action(clap::ArgAction::SetFalse))]
    pub use_default_args: bool,

    /// The flag passed to jq inside fzf to show color
    #[arg(long, allow_hyphen_values = true, default_value = "-C")]
    pub color_flag: String,

    /// The flag passed to jq inside fzf to disable color
    #[arg(long, allow_hyphen_values = true, default_value = "-M")]
    pub no_color_flag: String,

    /// The flag passed to jq inside fzf to use a compact format
    #[arg(long, allow_hyphen_values = true, default_value = "-c")]
    pub compact_flag: String,

    /// Name of the JSON file to read from (defaults to standard input)
    pub filename: Option<PathBuf>,

    /// Pager to pipe output to
    #[arg(long, default_value = "less")]
    pub pager: String,

    /// Options to pass to the pager
    #[arg(long, allow_hyphen_values = true)]
    pub pager_options: Vec<String>,

    /// Show versions of all relevant executables
    #[arg(long)]
    pub version_verbose: bool,

    /// Additional args passed to `jq`
    #[arg(last = true)]
    pub args: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_sanity_check() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }
}
