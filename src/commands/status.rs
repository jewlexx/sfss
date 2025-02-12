use std::{fmt::Write, sync::Arc};

use clap::{Parser, ValueEnum};
use parking_lot::Mutex;
use quork::prelude::*;
use rayon::prelude::*;
use serde_json::Value;

use sprinkles::{
    buckets::Bucket,
    contexts::ScoopContext,
    packages::models::install,
    progress::{indicatif::ProgressBar, style},
};

use crate::{
    models::status::Info,
    output::{
        sectioned::{Children, Section},
        structured::Structured,
    },
};

#[derive(Debug, Copy, Clone, ValueEnum, ListVariants)]
enum Command {
    Scoop,
    Buckets,
    Apps,
}

#[derive(Debug, Clone, Parser)]
/// Show status and check for new app versions
pub struct Args {
    #[clap(from_global)]
    json: bool,

    #[clap(from_global)]
    verbose: bool,

    #[clap(short = 'O', long, help = "Only check the provided sections of Scoop")]
    only: Vec<Command>,

    #[clap(short = 'H', long, help = "Ignore held packages")]
    ignore_held: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let value = Arc::new(Mutex::new(Value::default()));

        let pb = ProgressBar::new(3).with_style(style(None, None));

        let commands: &[Command] = {
            if self.only.is_empty() {
                &Command::VARIANTS
            } else {
                &self.only
            }
        };

        let outputs = commands.iter().map(|command| {
            let command = *command;
            let this = self.clone();
            let pb = pb.clone();
            let value = value.clone();
            async move {
                let mut output = String::new();

                match command {
                    Command::Scoop => this.handle_scoop(ctx, &value, &mut output).await?,
                    Command::Buckets => this.handle_buckets(ctx, &value, &mut output)?,
                    Command::Apps => this.handle_packages(ctx, &value, &mut output)?,
                };

                pb.inc(1);

                anyhow::Ok(output)
            }
        });

        let outputs = futures::future::try_join_all(outputs).await?;

        pb.finish_and_clear();

        if self.json {
            let output = serde_json::to_string(&value.lock().clone())?;
            println!("{output}");
        } else {
            for output in outputs {
                print!("{output}");
            }
        }

        Ok(())
    }
}

impl Args {
    async fn handle_scoop(
        &self,
        ctx: &impl ScoopContext,
        value: &Mutex<Value>,
        output: &mut dyn Write,
    ) -> anyhow::Result<()> {
        let is_outdated = ctx.outdated().await?;

        if self.json {
            value.lock()["scoop"] = serde_json::to_value(is_outdated)?;
            return Ok(());
        } else if is_outdated {
            writeln!(
                output,
                "{}",
                console::style(
                    "Scoop is out of date. Run `scoop update` to get the latest changes."
                )
                .yellow()
            )?;
        } else {
            writeln!(output, "Scoop app is up to date.")?;
        }

        Ok(())
    }

    fn handle_buckets(
        &self,
        ctx: &impl ScoopContext,
        value: &Mutex<Value>,
        output: &mut dyn Write,
    ) -> anyhow::Result<()> {
        let buckets = Bucket::list_all(ctx)?;

        // Handle buckets
        if self.verbose || self.json {
            let outdated_buckets = buckets
                .par_iter()
                .filter_map(|bucket| {
                    bucket.outdated().ok().and_then(|outdated| {
                        if outdated {
                            Some(bucket.name().to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();

            if self.json {
                value.lock()["buckets"] = serde_json::to_value(&outdated_buckets)?;
                return Ok(());
            }

            if outdated_buckets.is_empty() {
                writeln!(output, "All buckets up to date.")?;
            } else {
                let title = format!("{} outdated buckets:", outdated_buckets.len());

                let section = Section::new(Children::from(outdated_buckets)).with_title(title);

                writeln!(output, "{section}")?;
            }
        } else {
            let buckets_outdated = buckets.par_iter().any(|bucket| {
                bucket.outdated().unwrap_or_else(|err| {
                    eprintln!("Failed to check bucket: {}", bucket.name());
                    error!(
                        "Failed to check bucket: {}. Threw Error: {:?}",
                        bucket.name(),
                        err
                    );
                    false
                })
            });

            if buckets_outdated {
                writeln!(
                    output,
                    "{}",
                    console::style(
                        "Bucket(s) are out of date. Run `scoop update` to get the latest changes."
                    )
                    .yellow()
                )?;
            } else {
                writeln!(output, "All buckets up to date.")?;
            }
        }

        Ok(())
    }

    fn handle_packages(
        &self,
        ctx: &impl ScoopContext,
        value: &Mutex<Value>,
        output: &mut dyn Write,
    ) -> anyhow::Result<()> {
        let apps = install::Manifest::list_all_unchecked(ctx)?;

        debug!("Checking {} apps", apps.len());

        let mut invalid_apps = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<Info> {
                if let Some(bucket) = &app.bucket {
                    let local_manifest = app.get_manifest(ctx)?;
                    // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                    let bucket = Bucket::from_name(ctx, bucket)?;

                    match Info::from_manifests(ctx, &local_manifest, &bucket) {
                        Ok(info) => Ok(info),
                        Err(err) => {
                            error!(
                                "Failed to get status for {}: {:?}",
                                unsafe { app.name() },
                                err
                            );
                            Err(err)?
                        }
                    }
                } else {
                    error!("no bucket specified");
                    anyhow::bail!("no bucket specified")
                }
            })
            .filter(|app| {
                let missing_deps = !app.missing_dependencies.is_empty();

                let info_exists = if let Some(ref info) = app.info {
                    // Ignore held packages if the flag is specified and there are no other reasons to show it
                    if !missing_deps && info == "Held package" && self.ignore_held {
                        return false;
                    }
                    true
                } else {
                    false
                };

                // Filter out apps that are okay
                info_exists || missing_deps || app.current != app.available
            })
            .collect::<Vec<_>>();

        invalid_apps.dedup();

        if self.json {
            value.lock()["packages"] = serde_json::to_value(&invalid_apps)?;
            return Ok(());
        }

        if invalid_apps.is_empty() {
            writeln!(output, "All packages are okay and up to date.")?;
        } else {
            invalid_apps.par_sort_by(|a, b| a.name.cmp(&b.name));

            let values = invalid_apps
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs = Structured::new(&values).with_max_length(30);

            write!(output, "{outputs}")?;
        }

        Ok(())
    }
}
