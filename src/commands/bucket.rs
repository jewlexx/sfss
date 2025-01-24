pub mod add;
pub mod known;
pub mod list;
pub mod outdated;
pub mod remove;
pub mod unused;
pub mod update;

use clap::{Parser, Subcommand};

use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner, Runnable};

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Add(add::Args),
    #[clap(alias = "rm")]
    Remove(remove::Args),
    List(list::Args),
    Known(known::Args),
    Unused(unused::Args),
    #[cfg(not(feature = "v2"))]
    Outdated(outdated::Args),
    Update(update::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::Add(args) => args.run(ctx).await,
            Commands::Remove(args) => args.run(ctx).await,
            Commands::List(args) => args.run(ctx).await,
            Commands::Known(args) => args.run(ctx).await,
            Commands::Unused(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::Outdated(args) => args.run(ctx).await,
            Commands::Update(args) => args.run(ctx).await,
        }
    }
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing buckets
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    #[inline]
    async fn runner(
        self,
        ctx: &impl ScoopContext<Config = config::Scoop>,
    ) -> Result<(), anyhow::Error> {
        self.command.run(ctx).await
    }
}
