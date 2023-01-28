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
        print_cmd_version(&opt.pager, "--version");

        return Ok(());
    }

    opt.null_input = opt.null_input || (atty::is(atty::Stream::Stdin) && opt.filenames.is_empty());
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

#[derive(Debug)]
pub enum InputFile<'a> {
    Stdin(File, PathBuf),
    File(&'a Path),
}

impl<'a> PartialEq for InputFile<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stdin(_, l0), Self::Stdin(_, r0)) => l0 == r0,
            (Self::File(l0), Self::File(r0)) => l0 == r0,
            _ => false,
        }
    }
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
                    .expect("Only valid unicode filenames are allowed")
            ),
        }
    }
}

pub fn build_fzf_cmd(opt: &Opt) -> Result<(Command, Vec<InputFile>), Error> {
    let mut filenames: Vec<InputFile> = vec![];

    let has_piped_input = atty::isnt(atty::Stream::Stdin);

    let input_file = if opt.show_fzf_command {
        opt.filenames
            .iter()
            .map(|x| x.to_str().expect("only unicode names are allowed"))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        for filename in &opt.filenames {
            if has_piped_input && matches!(filename.to_str(), Some("-")) {
                let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
                std::io::copy(&mut std::io::stdin(), &mut file)?;

                filenames.push(InputFile::Stdin(file, path));
            } else if !filename.is_file() {
                let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
                let mut source = File::open(filename)?;
                std::io::copy(&mut source, &mut file)?;

                filenames.push(InputFile::Stdin(file, path));
            } else {
                filenames.push(InputFile::File(filename));
            }
        }

        if has_piped_input && filenames.is_empty() {
            let (mut file, path) = tempfile::NamedTempFile::new()?.keep()?;
            std::io::copy(&mut std::io::stdin(), &mut file)?;
            filenames.push(InputFile::Stdin(file, path));
        }

        filenames
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ")
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

    Ok((fzf, filenames))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <Opt as clap::CommandFactory>::command().debug_assert();
    }
}
