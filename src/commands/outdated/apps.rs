use clap::Parser;
use rayon::prelude::*;
use serde_json::Value;
use sprinkles::{buckets::Bucket, contexts::ScoopContext, packages::models::install};

use crate::{models::outdated::Info, output::structured::Structured};

#[derive(Debug, Clone, Parser)]
/// List outdated apps
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
    /// Special function for these subcommands which can be run in different contexts
    ///
    /// Will be called with `is_subcommand` as true when called via clap subcommands,
    /// or with `is_subcommand` as false, when called directly, from the `sfsu outdated` command
    pub fn run_direct(
        self,
        ctx: &impl ScoopContext,
        is_subcommand: bool,
    ) -> Result<Option<Vec<Value>>, anyhow::Error> {
        let apps = install::Manifest::list_all_unchecked(ctx)?;

        let mut outdated: Vec<Info> = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<Info> {
                if let Some(bucket) = &app.bucket {
                    let local_manifest = app.get_manifest(ctx)?;
                    // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                    let bucket = Bucket::from_name(ctx, bucket)?;

                    let remote_manifest = bucket.get_manifest(unsafe { app.name() })?;

                    match Info::from_manifests(&local_manifest, &remote_manifest) {
                        Some(info) => Ok(info),
                        None => anyhow::bail!("no update available"),
                    }
                } else {
                    anyhow::bail!("no bucket specified")
                }
            })
            .collect();

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
            outdated.dedup();
            outdated.par_sort_by(|a, b| a.name.cmp(&b.name));

            let values = outdated
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            if self.json {
                if !is_subcommand {
                    return Ok(Some(values));
                }

                let output = serde_json::to_string_pretty(&values)?;

                println!("{output}");
            } else {
                // TODO: Add a better way to add colours than this
                // TODO: p.s this doesnt work atm
                // use owo_colors::OwoColorize;
                // let values = values
                //     .into_par_iter()
                //     .map(|mut value| {
                //         if let Some(current) = value.get_mut("Current") {
                //             *current = current.as_str().unwrap().red().to_string().into();
                //         }

                //         if let Some(available) = value.get_mut("Available") {
                //             *available = available.as_str().unwrap().green().to_string().into();
                //         }

                //         value
                //     })
                //     .collect::<Vec<_>>();

                let outputs = Structured::new(&values).with_max_length(30);

                print!("{outputs}");
            }
        }

        Ok(None)
    }
}
