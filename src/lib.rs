use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

#[derive(clap::Parser)]
struct Opt {
    /// Name of the JSON file to read from (defaults to standard input)
    filename: Option<PathBuf>,

    /// Additional args passed to `jq`
    #[clap(last = true)]
    jq: Vec<String>,
}

pub fn run() -> Result<Option<ExitStatus>, Error> {
    let opt = Opt::parse();

    let (path, is_temp) = match opt.filename {
        Some(filename) if filename.exists() => (filename, false),
        Some(_) | None => {
            let mut stdin = std::io::stdin();
            let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
            std::io::copy(&mut stdin, &mut file)?;
            (path, true)
        }
    };

    let input = path.display();

    let echo = Command::new("echo")
        .arg("")
        .stdout(Stdio::piped())
        .spawn()?;

    let jq_prefix = format!("jq --color-output --raw-output {}", opt.jq.join(" "));

    let bind = |key: &str, undo_key: &str, value: &str| {
        [
            format!("--bind={key}:preview:{jq_prefix} {value} {{q}} {input}"),
            format!("--bind={undo_key}:preview:{jq_prefix} {{q}} {input}"),
        ]
    };

    let mut fzf = Command::new("fzf")
        .args(["--print-query", "--preview-window=up,99%"])
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
        .spawn()?;

    let status = match fzf.try_wait() {
        Ok(Some(status)) => Some(status),
        Ok(None) => Some(fzf.wait()?),
        Err(err) => {
            return Err(Error::from(err));
        }
    };

    if is_temp {
        std::fs::remove_file(path)?;
    }

    Ok(status)
}

pub use error::Error;
mod error {
    use std::fmt;

    #[derive(Debug)]
    pub enum Error {
        Io(std::io::Error),
        TmpPersist(tempfile::PersistError),
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Io(err) => err.fmt(f),
                Self::TmpPersist(err) => err.fmt(f),
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
}
