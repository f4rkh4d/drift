use clap::Parser;
use drift::cli::{run, Cli};

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("drift: {e:#}");
            std::process::exit(2);
        }
    }
}
