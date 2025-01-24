use std::borrow::Cow;

use anyhow::Context;
use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;

use sprinkles::{
    buckets::{self, Bucket},
    config::Scoop as ScoopConfig,
    contexts::ScoopContext,
    git::{self, Repo},
    progress::{
        indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle},
        style, Message, ProgressOptions,
    },
};

use crate::output::sectioned::{Children, Section};

#[derive(Debug, Clone, Parser)]
/// Update Scoop and Scoop buckets
pub struct Args {
    #[clap(short, long, help = "Show commit messages for each update")]
    changelog: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));

        let buckets = Bucket::list_all(ctx)?;

        let longest_bucket_name = buckets
            .iter()
            .map(|bucket| bucket.name().len())
            .max()
            .unwrap_or(0);

        // Force checkout to the config's branch
        _ = ctx.outdated().await?;

        let scoop_changelog =
            self.update_scoop(ctx, longest_bucket_name, progress_style.clone())?;

        let mp = MultiProgress::new();

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = mp.add(
                    ProgressBar::new(1)
                        .with_style(progress_style.clone())
                        .with_message("Checking updates")
                        .with_prefix(format!("🪣 {:<longest_bucket_name$}", bucket.name()))
                        .with_finish(ProgressFinish::WithMessage(Self::FINISH_MESSAGE.into())),
                );

                pb.set_position(0);

                (bucket, pb)
            })
            .collect_vec();

        let bucket_changelogs = self.update_buckets(ctx, &outdated_buckets)?;

        let mut scoop_config = ScoopConfig::load()?;
        scoop_config.update_last_update_time();
        scoop_config.save()?;

        if self.changelog {
            println!();
            if let Some(scoop_changelog) = scoop_changelog {
                let scoop_changelog =
                    Section::new(Children::from(scoop_changelog)).with_title("Scoop changes:");

                print!("{scoop_changelog}");
            };

            for bucket_changelog in bucket_changelogs {
                let (name, changelog) = bucket_changelog;

                if changelog.is_empty() {
                    continue;
                }

                let changelog =
                    Section::new(Children::from(changelog)).with_title(format!("{name} changes:"));

                println!("{changelog}");
            }
        }

        Ok(())
    }
}

impl Args {
    const FINISH_MESSAGE: &'static str = "✅";

    fn update_scoop(
        &self,
        ctx: &impl ScoopContext,
        longest_bucket_name: usize,
        style: ProgressStyle,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let repo = ctx.open_repo().context("missing user repository")??;

        let pb = ProgressBar::new(1)
            .with_style(style)
            .with_message("Checking for updates")
            .with_prefix(format!("🍨 {:<longest_bucket_name$}", "Scoop"))
            .with_finish(ProgressFinish::WithMessage(Self::FINISH_MESSAGE.into()));

        let changelog = self.update(ctx, &repo, &pb)?;

        Ok(changelog)
    }

    fn update_buckets<'a>(
        &self,
        ctx: &impl ScoopContext,
        outdated_buckets: &'a [(Bucket, ProgressBar)],
    ) -> anyhow::Result<Vec<(Cow<'a, str>, Vec<String>)>> {
        let bucket_changelogs = outdated_buckets
            .par_iter()
            .map(|(bucket, pb)| -> buckets::Result<_> {
                let repo = bucket.open_repo()?;

                let changelog = self.update(ctx, &repo, pb)?;

                Ok((bucket.name(), changelog.unwrap_or_default()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bucket_changelogs)
    }

    fn update(
        &self,
        ctx: &impl ScoopContext,
        repo: &Repo,
        pb: &ProgressBar,
    ) -> git::Result<Option<Vec<String>>> {
        if !repo.outdated()? {
            pb.finish_with_message("✅ No updates available");
            return Ok(None);
        }

        let changelog = if self.changelog {
            repo.pull_with_changelog(ctx, Some(&Self::gen_stats_callback(pb)))?
        } else {
            repo.pull(ctx, Some(&Self::gen_stats_callback(pb)))?;

            vec![]
        };

        pb.finish_with_message(Self::FINISH_MESSAGE);

        Ok(Some(changelog))
    }

    fn gen_stats_callback(
        pb: &ProgressBar,
    ) -> impl Fn(sprinkles::git::implementations::git2::Progress<'_>, bool) -> bool + '_ {
        |stats, thin| {
            if thin {
                pb.set_position(stats.indexed_objects() as u64);
                pb.set_length(stats.total_objects() as u64);
            } else if stats.received_objects() == stats.total_objects() {
                pb.set_position(stats.indexed_deltas() as u64);
                pb.set_length(stats.total_deltas() as u64);
                pb.set_message("Resolving deltas");
            } else if stats.total_objects() > 0 {
                pb.set_position(stats.received_objects() as u64);
                pb.set_length(stats.total_objects() as u64);
                pb.set_message("Receiving objects");
            }

            true
        }
    }
}
