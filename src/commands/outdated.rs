use clap::{Parser, Subcommand};
use serde_json::Map;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner, DeprecationMessage, DeprecationWarning, Runnable};

pub mod apps;
pub mod buckets;

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Apps(apps::Args),
    Buckets(buckets::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::Apps(args) => args.run(ctx).await,
            Commands::Buckets(args) => args.run(ctx).await,
        }
    }
}

#[derive(Debug, Clone, Parser)]
/// List outdated buckets and/or packages
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(from_global)]
    json: bool,
}

impl Command for Args {
    const DEPRECATED: Option<DeprecationWarning> = Some(DeprecationWarning {
        message: DeprecationMessage::Replacement("sfsu status"),
        version: Some(2.0),
    });

    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()> {
        if let Some(command) = self.command {
            command.run(ctx).await
        } else {
            if self.json {
                let mut map = Map::new();

                let apps = apps::Args { json: self.json }
                    .run_direct(ctx, false)?
                    .unwrap_or_default();

                let buckets = buckets::Args { json: self.json }
                    .run_direct(ctx, false)?
                    .unwrap_or_default();

                map.insert("outdated_apps".into(), apps.into());
                map.insert("outdated_buckets".into(), buckets.into());

                let output = serde_json::to_string_pretty(&map)?;

                println!("{output}");
            } else {
                println!("Outdated Apps:");
                Commands::Apps(apps::Args { json: self.json })
                    .run(ctx)
                    .await?;
                println!("\nOutdated Buckets:");
                Commands::Buckets(buckets::Args { json: self.json })
                    .run(ctx)
                    .await?;
            }

            Ok(())
        }
    }
}
