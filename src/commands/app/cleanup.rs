use std::{path::Path, str::FromStr, time::Duration};

use clap::Parser;
use futures::{StreamExt, TryFutureExt, stream::FuturesUnordered};
use itertools::Itertools;
use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
    progress::{
        Message,
        indicatif::{MultiProgress, ProgressBar},
        style,
    },
    version::Version,
};

use crate::{
    abandon,
    handlers::{AppsDecider, ListApps},
    logging::macros::ddbg,
    output::colours::eprintln_green,
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

        let mp = MultiProgress::new();

        let cleanup_tasks = cleanup_apps
            .iter()
            .map(|reference| {
                self.cleanup_app(ctx, reference, mp.clone())
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Failed to cleanup {}: {error}",
                            match &reference.manifest {
                                manifest::Reference::File(path_buf) =>
                                    path_buf.display().to_string(),
                                manifest::Reference::BucketNamePair { bucket, name } =>
                                    format!("{bucket}/{name}"),
                                manifest::Reference::Name(name) => name.clone(),
                                manifest::Reference::Url(url) => url.to_string(),
                            }
                        )
                    })
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        for result in cleanup_tasks {
            if let Err(error) = result {
                error!("{error}");
            }
        }

        eprintln_green!("All squeaky clean!!");

        Ok(())
    }
}

impl Args {
    fn list_apps<C: ScoopContext>(&self) -> ListApps<C> {
        let all = self.all;
        std::rc::Rc::new(move |ctx: &C| {
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
        })
    }

    async fn cleanup_app(
        &self,
        ctx: &impl ScoopContext,
        app: &package::Reference,
        mp: MultiProgress,
    ) -> anyhow::Result<()> {
        let Ok(app_handle) = app.clone().open_handle(ctx).await else {
            return Ok(());
        };

        let current_version = app_handle.local_version()?;

        let versions = app_handle.list_versions()?;

        let old_versions = versions
            .into_iter()
            .filter(|version| version.version() != current_version.as_str())
            .collect_vec();

        // Remove old cache entries
        if self.cache {
            let cache_path = ctx.cache_path();
            let cache_entries = std::fs::read_dir(&cache_path)?
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let Ok(cache_entry) = CacheEntry::parse_path(ddbg!(entry.path())) else {
                        return None;
                    };
                    if Some(&cache_entry.name) == app.name().as_ref()
                        && cache_entry.version != current_version
                    {
                        Some((entry.path(), cache_entry))
                    } else {
                        None
                    }
                })
                .collect_vec();

            if cache_entries.is_empty() {
                debug!("No matching cache entries found");
            } else {
                let pb = mp.add(ProgressBar::new(cache_entries.len() as u64));
                pb.enable_steady_tick(Duration::from_millis(100));

                pb.set_style(style(
                    None,
                    Some(Message::prefix().with_message(&format!("Cleaning up {app} cache"))),
                ));

                for (path, cache_entry) in cache_entries {
                    debug!(
                        "Found matching outdated cache entry: {}",
                        cache_entry.url_hash
                    );

                    if self.dry_run {
                        debug!("Would remove cache entry: {}", cache_entry.url_hash);
                    } else {
                        std::fs::remove_file(ddbg!(path))?;
                    }

                    debug!("Removed cache entry: {}", cache_entry.url_hash);
                    pb.inc(1);
                }

                debug!("Cleaned up old cache entries");

                pb.finish_with_message(format!("Cleaned up old cache entries for {app}"));
            }
        }

        if old_versions.is_empty() {
            debug!("No matching versions found");
        } else {
            let pb = mp.add(ProgressBar::new(old_versions.len() as u64));
            pb.enable_steady_tick(Duration::from_millis(100));

            pb.set_style(style(
                None,
                Some(Message::prefix().with_message(&format!("Cleaning up {app} versions"))),
            ));

            for version in old_versions {
                debug!("Cleaning up {app}@{}", version.version());
                if self.dry_run {
                    debug!(
                        "Would remove old version directory: {}",
                        version.path().display()
                    );
                } else {
                    std::fs::remove_dir_all(version.path())?;
                }
                pb.inc(1);
            }

            pb.finish_with_message(format!("Cleaned up old versions for {app}"));
        }

        Ok(())
    }
}

// TODO: Move this into the sprinkles crate
#[derive(Debug)]
pub struct CacheEntry {
    name: String,
    version: Version,
    url_hash: UrlHash,
}

#[derive(Debug)]
enum UrlHash {
    Valid([char; 7]),
    Invalid(String),
}

impl std::fmt::Display for UrlHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrlHash::Valid(chars) => {
                let hash_string = chars.iter().collect::<String>();
                std::fmt::Display::fmt(&hash_string, f)
            }
            UrlHash::Invalid(hash_string) => std::fmt::Display::fmt(hash_string, f),
        }
    }
}

impl FromStr for CacheEntry {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_path(s)
    }
}

impl CacheEntry {
    pub fn parse_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file_name = path.as_ref();

        let name = file_name
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("no file stem"))?
            .to_string_lossy();

        let mut parts = name.split('#');

        let name = parts.next().ok_or_else(|| anyhow::anyhow!("no name"))?;
        let version = parts.next().ok_or_else(|| anyhow::anyhow!("no version"))?;
        let hash = parts.next().ok_or_else(|| anyhow::anyhow!("no hash"))?;

        Ok(Self {
            name: name.to_string(),
            version: Version::new(version),
            url_hash: {
                let hash_chars = hash.chars().collect::<Vec<_>>();

                if hash_chars.len() == 7 {
                    UrlHash::Valid(
                        hash_chars
                            .try_into()
                            .expect("Valid length of 7. This is a bug"),
                    )
                } else {
                    UrlHash::Invalid(hash_chars.into_iter().collect())
                }
            },
        })
    }
}
