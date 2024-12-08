use clap::Parser;
use futures::{stream::FuturesUnordered, TryFutureExt, TryStreamExt};
use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
};

use crate::{
    abandon,
    handlers::{AppsDecider, ListApps},
};

#[derive(Debug, Clone, Parser)]
#[allow(clippy::struct_excessive_bools)]
/// Cleanup apps by removing old versions
pub struct Args {
    #[clap(help = "The app(s) to cleanup")]
    apps: Vec<package::Reference>,

    #[clap(short, long, help = "Cleanup all installed apps")]
    all: bool,

    #[clap(
        short = 'k',
        long,
        help = "Cleanup old versions of the app from the cache"
    )]
    cache: bool,

    #[clap(from_global)]
    assume_yes: bool,

    #[clap(
        long,
        help = "Print what would be done, but don't actually do anything"
    )]
    dry_run: bool,
}

impl super::Command for Args {
    async fn runner(mut self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let provided_apps = std::mem::take(&mut self.apps);

        let cleanup_apps = match AppsDecider::new(ctx, self.list_apps(), provided_apps).decide()? {
            Some(apps) if apps.is_empty() => abandon!("No apps selected"),
            None => abandon!("No apps selected"),
            Some(apps) => apps,
        };

        let cleanup_tasks = cleanup_apps
            .iter()
            .map(|reference| {
                self.cleanup_app(ctx, reference).map_err(|error| {
                    anyhow::anyhow!(
                        "Failed to cleanup {}: {error}",
                        match &reference.manifest {
                            manifest::Reference::File(path_buf) => path_buf.display().to_string(),
                            _ => unreachable!(),
                        }
                    )
                })
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        unimplemented!()
    }
}

impl Args {
    fn list_apps<C: ScoopContext>(&self) -> impl ListApps<C> + use<C> {
        let all = self.all;
        move |ctx: &C| {
            if all {
                let installed_apps: Vec<package::Reference> = {
                    let installed_apps = ctx.installed_apps()?;
                    let manifest_paths = installed_apps.into_iter().filter_map(|path| {
                        let manifest_path = path.join("current").join("manifest.json");

                        manifest_path
                            .try_exists()
                            .ok()
                            .and_then(|exists| exists.then_some(manifest_path))
                    });

                    let references = manifest_paths
                        .map(|path| manifest::Reference::File(path).into_package_ref());

                    references.collect()
                };

                anyhow::Ok(Some(installed_apps))
            } else {
                anyhow::Ok(None)
            }
        }
    }

    async fn cleanup_app(
        &self,
        ctx: &impl ScoopContext,
        app: &package::Reference,
    ) -> anyhow::Result<()> {
        let app_handle = app.clone().open_handle(ctx);

        let current_version = app_handle.await?.local_manifest().map(|manifest| {
            debug!("Cleaning up {app}@{}", manifest.version);
            manifest.version
        })?;

        if self.cache {
            let cache_path = ctx.cache_path();

            while let Some(entry) = tokio::fs::read_dir(&cache_path).await?.next_entry().await? {
                let path = dbg!(entry.file_name());
            }
        }

        /**
         *     if ($cache) {
            Remove-Item "$cachedir\$app#*" -Exclude "$app#$current_version#*"
        }
         */
        unimplemented!()
    }
}
