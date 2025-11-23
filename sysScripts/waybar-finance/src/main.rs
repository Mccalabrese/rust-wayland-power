use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    tui: bool,
}

fn main() -> Result<()> {

    let args = Args::parse();

    if args.tui {
        println!("Initializing TUI mode...");
    } else {
        println!("Outputting JSON for waybar...")
    }
    Ok(())
}
