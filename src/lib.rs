pub mod config;
mod error;
pub mod menu;
mod opt;
mod prompt;
pub mod transform;

#[macro_use]
extern crate log;

use clap::Parser;
pub use config::Config;
use directories::ProjectDirs;
pub use error::Error;
use opt::Opt;
pub use prompt::Prompt;
use shell_quote::{Bash, Quote};
use std::fs::File;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::{NamedTempFile, TempPath};

pub fn setup_logging<P: AsRef<Path>>(log_file_name: P, msg: &str) -> Result<(), Error> {
    let target = if std::env::var("RUST_LOG").is_ok_and(|x| !x.is_empty()) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(log_file_name);
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        env_logger::Target::Pipe(Box::new(log_file))
    } else {
        env_logger::Target::default()
    };

    env_logger::builder()
        .target(target)
        .write_style(env_logger::WriteStyle::Always)
        .try_init()?;

    info!("{msg}");

    Ok(())
}

pub fn run() -> Result<(), Error> {
    setup_logging(
        "debug.log",
        &format!(":: started jq-repl: pid {}", std::process::id()),
    )?;
    let opt = Opt::parse();

    if opt.version_verbose {
        print_verbose_versions(&opt)?;
        return Ok(());
    }

    if let Some(shell) = opt.completion {
        let mut cmd = <Opt as clap::CommandFactory>::command();
        clap_complete::generate(shell, &mut cmd, clap::crate_name!(), &mut std::io::stdout());
        return Ok(());
    }

    let project = ProjectDirs::from("", "", clap::crate_name!()).ok_or_else(|| Error::Project)?;

    let config = if opt.clean {
        Config::default()
    } else {
        let config_path = opt
            .config
            .clone()
            .unwrap_or_else(|| project.config_dir().join("config.toml"));
        Config::load(&config_path)?.unwrap_or_default()
    };

    let paths = ResolvedPaths::new(&opt, &project)?;
    menu::write_menu_contents(&paths.menu_path, &config)?;

    let files = get_files(&opt.files)?;

    if files.len() > 1 && opt.pass_as_stdin {
        let err = <Opt as clap::CommandFactory>::command().error(
            clap::error::ErrorKind::ArgumentConflict,
            "must only pass one file with --pass-as-stdin",
        );
        return Err(Error::from(err));
    }

    let input_files = files
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let input_file_paths = if opt.pass_as_stdin {
        format!("< {}", &input_files[0])
    } else {
        input_files.join(" ")
    };

    // Keep a reference to the temp file alive until we quit
    let mut fzf_cmd = build_fzf_cmd(&opt, &config, &paths, &input_file_paths)?;

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

    let print_builtin_command = |cmd: &str| {
        println!(
            "{}:\t{}",
            cmd,
            if Command::new(cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .status()
                .is_ok()
            {
                "OK"
            } else {
                "Not Found"
            }
        );
    };

    if &opt.charcounter_bin == "_jq-repl-charcounter" {
        print_builtin_command(&opt.charcounter_bin);
    } else {
        print_cmd_version(&opt.charcounter_bin, "--version")?;
    }

    if &opt.completion_bin == "_jq-repl-tab-completion" {
        print_builtin_command(&opt.completion_bin);
    } else {
        print_cmd_version(&opt.completion_bin, "--version")?;
    }

    print_cmd_version(&opt.transform_bin, "--version")?;

    Ok(())
}

pub fn bash_quote(s: impl AsRef<std::ffi::OsStr>) -> String {
    String::from_utf8(Bash::quote(s.as_ref())).expect("Bash::quote always produces valid UTF-8")
}

pub fn bash_quote_join<T: AsRef<std::ffi::OsStr>>(args: impl IntoIterator<Item = T>) -> String {
    args.into_iter()
        .map(bash_quote)
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug)]
pub enum InputFile<'a> {
    Stdin(NamedTempFile),
    File(&'a Path),
}

impl std::fmt::Display for InputFile<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = match self {
            Self::Stdin(file) => file.path(),
            Self::File(path) => path,
        };

        write!(f, "{}", bash_quote(path))
    }
}

/// Manage resolving temporary files or state files used by jq-repl
pub struct ResolvedPaths {
    pub state_path: TempPath,
    pub menu_path: TempPath,
    pub history_file: Option<PathBuf>,
}

impl ResolvedPaths {
    /// Determine the paths to use from the given options and configuration
    ///
    /// When using default files, it may create the files and parent directories if they don't
    /// exist.
    fn new(opt: &Opt, project: &ProjectDirs) -> Result<Self, Error> {
        let state_path = if let Some(path) = &opt.state_path {
            TempPath::try_from_path(path)?
        } else {
            tempfile::Builder::new()
                .prefix("jq_repl_state.")
                .suffix(".json")
                .tempfile()?
                .into_temp_path()
        };

        let menu_path = if let Some(path) = &opt.menu_path {
            TempPath::try_from_path(path)?
        } else {
            tempfile::Builder::new()
                .prefix("jq_repl_menu.")
                .tempfile()?
                .into_temp_path()
        };

        let history_file = Self::resolve_history_file(opt, project)?;

        Ok(Self {
            state_path,
            menu_path,
            history_file,
        })
    }

    /// Determine what the path to the history file from the options and project directories.
    ///
    /// If the default XDG path is used, parent directories up to the file path are created if they
    /// don't exist.
    fn resolve_history_file(opt: &Opt, project: &ProjectDirs) -> Result<Option<PathBuf>, Error> {
        if opt.no_history {
            Ok(None)
        } else if opt.history_file.is_some() {
            Ok(opt.history_file.clone())
        } else {
            let path = project.data_dir().join("history");

            if let Some(parent_dir) = path.parent() {
                std::fs::create_dir_all(parent_dir)?;
            }

            Ok(Some(path))
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "passing arguments to fzf is inherently complicated"
)]
pub fn build_fzf_cmd(
    opt: &Opt,
    config: &Config,
    paths: &ResolvedPaths,
    input_file_paths: &str,
) -> Result<Command, Error> {
    let jq_bin = &opt.jq_bin;

    let jq_arg_prefix = opt.get_jq_arg_prefix();

    let mut fzf = Command::new(&opt.fzf_bin);

    let keys_to_unbind = vec![
        config.keybinds.reset_lens.as_str(),
        config.keybinds.cycle_frozen_headers.as_str(),
        "alt-c",
        "alt-C",
        "tab",
    ]
    .into_iter()
    .chain(config.external.values().map(|x| x.key.as_str()))
    .chain(config.lens.values().map(|x| x.key.as_str()))
    .collect::<Vec<_>>();

    // Add some jq-repl environment variables so they can be referenced from within
    fzf.env("JQ_REPL_VERSION", clap::crate_version!())
        .env("JQ_REPL_JQ_BIN", &opt.jq_bin)
        .env("JQ_REPL_JQ_ARG_PREFIX", &jq_arg_prefix)
        .env("JQ_REPL_COLOR_FLAG", &opt.color_flag)
        .env("JQ_REPL_NO_COLOR_FLAG", &opt.no_color_flag)
        .env("JQ_REPL_MENU_PATH", &paths.menu_path)
        .env("JQ_REPL_STATE_PATH", &paths.state_path)
        .env("JQ_REPL_MENU_KEYS_TO_UNBIND", keys_to_unbind.join(","))
        .env("JQ_REPL_TRANSFORM_BIN", &opt.transform_bin)
        .env("JQ_REPL_INPUT_FILE_PATHS", input_file_paths)
        .env("JQ_REPL_RESET_KEY", &config.keybinds.reset_lens);

    // Pass lens commands as env vars so _jq-repl-transform can build the preview command.
    // Each lens is exposed as JQ_REPL_LENS_<NAME> (uppercased).
    for (name, lens) in &config.lens {
        fzf.env(
            format!("JQ_REPL_LENS_{}", name.to_uppercase()),
            &lens.command,
        );
    }

    // Setup layout and style
    fzf.args([
        "--disabled",
        "--preview-window=up,99%,border-bottom",
        "--layout=reverse-list",
        "--no-separator",
        "--info=hidden",
        "--query=.",
    ]);

    if let Some(path) = &paths.history_file {
        // If the menu is open, ctrl-p/ctrl-n go up and down the list, otherwise they navigate
        // history.
        fzf.args([
            "--bind=ctrl-p:transform:[[ $FZF_PROMPT =~ ^\\[ ]] && echo up || echo prev-history",
            "--bind=ctrl-n:transform:[[ $FZF_PROMPT =~ ^\\[ ]] && echo down || echo next-history",
        ]);
        fzf.arg(format!("--history={}", path.display()));
    }

    // Setup the prompt
    //
    // Note that this is used along with environment variables to pass state between jq-repl and
    // _jq-repl-transform, the program that changes how jq is invoked when certain key-bindings are
    // pressed. As a consequence, the format should be kept consistent between the two.
    fzf.arg(format!(
        "--prompt={}",
        Prompt::new(opt.raw_input, opt.null_input())
    ));

    fzf.arg(format!(
        "--preview={jq_bin} {jq_arg_prefix} {} {{q}} {input_file_paths}",
        &opt.color_flag
    ));

    // Setup an indicator that shows when approaching the query limit
    fzf.args([
        &format!(
            "--bind=change:bg-transform-preview-label:printf \"%s\" {{q}} | {} {}",
            bash_quote(&opt.charcounter_bin),
            &opt.charcounter_options.join(" "),
        ),
        "--preview-label-pos=-1", // Right-aligned
    ]);

    // Setup tab completion
    //
    // This is done synchronously since it's affecting user input.
    fzf.arg(format!(
        "--bind=tab:transform-query:echo {{q}} | {}",
        bash_quote(&opt.completion_bin)
    ));

    let menu_bin = bash_quote(&opt.menu_bin);
    fzf.args([
        "--delimiter=\t".to_string(),
        "--with-nth=2".to_string(),
        format!(
            "--bind=enter:transform:if [[ $FZF_PROMPT =~ ^\\[ ]]; then {{ {menu_bin} \
             --accept={{1}}; }} else echo accept; fi"
        ),
        format!(
            "--bind=ctrl-g,esc:transform:[[ $FZF_PROMPT =~ ^\\[ ]] && {menu_bin} || echo abort"
        ),
        format!("--bind=alt-/:transform:{menu_bin}"),
    ]);

    // Simple readline-like key bindings that make life easier
    //
    // Fzf has a lot of readline bindings builtin, but we need to adjust it, since we heavily
    // uses the preview pane, not the results list.
    fzf.arg(format!(
        "--bind={}",
        [
            ("ctrl-k", "kill-line"),
            ("pgup", "preview-page-up"),
            ("pgdn", "preview-page-down"),
            ("alt-w", "toggle-preview-wrap"),
            ("alt-W", "toggle-preview-wrap-word"),
            ("home", "preview-top"),
            ("end", "preview-bottom"),
            // (
            //     if cfg!(target_os = "android") && std::env::var("JQ_REPL_TEST").is_err() {
            //         // This key repeats when held in Termux, but the tab key doesn't
            //         "up"
            //     } else {
            //         "tab"
            //     },
            //     "refresh-preview"
            // ),
        ]
        .map(|(key, value)| [key, value].join(":"))
        .join(","),
    ));

    add_freeze_headers_binding(
        &mut fzf,
        &config.keybinds.cycle_frozen_headers,
        &[1, 2, 3, 0],
    );

    add_runtime_flag_toggle(
        &mut fzf,
        &opt.transform_bin,
        'c',
        "alt-c",
        "alt-C",
        input_file_paths,
    );

    let transform_bin = &opt.transform_bin;

    // Add a binding per configured lens to activate it
    for (name, lens) in &config.lens {
        fzf.arg(format!(
            "--bind={}:bg-transform:{transform_bin} -p {name} -- {input_file_paths}",
            lens.key,
        ));
    }

    // Add a single binding to reset back to the default jq view
    fzf.arg(format!(
        "--bind={}:bg-transform:{transform_bin} -p -- {input_file_paths}",
        config.keybinds.reset_lens,
    ));

    // Add bindings to open output in an external program
    add_external_bindings(
        &mut fzf,
        jq_bin,
        &jq_arg_prefix,
        opt,
        config,
        input_file_paths,
    );

    // Pass additional arguments given on the command line
    fzf.args(&opt.fzf_args);

    fzf.stdin(Stdio::null()).stdout(Stdio::inherit());

    Ok(fzf)
}

fn add_freeze_headers_binding(fzf: &mut Command, binding: &str, lines_frozen: &[u32]) {
    let mut layouts = vec![];
    for line in lines_frozen {
        if *line == 0 {
            layouts.push(String::new());
        } else {
            layouts.push(format!("~{line}"));
        }
    }
    fzf.arg(format!(
        "--bind={binding}:change-preview-window({})",
        layouts.join("|")
    ));
}

fn add_runtime_flag_toggle(
    fzf: &mut Command,
    transform_bin: &str,
    flag: char,
    toggle_on_binding: &str,
    toggle_off_binding: &str,
    input_file_paths: &str,
) {
    fzf.args([
        format!(
            "--bind={toggle_on_binding}:bg-transform:{transform_bin} -f +{flag} -- \
             {input_file_paths}"
        ),
        format!(
            "--bind={toggle_off_binding}:bg-transform:{transform_bin} -f -{flag} -- \
             {input_file_paths}"
        ),
    ]);
}

fn add_external_bindings(
    fzf: &mut Command,
    jq_bin: &str,
    jq_arg_prefix: &str,
    opt: &Opt,
    config: &Config,
    input_file_paths: &str,
) {
    let no_color_flag = &opt.no_color_flag;

    for external in config.external.values() {
        // Extra jq flags (e.g. "-c") are joined and inserted before the no-color flag
        let extra_flags = if external.jq_flags.is_empty() {
            String::new()
        } else {
            format!("{} ", external.jq_flags.join(" "))
        };

        fzf.arg(format!(
            "--bind={}:execute:{jq_bin} {jq_arg_prefix} {extra_flags}{no_color_flag} {{q}} \
             {input_file_paths} | {}",
            external.key, external.command,
        ));
    }
}

fn get_files(positional_files: &[PathBuf]) -> Result<Vec<InputFile<'_>>, Error> {
    let mut files: Vec<InputFile> = vec![];

    let has_piped_input = !std::io::stdin().is_terminal();

    for file_name in positional_files {
        if file_name == "-" {
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
