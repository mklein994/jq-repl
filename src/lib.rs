use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, clap::Parser)]
pub struct Opt {
    /// Executable to call
    #[clap(long, env, default_value = "jq")]
    jq_bin: String,

    /// Name of the JSON file to read from (defaults to standard input)
    filename: Option<PathBuf>,

    /// Additional args passed to `jq`
    #[clap(last = true)]
    args: Vec<String>,
}

pub fn run() -> Result<(), Error> {
    let opt = Opt::parse();
    let (cmd, path) = build_cmd(&opt)?;
    let query = get_query(cmd)?;

    eprintln!("{:?}", query);

    let output = build_output_cmd(&opt.jq_bin, &path, &opt.args, &query)?.output()?;

    print!("{}", String::from_utf8(output.stdout)?);

    if let InputFile::Stdin(file) = path {
        std::fs::remove_file(file)?;
    }

    Ok(())
}

pub fn build_output_cmd(
    jq_bin: &str,
    input: &InputFile,
    args: &[String],
    output: &str,
) -> Result<Command, Error> {
    let cat = Command::new("cat")
        .arg(input.to_string())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut jq = Command::new(jq_bin);

    jq.args(args).arg(output).stdin(cat.stdout.unwrap());
    Ok(jq)
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputFile<'a> {
    Stdin(PathBuf),
    File(&'a Path),
}

impl<'a> std::fmt::Display for InputFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin(file) => file.display().fmt(f),
            Self::File(file) => file.display().fmt(f),
        }
    }
}

pub fn build_cmd(opt: &Opt) -> Result<(Command, InputFile), Error> {
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
    let jq_bin = &opt.jq_bin;

    let echo = Command::new("echo").stdout(Stdio::piped()).spawn()?;

    let jq_prefix = format!(
        "{jq_bin} --color-output --raw-output {}",
        opt.args.join(" ")
    );

    let jq_history_file = Path::new(concat!(env!("HOME"), "/.jq_repl_history")).display();

    let bind = |key: &str, undo_key: &str, value: &str| {
        [
            format!("--bind={key}:preview:{jq_prefix} {value} {{q}} {input}"),
            format!("--bind={undo_key}:preview:{jq_prefix} {{q}} {input}"),
        ]
    };

    let mut cmd = Command::new("fzf");
    cmd.args(["--disabled", "--print-query", "--preview-window=up,99%"])
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
        .stdin(echo.stdout.unwrap())
        .stdout(Stdio::piped());

    Ok((cmd, path))
}

pub fn get_query(mut fzf: Command) -> Result<String, Error> {
    let child = fzf.spawn()?;

    let query = match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            let out = String::from_utf8(output.stdout)?;
            let out = out.trim();
            Ok(out.to_string())
        }
        Ok(output) => Err(Error::Fzf(output.status)),
        Err(err) => Err(err.into()),
    };

    query
}

pub use error::Error;
mod error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        Io(std::io::Error),
        TmpPersist(tempfile::PersistError),
        Utf8(std::string::FromUtf8Error),
        Fzf(std::process::ExitStatus),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(err) => err.fmt(f),
                Self::TmpPersist(err) => err.fmt(f),
                Self::Utf8(err) => err.fmt(f),
                Self::Fzf(err) => write!(f, "{err}"),
            }
        }
    }

    impl std::error::Error for Error {}

    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            Self::Io(err)
        }
    }

    impl From<tempfile::PersistError> for Error {
        fn from(err: tempfile::PersistError) -> Self {
            Self::TmpPersist(err)
        }
    }

    impl From<std::string::FromUtf8Error> for Error {
        fn from(err: std::string::FromUtf8Error) -> Self {
            Self::Utf8(err)
        }
    }
}
