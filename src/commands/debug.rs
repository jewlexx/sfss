use clap::{Parser, Subcommand};
use sfsu_macros::Hooks;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner, Runnable};

mod save;

#[derive(Debug, Hooks, Clone, Subcommand)]
pub enum Commands {
    Save(save::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::Save(args) => args.run(ctx).await,
        }
    }
}
#[derive(Debug, Clone, Parser)]
/// Debugging commands
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()> {
        self.command.run(ctx).await
    }
}
