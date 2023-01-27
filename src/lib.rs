mod error;
mod opt;

use clap::Parser;
pub use error::Error;
use opt::Opt;
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
        print_cmd_version(&opt.fzf_bin, "--version");
        print_cmd_version(&opt.bin, "--version");
        print_cmd_version("bat", "--version");
        print_cmd_version(&opt.pager, "--version");

        return Ok(());
    }

    opt.null_input = opt.null_input || (atty::is(atty::Stream::Stdin) && opt.filename.is_none());
    if opt.null_input {
        opt.args.push("-n".to_string());
    }

    // Keep a reference to the temp file alive until we quit
    let (mut fzf_cmd, _path) = build_fzf_cmd(&opt)?;

    if opt.show_fzf_command {
        println!("#!/bin/bash");
        println!();
        println!(
            "{} \\",
            shell_quote::bash::quote(fzf_cmd.get_program())
                .to_str()
                .unwrap()
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
            Self::File(file) => write!(
                f,
                "{}",
                shell_quote::bash::quote(file)
                    .to_str()
                    .expect("Only valid unicode filenames are allowed")
            ),
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
            "--bind={key}:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file} | \
             {cmd}"
        )
    };

    let external_with_color = |key: &str, cmd: &str| {
        format!("--bind={key}:execute:{jq_bin} {jq_arg_prefix} {{q}} {input_file} | {cmd}")
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
        ["M-e: nvim", "M-v: vd", "M-l: pager", "^<space>: gron"].join(" ⁄ "),
    ))
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
    .args([
        format!(
            "--bind=alt-s:change-prompt(-s> )+change-preview:{jq_bin} {jq_arg_prefix} --slurp \
             {{q}} {input_file}"
        ),
        format!(
            "--bind=alt-S:change-prompt(> )+change-preview:{jq_bin} {jq_arg_prefix} {{q}} \
             {input_file}"
        ),
    ])
    .args([
        format!(
            "--bind=alt-c:change-prompt(-c> )+preview:{jq_bin} {jq_arg_prefix} {} {{q}} \
             {input_file}",
            &opt.compact_flag
        ),
        format!(
            "--bind=alt-C:change-prompt(> )+preview:{jq_bin} {jq_arg_prefix} {{q}} {input_file}"
        ),
    ])
    .arg(format!(
        "--bind=ctrl-space:change-prompt(gron> )+change-preview:{jq_bin} {jq_arg_prefix} \
         {no_color_flag} {{q}} {input_file} | gron --colorize"
    ))
    .arg(format!(
        "--bind=alt-space:change-prompt(> )+change-preview:{jq_bin} {jq_arg_prefix} {{q}} \
         {input_file}"
    ))
    .arg(external("alt-e", "nvim -c 'set ft=json' -"))
    .arg(external("alt-v", "vd -f json"))
    .arg(external("alt-V", "vd -f csv"))
    .arg(external_with_color(
        "alt-l",
        &format!("{} {}", &opt.pager, &opt.pager_options.join(" ")),
    ))
    .arg(external("alt-L", "bat -l json"))
    .args(&opt.fzf_args)
    .stdin(Stdio::null())
    .stdout(Stdio::inherit());

    Ok((fzf, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }
}
