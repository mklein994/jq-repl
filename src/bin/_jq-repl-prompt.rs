use clap::Parser;

/// Transform a prompt for jq-repl.
///
/// This program expects the environment variable `FZF_PROMPT` to be set, e.g. "> ", "-n> ", or
/// "gron> ". The idea is that short flags passed to `jq` are shown chained together, and if a
/// rendering program is run, the name of it is appended.
///
/// If known flags are set, they are shown at the start with a leading '-', e.g. `--null-input`
/// (`-n`) is shown as "-n> ". If a program is set, the program name is added to the prompt, after a
/// space if there are flags (e.g. "braille> ", "-n braille> ").
///
/// There will always be a trailing "> ".
#[derive(Debug, Parser)]
#[command(name = "_jq-repl-prompt", version)]
struct PromptOpts {
    /// Set the prompt string to manipulate
    ///
    /// This is usually set by `fzf`, but in case you are using a different program, or testing, it
    /// can be useful to set this manually.
    #[arg(long, env = "FZF_PROMPT")]
    prompt: String,

    /// Add or remove a flag from the prompt string
    ///
    /// Expects a '+' or '-' character (to add or remove the flag respectively), followed by the
    /// single character to add or remove from the prompt. Some examples: "+n", "-c".
    #[arg(short, allow_hyphen_values = true)]
    flag: Option<String>,

    /// Add or remove the custom program name from the prompt
    ///
    /// Passing only the flag removes the program from the prompt. Some examples: "braille", "gron".
    #[arg(short)]
    #[allow(
        clippy::option_option,
        reason = "clap understands this means both the flag the argument are optional"
    )]
    program: Option<Option<String>>,
}

fn main() {
    let opts = PromptOpts::parse();
    let current_prompt = &opts.prompt;
    let mut prompt = current_prompt.parse::<Prompt>().unwrap();
    prompt.update_from_opts(&opts);

    println!("{prompt}");
}

#[cfg_attr(test, derive(Debug, PartialEq))]
struct Prompt {
    raw: bool,
    null: bool,
    compact: bool,
    program: Option<String>,
}

impl Prompt {
    fn format_flags(&self) -> String {
        let flags = [
            if self.raw { Some("R") } else { None },
            if self.null { Some("n") } else { None },
            if self.compact { Some("c") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect::<String>();

        if flags.is_empty() {
            flags
        } else {
            format!("-{flags}")
        }
    }

    fn update_from_opts(&mut self, opts: &PromptOpts) {
        if let Some(flag_opt) = &opts.flag {
            let [toggle, flag] = flag_opt.chars().collect::<Vec<_>>().try_into().unwrap();
            let on = toggle == '+';
            match flag {
                'R' => {
                    self.raw = on;
                }
                'n' => {
                    self.null = on;
                }
                'c' => {
                    self.compact = on;
                }
                _ => {}
            }
        }

        if let Some(program_flag) = &opts.program {
            self.program.clone_from(program_flag);
        }
    }
}

impl std::fmt::Display for Prompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prompt = vec![];

        let flags = self.format_flags();
        if !flags.is_empty() {
            prompt.push(flags.as_str());
        }

        if let Some(program) = &self.program {
            prompt.push(program.as_str());
        }

        write!(f, "{}> ", prompt.join(" "))
    }
}

impl std::str::FromStr for Prompt {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug_assert!(s.ends_with("> "));
        let prompt = s.trim_end_matches("> ");

        // If it starts with '-', it has flags, otherwise assume the name of the program
        let (flags, program) = if prompt.starts_with('-') {
            let p = prompt.trim_start_matches('-');
            if p.contains(' ') {
                p.split_once(' ')
                    .map(|(flags, program)| (flags, Some(program)))
                    .unwrap()
            } else {
                (p, None)
            }
        } else if prompt.is_empty() {
            ("", None)
        } else {
            ("", Some(prompt))
        };

        Ok(Self {
            raw: flags.contains('R'),
            null: flags.contains('n'),
            compact: flags.contains('c'),
            program: program.map(std::string::ToString::to_string),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_args() {
        <PromptOpts as clap::CommandFactory>::command().debug_assert();
    }

    macro_rules! test_parse {
        ($name:ident, $input:literal, $expected:expr) => {
            #[test]
            fn $name() {
                let input = $input;
                let prompt: Prompt = input.parse().unwrap();
                assert_eq!($expected, prompt);
            }
        };
    }

    macro_rules! test_round_trip {
        ($name:ident, $prompt:literal) => {
            #[test]
            fn $name() {
                let input = $prompt;
                let prompt = input.parse::<Prompt>().unwrap();
                assert_eq!(input, prompt.to_string());
            }
        };
    }

    test_parse!(
        test_raw_null_compact_program,
        "-Rnc braille> ",
        Prompt {
            raw: true,
            null: true,
            compact: true,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_raw_null_compact_not_program,
        "-Rnc> ",
        Prompt {
            raw: true,
            null: true,
            compact: true,
            program: None
        }
    );
    test_parse!(
        test_raw_null_not_compact_program,
        "-Rn braille> ",
        Prompt {
            raw: true,
            null: true,
            compact: false,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_raw_null_not_compact_not_program,
        "-Rn> ",
        Prompt {
            raw: true,
            null: true,
            compact: false,
            program: None
        }
    );
    test_parse!(
        test_raw_not_null_compact_program,
        "-Rc braille> ",
        Prompt {
            raw: true,
            null: false,
            compact: true,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_raw_not_null_compact_not_program,
        "-Rc> ",
        Prompt {
            raw: true,
            null: false,
            compact: true,
            program: None
        }
    );
    test_parse!(
        test_raw_not_null_not_compact_program,
        "-R braille> ",
        Prompt {
            raw: true,
            null: false,
            compact: false,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_raw_not_null_not_compact_not_program,
        "-R> ",
        Prompt {
            raw: true,
            null: false,
            compact: false,
            program: None
        }
    );

    test_parse!(
        test_not_raw_null_compact_program,
        "-nc braille> ",
        Prompt {
            raw: false,
            null: true,
            compact: true,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_not_raw_null_compact_not_program,
        "-nc> ",
        Prompt {
            raw: false,
            null: true,
            compact: true,
            program: None
        }
    );
    test_parse!(
        test_not_raw_null_not_compact_program,
        "-n braille> ",
        Prompt {
            raw: false,
            null: true,
            compact: false,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_not_raw_null_not_compact_not_program,
        "-n> ",
        Prompt {
            raw: false,
            null: true,
            compact: false,
            program: None
        }
    );
    test_parse!(
        test_not_raw_not_null_compact_program,
        "-c braille> ",
        Prompt {
            raw: false,
            null: false,
            compact: true,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_not_raw_not_null_compact_not_program,
        "-c> ",
        Prompt {
            raw: false,
            null: false,
            compact: true,
            program: None
        }
    );
    test_parse!(
        test_not_raw_not_null_not_compact_program,
        "braille> ",
        Prompt {
            raw: false,
            null: false,
            compact: false,
            program: Some("braille".to_string())
        }
    );
    test_parse!(
        test_not_raw_not_null_not_compact_not_program,
        "> ",
        Prompt {
            raw: false,
            null: false,
            compact: false,
            program: None
        }
    );

    test_round_trip!(test_raw_null_compact_program_round_trip, "-Rnc braille> ");
    test_round_trip!(test_raw_null_compact_not_program_round_trip, "-Rnc> ");
    test_round_trip!(
        test_raw_null_not_compact_program_round_trip,
        "-Rn braille> "
    );
    test_round_trip!(test_raw_null_not_compact_not_program_round_trip, "-Rn> ");
    test_round_trip!(
        test_raw_not_null_compact_program_round_trip,
        "-Rc braille> "
    );
    test_round_trip!(test_raw_not_null_compact_not_program_round_trip, "-Rc> ");
    test_round_trip!(
        test_raw_not_null_not_compact_program_round_trip,
        "-R braille> "
    );
    test_round_trip!(test_raw_not_null_not_compact_not_program_round_trip, "-R> ");

    test_round_trip!(
        test_not_raw_null_compact_program_round_trip,
        "-nc braille> "
    );
    test_round_trip!(test_not_raw_null_compact_not_program_round_trip, "-nc> ");
    test_round_trip!(
        test_not_raw_null_not_compact_program_round_trip,
        "-n braille> "
    );
    test_round_trip!(test_not_raw_null_not_compact_not_program_round_trip, "-n> ");
    test_round_trip!(
        test_not_raw_not_null_compact_program_round_trip,
        "-c braille> "
    );
    test_round_trip!(test_not_raw_not_null_compact_not_program_round_trip, "-c> ");
    test_round_trip!(
        test_not_raw_not_null_not_compact_program_round_trip,
        "braille> "
    );
    test_round_trip!(
        test_not_raw_not_null_not_compact_not_program_round_trip,
        "> "
    );
}
