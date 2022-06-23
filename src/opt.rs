use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[clap(version, about)]
pub struct Opt {
    /// Executable to call
    #[clap(long, env = "JQ_BIN", default_value = "gojq")]
    pub bin: String,

    /// Path to the history file (use ^P and ^N to navigate it)
    ///
    /// History is only recorded when query is accepted (enter is pressed).
    #[clap(long, env = "JQ_REPL_HISTORY", default_value = concat!(env!("HOME"), "/.jq_repl_history"))]
    pub history_file: PathBuf,

    /// Name of the JSON file to read from (defaults to standard input)
    pub filename: Option<PathBuf>,

    /// Pager to pipe output to
    #[clap(long, default_value = "less")]
    pub pager: String,

    /// Options to pass to the pager
    #[clap(long, allow_hyphen_values = true)]
    pub pager_options: Vec<String>,

    /// Additional args passed to `jq`
    #[clap(last = true)]
    pub args: Vec<String>,
}
