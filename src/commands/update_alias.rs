use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

use crate::output::colours::eprintln_yellow;

use super::bucket;

#[derive(Debug, Clone, Parser)]
/// Update Scoop and Scoop buckets
pub struct ArgsWrapper {
    #[clap(flatten)]
    args: bucket::update::Args,
}

impl super::Command for ArgsWrapper {
    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()> {
        eprintln_yellow!("Updating buckets has been renamed to `bucket update`. Updating apps is not yet supported and will be added in a future release.");
        bucket::update::Args::runner(self.args, ctx).await
    }
}
