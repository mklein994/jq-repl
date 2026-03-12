use clap::Parser;

/// Transform a prompt for jq-repl.
///
/// This program expects the environment variable `FZF_PROMPT` to be set, e.g. "-> ", "- gron> ".
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

struct Prompt {
    null: bool,
    compact: bool,
    program: Option<String>,
}

impl Prompt {
    fn format_flags(&self) -> String {
        format!(
            "-{}{}",
            if self.null { "n" } else { "" },
            if self.compact { "c" } else { "" }
        )
    }

    fn update_from_opts(&mut self, opts: &PromptOpts) {
        if let Some(flag_opt) = &opts.flag {
            let [toggle, flag] = flag_opt.chars().collect::<Vec<_>>().try_into().unwrap();
            let on = toggle == '+';
            match flag {
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
        let flags = self.format_flags();
        let mut prompt = vec![flags.as_str()];

        if let Some(program) = &self.program {
            prompt.push(program.as_str());
        }

        write!(f, "{}> ", prompt.join(" "))
    }
}

impl std::str::FromStr for Prompt {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let p = s.trim_end_matches("> ");
        let (flags, program) = p
            .split_once(' ')
            .map_or((p, None), |(flags, program)| (flags, Some(program)));

        Ok(Self {
            null: flags.contains('n'),
            compact: flags.contains('c'),
            program: program.map(std::string::ToString::to_string),
        })
    }
}
