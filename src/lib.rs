mod error;
mod opt;

use clap::Parser;
pub use error::Error;
use opt::Opt;
use shell_quote::Bash;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

fn get_jq_arg_prefix(opt: &Opt) -> String {
    if opt.use_default_args {
        let default_lib_dir = &opt.jq_repl_lib; // setup the module path
        let default_lib_prelude = default_lib_dir.join(".jq"); // import all modules
        let default_arg_prefix = &[
            "-L",
            &default_lib_dir.to_string_lossy(),
            "-L",
            &default_lib_prelude.to_string_lossy(),
            "--raw-output",
        ];
        [&default_arg_prefix[..], &[&opt.color_flag]]
            .concat()
            .join(" ")
    } else {
        opt.color_flag.to_string()
    }
}

pub fn run() -> Result<(), Error> {
    let mut opt = Opt::parse();

    if opt.version_verbose {
        print_verbose_versions(&opt)?;
        return Ok(());
    }

    opt.null_input = opt.null_input || (atty::is(atty::Stream::Stdin) && opt.files.is_empty());
    if opt.null_input {
        opt.args.push("-n".to_string());
    }

    if opt.raw_input {
        opt.args.push(opt.raw_input_flag.to_string());
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
        String::from_utf8(Bash::quote(fzf_cmd.get_program()))
            .unwrap()
            .trim()
    );
    println!(
        "{} < /dev/null",
        fzf_cmd
            .get_args()
            .map(|arg| {
                String::from_utf8(Bash::quote(arg)).expect("Failed to convert arg to UTF-8 string")
            })
            .collect::<Vec<_>>()
            .join(" \\\n"),
    );
}

fn print_cmd_version(name: &str, version_flag: &str) -> Result<(), Error> {
    use std::io::ErrorKind;

    let output = Command::new(name).arg(version_flag).output();
    let version_output = match output {
        Err(err) if err.kind() == ErrorKind::NotFound => "(error: not found)".to_string(),
        Ok(output) => String::from_utf8(output.stdout)?,
        Err(err) => Err(err)?,
    };

    let version = version_output.lines().next().unwrap_or("(error: unknown)");
    println!("{name}:\t{version}");

    Ok(())
}

fn print_verbose_versions(opt: &Opt) -> Result<(), Error> {
    println!("{} {}", clap::crate_name!(), clap::crate_version!());
    println!();
    print_cmd_version(&opt.fzf_bin, "--version")?;
    print_cmd_version(&opt.jq_bin, "--version")?;
    print_cmd_version("bat", "--version")?;
    print_cmd_version("vd", "--version")?;
    print_cmd_version(&opt.braille_bin, "--version")?;
    print_cmd_version(&opt.editor, "--version")?;
    print_cmd_version(&opt.pager, "--version")?;
    if &opt.charcounter_bin == "charcounter" {
        println!(
            "{}:\t{}",
            &opt.charcounter_bin,
            if Command::new(&opt.charcounter_bin)
                .stdin(Stdio::null())
                .status()
                .is_ok()
            {
                "OK"
            } else {
                "Not Found"
            }
        );
    } else {
        print_cmd_version(&opt.charcounter_bin, "--version")?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum InputFile<'a> {
    Stdin(NamedTempFile),
    File(&'a Path),
}

impl<'a> std::fmt::Display for InputFile<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = match self {
            Self::Stdin(file) => file.path(),
            Self::File(path) => path,
        };

        write!(
            f,
            "{}",
            String::from_utf8(Bash::quote(path)).expect("Only valid UTF-8 file names are allowed")
        )
    }
}

#[allow(clippy::too_many_lines)]
pub fn build_fzf_cmd(opt: &Opt, input_file_paths: &str) -> Result<Command, Error> {
    let jq_bin = &opt.jq_bin;

    let mut jq_arg_prefix = get_jq_arg_prefix(opt);

    let no_color_flag = &opt.no_color_flag;

    let args = &opt.args;
    if !args.is_empty() {
        jq_arg_prefix.push(' ');
        jq_arg_prefix.push_str(&args.join(" "));
    }

    let jq_history_file = opt.history_file.display();

    let null_flag = if opt.null_input { "n" } else { "" };

    let null_flag_standalone = if opt.null_input { "-n" } else { "" };

    let mut fzf = Command::new(&opt.fzf_bin);
    fzf.args([
        "--disabled",
        "--preview-window=up,99%,border-none",
        "--info=hidden",
        "--header-first",
        &format!("--prompt={null_flag_standalone}> "),
    ])
    .arg(format!(
        "--header={}",
        [
            "M-e: editor",
            "M-j: vd",
            "M-l: pager",
            "M-g: braille",
            "^<space>: gron"
        ]
        .join(" ⁄ "),
    ))
    .arg(format!("--history={jq_history_file}"))
    .arg("--preview-label-pos=-1")
    .arg(format!(
        "--bind=change:transform-preview-label:printf \"%s\" {{q}} | {} {}",
        &opt.charcounter_bin,
        &opt.charcounter_options.join(" "),
    ))
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
            (
                if cfg!(target_os = "android") {
                    // This key repeats when held in Termux
                    "up"
                } else {
                    "tab"
                },
                "refresh-preview"
            ),
        ]
        .map(|(key, value)| [key, value].join(":"))
        .join(","),
    ))
    .args([
        format!(
            "--bind=alt-s:change-prompt(-{null_flag}s> )+change-preview:{jq_bin} {jq_arg_prefix} \
             --slurp {{q}} {input_file_paths}"
        ),
        format!(
            "--bind=alt-S:change-prompt({null_flag_standalone}> )+change-preview:{jq_bin} \
             {jq_arg_prefix} {{q}} {input_file_paths}"
        ),
    ])
    .args([
        format!(
            "--bind=alt-c:change-prompt(-{null_flag}c> )+change-preview:{jq_bin} {jq_arg_prefix} \
             {} {{q}} {input_file_paths}",
            &opt.compact_flag
        ),
        format!(
            "--bind=alt-C:change-prompt({null_flag_standalone}> )+change-preview:{jq_bin} \
             {jq_arg_prefix} {{q}} {input_file_paths}"
        ),
    ])
    .args([
        format!(
            "--bind=ctrl-space:change-prompt({null_flag_standalone} gron> \
             )+change-preview:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
             gron --colorize"
        ),
        format!(
            "--bind=alt-space:change-prompt({null_flag_standalone}> )+change-preview:{jq_bin} \
             {jq_arg_prefix} {{q}} {input_file_paths}"
        ),
    ])
    .args([
        format!(
            "--bind=alt-g:change-prompt({null_flag_standalone} braille> )+change-preview:{jq_bin} \
             {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
             BRAILLE_USE_FULL_DEFAULT_HEIGHT=1 {braille_bin} --modeline",
            braille_bin = &opt.braille_bin
        ),
        format!(
            "--bind=alt-G:change-prompt({null_flag_standalone}> )+change-preview:{jq_bin} \
             {jq_arg_prefix} {{q}} {input_file_paths}"
        ),
    ])
    .arg(format!(
        "--bind=alt-e:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
         {} {}",
        &opt.editor,
        &opt.editor_options.join(" ")
    ))
    .arg(format!(
        "--bind=alt-j:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
         vd --filetype json"
    ))
    .arg(format!(
        "--bind=alt-J:execute:{jq_bin} {jq_arg_prefix} {compact_flag} {no_color_flag} {{q}} \
         {input_file_paths} | vd --filetype jsonl",
        compact_flag = &opt.compact_flag
    ))
    .arg(format!(
        "--bind=alt-v:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
         vd --filetype csv"
    ))
    .arg(format!(
        "--bind=alt-l:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
         {} {}",
        &opt.pager,
        &opt.pager_options.join(" ")
    ))
    .arg(format!(
        "--bind=alt-L:execute:{jq_bin} {jq_arg_prefix} {no_color_flag} {{q}} {input_file_paths} | \
         bat --language json --paging always"
    ))
    .args(&opt.fzf_args)
    .stdin(Stdio::null())
    .stdout(Stdio::inherit());

    Ok(fzf)
}

fn get_files(positional_files: &[PathBuf]) -> Result<Vec<InputFile>, Error> {
    let mut files: Vec<InputFile> = vec![];
    let cat_file = PathBuf::from("-");

    let has_piped_input = atty::isnt(atty::Stream::Stdin);

    for file_name in positional_files {
        if file_name == &cat_file {
            let mut file = NamedTempFile::new()?;
            std::io::copy(&mut std::io::stdin(), &mut file)?;

            files.push(InputFile::Stdin(file));
        } else if !file_name.is_file() {
            let mut file = NamedTempFile::new()?;
            let mut source = File::open(file_name)?;
            std::io::copy(&mut source, &mut file)?;

            files.push(InputFile::Stdin(file));
        } else {
            files.push(InputFile::File(file_name));
        }
    }

    if has_piped_input && files.is_empty() {
        let mut file = NamedTempFile::new()?;
        std::io::copy(&mut std::io::stdin(), &mut file)?;
        files.push(InputFile::Stdin(file));
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
