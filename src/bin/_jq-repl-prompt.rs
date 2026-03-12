use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "_jq-repl-prompt", version)]
struct PromptOpts {
    #[arg(short, allow_hyphen_values = true)]
    flag: Option<String>,
    #[arg(short)]
    program: Option<Option<String>>,
}

fn main() {
    let opts = PromptOpts::parse();
    let current_prompt = std::env::var("FZF_PROMPT").unwrap();
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
            };
        }

        if let Some(program_flag) = &opts.program {
            self.program = program_flag.clone();
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
        let (flags, program) = p.split_once(" ").unwrap();

        Ok(Self {
            null: flags.contains('n'),
            compact: flags.contains('c'),
            program: if program.is_empty() {
                None
            } else {
                Some(program.to_string())
            },
        })
    }
}
