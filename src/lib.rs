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
        print_verbose_version(&opt);
        return Ok(());
    }

    opt.null_input = opt.null_input || (atty::is(atty::Stream::Stdin) && opt.files.is_empty());
    if opt.null_input {
        opt.args.push("-n".to_string());
    }

    let files = get_files(&opt.files)?;

    let input_file_paths = files
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");

    // Keep a reference to the temp file alive until we quit
    let mut fzf_cmd = build_fzf_cmd(&opt, &input_file_paths)?;

    if opt.show_fzf_command {
        print_fzf_command(&fzf_cmd);
        return Ok(());
    }

    let status = fzf_cmd.spawn()?.wait()?;

    // Forward the return status from fzf. An error code of 1 means no match was found,
    // which is meaningless here.
    if status.success() || matches!(status.code(), Some(1)) {
        Ok(())
    } else {
        Err(Error::Fzf(status))
    }
}

fn print_fzf_command(fzf_cmd: &Command) {
    println!("#!/bin/bash");
    println!();
    println!(
        "{} \\",
        shell_quote::bash::quote(fzf_cmd.get_program())
            .to_str()
            .unwrap()
            .trim()
    );
    println!(
        "{} < /dev/null",
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
}

fn print_verbose_version(opt: &Opt) {
    let print_cmd_version = |name: &str, version_flag: &str| {
        let version_output = String::from_utf8(
            Command::new(name)
                .arg(version_flag)
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        let version = version_output.lines().next().unwrap();
        println!("{name}:\t{version}");
    };

    println!("{} {}", clap::crate_name!(), clap::crate_version!());
    println!();
    print_cmd_version(&opt.fzf_bin, "--version");
    print_cmd_version(&opt.bin, "--version");
    print_cmd_version("bat", "--version");
    print_cmd_version("vd", "--version");
    print_cmd_version(&opt.editor, "--version");
    print_cmd_version(&opt.pager, "--version");
}

#[derive(Debug)]
pub enum InputFile<'a> {
    Stdin(File, PathBuf),
    File(&'a Path),
}

impl<'a> Drop for InputFile<'a> {
    fn drop(&mut self) {
        if let Self::Stdin(_, path) = self {
            std::fs::remove_file(path).expect("failed to remove temp file!");
        }
    }
}

impl<'a> std::fmt::Display for InputFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin(_, path) => path.display().fmt(f),
            Self::File(file) => write!(
                f,
                "{}",
                shell_quote::bash::quote(file)
                    .to_str()
                    .expect("Only valid unicode file names are allowed")
            ),
        }
    }
}

pub fn build_fzf_cmd(opt: &Opt, input_file_paths: &str) -> Result<Command, Error> {
    let jq_bin = &opt.bin;

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

    let external = |key: &str, cmd: &str| {
        format!(
            "--bind={key}:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} \
             {input_file_paths} | {cmd}"
        )
    };

    let external_with_color = |key: &str, cmd: &str| {
        format!("--bind={key}:execute:{jq_bin} {jq_arg_prefix} {{q}} {input_file_paths} | {cmd}")
    };

    let mut fzf = Command::new(&opt.fzf_bin);
    fzf.args([
        "--disabled",
        "--preview-window=up,99%,border-bottom",
        "--info=hidden",
        "--header-first",
    ])
    .arg(format!(
        "--header={}",
        ["M-e: editor", "M-v: vd", "M-l: pager", "^<space>: gron"].join(" ⁄ "),
    ))
    .arg(format!("--history={jq_history_file}"))
    .arg(format!(
        "--preview={jq_bin} {jq_arg_prefix} {{q}} {input_file_paths}"
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
    .args([
        format!(
            "--bind=alt-s:change-prompt(-s> )+change-preview:{jq_bin} {jq_arg_prefix} --slurp \
             {{q}} {input_file_paths}"
        ),
        format!(
            "--bind=alt-S:change-prompt(> )+change-preview:{jq_bin} {jq_arg_prefix} {{q}} \
             {input_file_paths}"
        ),
    ])
    .args([
        format!(
            "--bind=alt-c:change-prompt(-c> )+preview:{jq_bin} {jq_arg_prefix} {} {{q}} \
             {input_file_paths}",
            &opt.compact_flag
        ),
        format!(
            "--bind=alt-C:change-prompt(> )+preview:{jq_bin} {jq_arg_prefix} {{q}} \
             {input_file_paths}"
        ),
    ])
    .args([
        format!(
            "--bind=ctrl-space:change-prompt(gron> )+change-preview:{jq_bin} {jq_arg_prefix} \
             {no_color_flag} {{q}} {input_file_paths} | gron --colorize"
        ),
        format!(
            "--bind=alt-space:change-prompt(> )+change-preview:{jq_bin} {jq_arg_prefix} {{q}} \
             {input_file_paths}"
        ),
    ])
    .arg(external(
        "alt-e",
        &format!("{} {}", &opt.editor, &opt.editor_options.join(" ")),
    ))
    .arg(external("alt-v", "vd --filetype json"))
    .arg(external("alt-V", "vd --filetype csv"))
    .arg(external_with_color(
        "alt-l",
        &format!("{} {}", &opt.pager, &opt.pager_options.join(" ")),
    ))
    .arg(external("alt-L", "bat --language json"))
    .args(&opt.fzf_args)
    .stdin(Stdio::null())
    .stdout(Stdio::inherit());

    Ok(fzf)
}

fn get_files(positional_files: &[PathBuf]) -> Result<Vec<InputFile>, Error> {
    let mut files: Vec<InputFile> = vec![];

    let has_piped_input = atty::isnt(atty::Stream::Stdin);

    for file_name in positional_files {
        if has_piped_input && matches!(file_name.to_str(), Some("-")) {
            let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
            std::io::copy(&mut std::io::stdin(), &mut file)?;

            files.push(InputFile::Stdin(file, path));
        } else if !file_name.is_file() {
            let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
            let mut source = File::open(file_name)?;
            std::io::copy(&mut source, &mut file)?;

            files.push(InputFile::Stdin(file, path));
        } else {
            files.push(InputFile::File(file_name));
        }
    }

    if has_piped_input && files.is_empty() {
        let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
        std::io::copy(&mut std::io::stdin(), &mut file)?;
        files.push(InputFile::Stdin(file, path));
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }
}
