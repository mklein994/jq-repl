mod error;

use clap::Parser;
pub use error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, clap::Parser)]
pub struct Opt {
    /// Executable to call
    #[clap(long, env, default_value = "jq")]
    bin: String,

    /// Path to the history file (use ^P and ^N to navigate it)
    ///
    /// History is only recorded when query is accepted (enter is pressed).
    #[clap(long, env = "JQ_REPL_HISTORY", default_value = concat!(env!("HOME"), "/.jq_repl_history"))]
    history_file: PathBuf,

    /// Name of the JSON file to read from (defaults to standard input)
    filename: Option<PathBuf>,

    /// Pager to pipe output to
    #[clap(long, default_value = "less")]
    pager: String,

    /// Options to pass to the pager
    #[clap(long, allow_hyphen_values = true)]
    pager_options: Vec<String>,

    /// Additional args passed to `jq`
    #[clap(last = true)]
    args: Vec<String>,
}

pub fn run() -> Result<(), Error> {
    let opt = Opt::parse();
    let (fzf_cmd, path) = build_fzf_cmd(&opt)?;
    let query = get_query(fzf_cmd)?;

    eprintln!("{:?}", if query.is_empty() { "." } else { &query });

    let mut output_cmd = build_output_cmd(&opt.bin, &path, &opt.args, &query)?;

    let is_output_interactive = atty::is(atty::Stream::Stdout);
    if is_output_interactive {
        output_cmd.stdout(Stdio::piped());

        let fzf_handle = output_cmd.spawn()?;

        Command::new(&opt.pager)
            .args(&opt.pager_options)
            .stdin(fzf_handle.stdout.unwrap())
            .stdout(Stdio::inherit())
            .spawn()?
            .wait()?;
    } else {
        output_cmd.stdout(Stdio::inherit()).spawn()?.wait()?;
    }

    Ok(())
}

pub fn build_output_cmd(
    jq_bin: &str,
    input: &InputFile,
    args: &[String],
    output: &str,
) -> Result<Command, Error> {
    let file = match input {
        InputFile::File(file) => File::open(file)?,
        InputFile::Stdin(stdin) => File::open(stdin)?,
    };

    let mut jq = Command::new(jq_bin);

    jq.args(args).arg(output).stdin(file);
    Ok(jq)
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputFile<'a> {
    Stdin(PathBuf),
    File(&'a Path),
}

impl<'a> Drop for InputFile<'a> {
    fn drop(&mut self) {
        if let Self::Stdin(path) = self {
            std::fs::remove_file(path).expect("failed to remove temp file!");
        }
    }
}

impl<'a> std::fmt::Display for InputFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin(file) => file.display().fmt(f),
            Self::File(file) => file.display().fmt(f),
        }
    }
}

pub fn build_fzf_cmd(opt: &Opt) -> Result<(Command, InputFile), Error> {
    let path = match &opt.filename {
        Some(filename) => InputFile::File(filename),
        None => {
            let mut stdin = std::io::stdin();
            let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
            std::io::copy(&mut stdin, &mut file)?;
            InputFile::Stdin(path)
        }
    };

    let input = path.to_string();
    let jq_bin = &opt.bin;

    let echo = Command::new("echo").stdout(Stdio::piped()).spawn()?;

    let mut jq_prefix = format!("{jq_bin} --color-output --raw-output -L~/.local/lib/jq/.jq");
    let args = opt.args.join(" ");
    if !args.is_empty() {
        jq_prefix.push(' ');
        jq_prefix.push_str(&args);
    }

    let jq_history_file = opt.history_file.display();

    let bind = |key: &str, undo_key: &str, value: &str| {
        [
            format!("--bind={key}:preview:{jq_prefix} {value} {{q}} {input}"),
            format!("--bind={undo_key}:preview:{jq_prefix} {{q}} {input}"),
        ]
    };

    let mut fzf = Command::new("fzf");
    fzf.args(["--disabled", "--print-query", "--preview-window=up,99%"])
        .arg(format!("--history={jq_history_file}"))
        .arg(format!("--preview={jq_prefix} {{q}} {input}"))
        .arg(format!(
            "--bind={}",
            [
                ("ctrl-k", "kill-line"),
                ("pgup", "preview-page-up"),
                ("pgdn", "preview-page-down"),
                ("alt-w", "toggle-preview-wrap"),
                ("home", "preview-top"),
                ("end", "preview-bottom"),
            ]
            .map(|(key, value)| [key, value].join(":"))
            .join(","),
        ))
        .args(bind("alt-s", "alt-S", "--slurp"))
        .args(bind("alt-c", "alt-C", "--compact-output"))
        .arg(format!(
            "--bind=ctrl-space:preview:{jq_prefix} --monochrome-output {{q}} {input} | \
             gron --colorize"
        ))
        .stdin(echo.stdout.unwrap())
        .stdout(Stdio::piped());

    Ok((fzf, path))
}

pub fn get_query(mut fzf: Command) -> Result<String, Error> {
    let output = fzf.stderr(Stdio::inherit()).output()?;

    if output.status.success() {
        let out = String::from_utf8(output.stdout)?;
        let out = out.trim();
        Ok(out.to_string())
    } else {
        Err(Error::Fzf(output.status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }

    #[test]
    #[ignore]
    fn check_fzf_args() {
        let name = env!("CARGO_CRATE_NAME");
        let opt = Opt::parse_from(&[
            name,
            "--history-file",
            "/tmp/.jq_repl_history",
            "/tmp/foo.json",
        ]);
        let (fzf, _) = build_fzf_cmd(&opt).unwrap();
        let args: Vec<_> = fzf.get_args().collect();

        assert_eq!(
            vec![
                "--disabled",
                "--print-query",
                "--preview-window=up,99%",
                "--history=/tmp/.jq_repl_history",
                "--preview=jq --color-output --raw-output {q} /tmp/foo.json",
                "--bind=ctrl-k:kill-line,pgup:preview-page-up,pgdn:preview-page-down,alt-w:\
                 toggle-preview-wrap,home:preview-top,end:preview-bottom",
                "--bind=alt-s:preview:jq --color-output --raw-output --slurp {q} /tmp/foo.json",
                "--bind=alt-S:preview:jq --color-output --raw-output {q} /tmp/foo.json",
                "--bind=alt-c:preview:jq --color-output --raw-output --compact-output {q} \
                 /tmp/foo.json",
                "--bind=alt-C:preview:jq --color-output --raw-output {q} /tmp/foo.json",
                "--bind=ctrl-space:change-preview:jq --color-output --raw-output \
                 --monochrome-output {q} /tmp/foo.json | gron --colorize",
            ],
            args
        );
    }
}
