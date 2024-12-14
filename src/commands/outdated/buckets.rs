use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;
use sprinkles::{buckets::Bucket, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
/// List outdated buckets
pub struct Args {
    #[clap(from_global)]
    pub(super) json: bool,
}

impl super::super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        self.run_direct(ctx, true)?;

        Ok(())
    }
}

impl Args {
    // TODO: Refactor this mess into a traits system
    // TODO: where the is a seperate command trait for those which (can) return data
    // TODO: and those which cant
    // TODO: alongside seperate impls with a where bound where needed
    /// Special function for these subcommands which can be run in different contexts
    ///
    /// Will be called with `is_subcommand` as true when called via clap subcommands,
    /// or with `is_subcommand` as false, when called directly, from the `sfsu outdated` command
    pub fn run_direct(
        self,
        ctx: &impl ScoopContext,
        is_subcommand: bool,
    ) -> Result<Option<Vec<String>>, anyhow::Error> {
        let outdated_buckets = Bucket::list_all(ctx)?
            .into_par_iter()
            .filter(|bucket| match bucket.outdated() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Bucket Error: {e}");
                    false
                }
            })
            .collect::<Vec<_>>();

        if self.json {
            let outdated_bucket_names = outdated_buckets
                .into_iter()
                .map(|bucket| bucket.name().to_string())
                .collect_vec();

            if !is_subcommand {
                return Ok(Some(outdated_bucket_names));
            }

            let output = serde_json::to_string_pretty(&outdated_bucket_names)?;

            println!("{output}");
        } else if outdated_buckets.is_empty() {
            println!("All buckets are up to date!");
        } else {
            for bucket in outdated_buckets {
                println!("❌ `{}` bucket is out of date", bucket.name());
            }
        }

        Ok(None)
    }
}
