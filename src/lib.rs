mod error;
mod opt;

use clap::Parser;
pub use error::Error;
use opt::Opt;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_JQ_ARG_PREFIX: &[&str] = &[
    "-L",
    "~/.jq", // setup the module path
    "-L",
    "~/.jq/.jq", // import all modules
    "--raw-output",
];

pub fn run() -> Result<(), Error> {
    let mut opt = Opt::parse();

    if opt.version_verbose {
        let print_cmd_version = |name: &str, version_flag: &str| {
            let version = String::from_utf8(
                Command::new(name)
                    .arg(version_flag)
                    .output()
                    .unwrap()
                    .stdout,
            )
            .unwrap();
            println!("{name}:\t{}", version.trim());
        };

        println!("{} {}", clap::crate_name!(), clap::crate_version!());
        println!();
        print_cmd_version("fzf", "--version");
        print_cmd_version(&opt.bin, "--version");
        print_cmd_version(&opt.pager, "--version");

        return Ok(());
    }

    opt.null_input = opt.null_input || (atty::is(atty::Stream::Stdin) && opt.filename.is_none());
    if opt.null_input {
        opt.args.push("-n".to_string());
    }

    let (fzf_cmd, path) = build_fzf_cmd(&opt)?;

    if opt.show_fzf_command {
        println!(
            "{} \\",
            shell_quote::bash::quote(fzf_cmd.get_program())
                .to_str()
                .unwrap()
        );
        println!(
            "{}",
            fzf_cmd
                .get_args()
                .map(|arg| {
                    shell_quote::bash::quote(arg)
                        .to_str()
                        .expect("Failed to convert arg to UTF-8 string")
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(" \\\n"),
        );
        return Ok(());
    }

    let query = get_query(fzf_cmd)?;

    eprintln!("{:?}", if query.is_empty() { "." } else { &query });

    let mut jq_cmd = build_jq_cmd(&opt.bin, &path, opt.use_default_args, &opt.args, &query)?;

    let is_output_interactive = atty::is(atty::Stream::Stdout);
    if is_output_interactive {
        jq_cmd.stdout(Stdio::piped());

        let jq = jq_cmd.spawn()?;

        Command::new(&opt.pager)
            .args(&opt.pager_options)
            .stdin(jq.stdout.unwrap())
            .stdout(Stdio::inherit())
            .spawn()?
            .wait()?;
    } else {
        jq_cmd.stdout(Stdio::inherit()).spawn()?.wait()?;
    }

    Ok(())
}

pub fn build_jq_cmd(
    jq_bin: &str,
    input_file: &InputFile,
    use_default_args: bool,
    args: &[String],
    output: &str,
) -> Result<Command, Error> {
    let file = match input_file {
        InputFile::File(file) => File::open(file)?,
        InputFile::Stdin(stdin) => File::open(stdin)?,
    };

    let mut jq = Command::new(jq_bin);

    if use_default_args {
        jq.args(DEFAULT_JQ_ARG_PREFIX);
    }
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
            let (mut file, filename) = tempfile::NamedTempFile::new()?.keep()?;
            if !opt.null_input {
                std::io::copy(&mut std::io::stdin(), &mut file)?;
            }
            InputFile::Stdin(filename)
        }
    };

    let input_file = if opt.null_input {
        String::new()
    } else {
        path.to_string()
    };

    let jq_bin = &opt.bin;

    let echo = Command::new("echo").stdout(Stdio::piped()).spawn()?;

    let mut jq_arg_prefix = if opt.use_default_args {
        [DEFAULT_JQ_ARG_PREFIX, &[&opt.color_flag]]
            .concat()
            .join(" ")
    } else {
        opt.color_flag.to_string()
    };

    let no_color_flag = &opt.no_color_flag;

    let args = &opt.args;
    if !args.is_empty() {
        jq_arg_prefix.push(' ');
        jq_arg_prefix.push_str(&args.join(" "));
    }

    let jq_history_file = opt.history_file.display();

    let bind = |key: &str, undo_key: &str, value: &str| {
        [
            format!("--bind={key}:preview:{jq_bin} {jq_arg_prefix} {value} {{q}} {input_file}"),
            format!("--bind={undo_key}:preview:{jq_bin} {jq_arg_prefix} {{q}} {input_file}"),
        ]
    };

    let mut fzf = Command::new("fzf");
    fzf.args([
        "--disabled",
        "--print-query",
        "--preview-window=up,99%,border-bottom",
        "--info=hidden",
    ])
    .arg(format!("--history={jq_history_file}"))
    .arg(format!(
        "--preview={jq_bin} {jq_arg_prefix} {{q}} {input_file}"
    ))
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
    .args(bind("alt-c", "alt-C", &opt.compact_flag))
    .arg(format!(
        "--bind=ctrl-space:preview:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file} | \
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
}
