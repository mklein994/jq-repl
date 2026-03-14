#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Prompt {
    raw: bool,
    null: bool,
    compact: bool,
    program: Option<String>,
}

impl Prompt {
    #[must_use]
    pub fn new(raw: bool, null: bool) -> Self {
        Self {
            raw,
            null,
            compact: false,
            program: None,
        }
    }

    #[must_use]
    pub fn compact(&self) -> bool {
        self.compact
    }

    #[must_use]
    pub fn program(&self) -> Option<&str> {
        self.program.as_deref()
    }

    /// Returns the flags that should be passed to jq, derived from the current prompt state.
    ///
    /// This is distinct from the prompt display flags: it omits flags like `n` (null-input) and `R`
    /// (raw-input) that are set once at startup and already present in `jq_arg_prefix`, and only
    /// includes toggleable runtime flags like `c` (compact).
    #[must_use]
    pub fn jq_flags(&self) -> String {
        let flags: String = [if self.compact { Some("c") } else { None }]
            .into_iter()
            .flatten()
            .collect();

        if flags.is_empty() {
            flags
        } else {
            format!("-{flags}")
        }
    }

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

    /// Transform the prompt string by adding or removing flags or a program name
    ///
    /// If a parameter is present, that value is changed as follows:
    ///
    ///   - If a flag starts with a '+', it's added, otherwise removed.
    ///   - If a program string is present, it's added to the prompt string, otherwise removed.
    pub fn transform(&mut self, flag: Option<String>, program: Option<Option<String>>) {
        if let Some(flag_opt) = flag {
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

        if let Some(program_flag) = program {
            self.program.clone_from(&program_flag);
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

        // If it starts with '-', it has flags, otherwise assume the name of a program that renders
        // the output
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
