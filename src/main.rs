use clap::Parser;

use sfsu::commands::*;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    args.command.run()
}
