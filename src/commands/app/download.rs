use std::time::Duration;

use clap::Parser;

use rayon::prelude::*;

use sprinkles::{
    buckets::Bucket,
    cache::{DownloadHandle, Handle},
    contexts::ScoopContext,
    packages::{
        downloading::Downloader,
        models::install,
        reference::{manifest, package},
    },
    progress::{
        indicatif::{MultiProgress, ProgressBar, ProgressFinish},
        style,
    },
    requests::AsyncClient,
    Architecture,
};

use crate::{
    abandon,
    models::status::Info,
    output::colours::{bright_red, eprintln_yellow},
};

#[derive(Debug, Clone, Parser)]
/// Download the specified app.
pub struct Args {
    #[clap(short, long, help = "Use the specified architecture, if the app supports it", default_value_t = Architecture::ARCH)]
    arch: Architecture,

    #[clap(short = 'H', long, help = "Disable hash validation")]
    no_hash_check: bool,

    #[clap(help = "The packages to download")]
    packages: Vec<package::Reference>,

    #[clap(long, help = "Download new versions of all outdated apps")]
    outdated: bool,
}

impl super::Command for Args {
    const BETA: bool = true;

    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let packages = if self.packages.is_empty() {
            if self.outdated {
                list_outdated(ctx)?
            } else {
                abandon!("No packages provided")
            }
        } else {
            self.packages
        };

        if self.no_hash_check {
            eprintln_yellow!(
                "Hash check has been disabled! This may allow modified files to be downloaded"
            );
        }

        let mp = MultiProgress::new();

        let pb = ProgressBar::new_spinner().with_message("Initializing download(s)");
        pb.enable_steady_tick(Duration::from_millis(100));

        let downloaders: Vec<DownloadHandle> =
            futures::future::try_join_all(packages.into_iter().map(|package| {
                let mp = mp.clone();
                async move {
                    let manifest = match package.manifest(ctx).await {
                        Ok(manifest) => manifest,
                        Err(e) => abandon!("\rFailed to generate manifest: {e}"),
                    };

                    let dl = Handle::open_manifest(ctx.cache_path(), &manifest, self.arch)?;

                    let downloaders = dl.into_iter().map(|dl| {
                        let mp = mp.clone();
                        let package_name = package.name();
                        async move {
                            match DownloadHandle::new::<AsyncClient>(dl, Some(&mp), package_name)
                                .await
                            {
                                Ok(dl) => anyhow::Ok(dl),
                                Err(e) => match e {
                                    sprinkles::cache::Error::ErrorCode(status) => {
                                        abandon!("Found {status} error while downloading")
                                    }
                                    _ => Err(e.into()),
                                },
                            }
                        }
                    });
                    let downloaders = futures::future::try_join_all(downloaders).await?;

                    anyhow::Ok(downloaders)
                }
            }))
            .await?
            .into_iter()
            .flatten()
            .collect();

        pb.finish_with_message("Generated manifests");

        let threads = downloaders
            .into_iter()
            .map(|dl| tokio::spawn(async move { dl.download().await }));

        let results = futures::future::try_join_all(threads).await?;

        let pb = if self.no_hash_check {
            ProgressBar::hidden()
        } else {
            ProgressBar::new(results.len() as u64)
                .with_style(style(None, None))
                .with_finish(ProgressFinish::WithMessage("✅ Checked all files".into()))
        };

        for result in results {
            let result = result?;

            if !self.no_hash_check {
                let actual_hash = result.actual_hash.no_prefix();

                if result.actual_hash == result.computed_hash {
                    pb.tick();
                } else {
                    eprintln!();
                    pb.println(
                        bright_red!(
                            "🔓 Hash mismatch: expected {actual_hash}, found {}",
                            result.computed_hash.no_prefix()
                        )
                        .to_string(),
                    );
                }
            }
        }

        Ok(())
    }
}

fn list_outdated(ctx: &impl ScoopContext) -> Result<Vec<package::Reference>, anyhow::Error> {
    let apps = install::Manifest::list_all_unchecked(ctx)?;

    Ok(apps
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
        .filter(|app| app.current != app.available)
        .map(|app| manifest::Reference::Name(app.name).into_package_ref())
        .collect::<Vec<_>>())
}
