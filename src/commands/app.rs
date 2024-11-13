pub mod cat;
pub mod cleanup;
#[cfg(feature = "download")]
pub mod download;
pub mod home;
pub mod info;
pub mod list;
pub mod purge;

use clap::{Parser, Subcommand};

use sfsu_macros::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    Cat(cat::Args),
    Cleanup(cleanup::Args),
    #[cfg(feature = "download")]
    Download(download::Args),
    Home(home::Args),
    Info(info::Args),
    List(list::Args),
    Purge(purge::Args),
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
