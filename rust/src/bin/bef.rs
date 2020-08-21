use std::fs::File;
use std::path::PathBuf;

use ansi_term::Colour::Green;
use anyhow::{anyhow, Context, Result};
use structopt::StructOpt;

use befunge_93::Interpreter;

#[derive(StructOpt)]
#[structopt(name = "bef", author, about = "A simple Befunge-93 interpreter.")]
struct Opts {
    #[structopt(short, long)]
    /// Path to program file.
    file: PathBuf,
    #[structopt(short, long)]
    /// Print the playfield at each step.
    playfield: bool,
    #[structopt(short, long)]
    /// Print the stack at each step.
    stack: bool,
    #[structopt(short, long)]
    /// Execute in trace mode.
    trace: bool,
    #[structopt(short, long)]
    /// Delay between steps (in milliseconds).
    delay: Option<u16>,
    #[structopt(long)]
    /// Run in debug mode; press enter to step.
    debug: bool,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let mut file = File::open(&opts.file)
        .with_context(|| anyhow!("Failed to open '{}'", opts.file.display()))?;

    let mut interpreter = Interpreter::new();

    interpreter
        .load(&mut file)
        .context("Failed to load program from stdin")?;

    println!("Loaded:\n{}", interpreter.to_string());

    println!("Running program...");
    interpreter
        .run(|int, iter_n| {
            if opts.trace {
                println!(
                    "[{}] Executing: {:?}\nStack: {}\nOutput: {}\n{}",
                    iter_n,
                    int.get_current_command(),
                    int.get_stack().to_string(),
                    int.get_output(),
                    String::from("-").repeat(60)
                );

                if opts.debug {
                    let mut s = String::new();
                    std::io::stdin().read_line(&mut s).unwrap();
                }

                return true;
            }

            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

            if opts.playfield {
                println!("{}\n{}", Green.paint("Playfield:"), int.to_string());
            }

            if opts.stack {
                println!("{} {}", Green.paint("Stack:"), int.get_stack().to_string());
            }

            print!("{}\n{}", Green.paint("Output:"), int.get_output());

            if opts.debug {
                let mut s = String::new();
                std::io::stdin().read_line(&mut s).unwrap();
            }

            if let Some(delay) = opts.delay {
                std::thread::sleep(std::time::Duration::from_millis(delay.into()));
            }

            true
        })
        .with_context(|| anyhow!("Failed to run the program:\n{}", interpreter.to_string()))?;

    Ok(())
}
