pub mod cat;
#[cfg(feature = "download")]
pub mod download;
pub mod home;
pub mod info;
pub mod list;
pub mod purge;

use clap::{Parser, Subcommand};

use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner, Runnable};

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Cat(cat::Args),
    #[cfg(feature = "download")]
    Download(download::Args),
    Home(home::Args),
    Info(info::Args),
    List(list::Args),
    Purge(purge::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::Cat(args) => args.run(ctx).await,
            Commands::Download(args) => args.run(ctx).await,
            Commands::Home(args) => args.run(ctx).await,
            Commands::Info(args) => args.run(ctx).await,
            Commands::List(args) => args.run(ctx).await,
            Commands::Purge(args) => args.run(ctx).await,
        }
    }
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing apps
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl Command for Args {
    #[inline]
    async fn runner(
        self,
        ctx: &impl ScoopContext<Config = config::Scoop>,
    ) -> Result<(), anyhow::Error> {
        self.command.run(ctx).await
    }
}
