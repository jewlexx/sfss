use std::time::Duration;

use anyhow::Context;
use clap::{Parser, ValueEnum};
use rayon::prelude::*;
use sprinkles::{
    Architecture, config,
    contexts::ScoopContext,
    hash::Hash,
    packages::{CreateManifest, Manifest, reference::package},
    progress::{ProgressOptions, indicatif::ProgressBar, style},
    requests::USER_AGENT,
};

use crate::{
    calm_panic::CalmUnwrap,
    errors::{RecoverableError, RecoverableResult},
    limits::RateLimiter,
    output::colours::{eprintln_green, eprintln_red, eprintln_yellow},
};

impl RecoverableError for vt3::error::VtError {
    fn recoverable(&self) -> bool {
        matches!(
            self,
            vt3::error::VtError::TransientError
                | vt3::error::VtError::AlreadyExistsError
                | vt3::error::VtError::NotFoundError
                | vt3::error::VtError::NotAvailableYet
        )
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Undetected,
    Suspicious,
    Malicious,
}

impl Status {
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    pub fn from_stats(dangerous: u64, total: u64) -> Self {
        let dangerous = dangerous as f64;
        let total = total as f64;
        let ratio = dangerous / total;

        if ratio > 0.1 {
            Self::Malicious
        } else if dangerous > 0.0 {
            Self::Suspicious
        } else {
            Self::Undetected
        }
    }
}

#[derive(Debug, Clone)]
enum SearchType {
    FileHash(Hash),
    Url(String),
}

#[derive(Debug, Clone)]
struct StrippedManifest {
    name: String,
    bucket: String,
    search_type: SearchType,
}

impl StrippedManifest {
    fn new(manifest: &Manifest, search_type: SearchType) -> Self {
        Self {
            name: unsafe { manifest.name() }.to_string(),
            bucket: unsafe { manifest.bucket() }.to_string(),
            search_type,
        }
    }
}

/// Value should be a `Root` object
fn extract_info(value: &serde_json::Value) -> anyhow::Result<(u64, u64)> {
    let stats = &value["data"]["attributes"]["last_analysis_stats"];
    let detected = stats["malicious"].as_u64().context("no malicious")?
        + stats["suspicious"].as_u64().context("no suspicious")?;
    let total = detected + stats["undetected"].as_u64().context("no undetected")?;

    Ok((detected, total))
}

#[derive(Debug, Clone, Parser)]
/// Scan a file with `VirusTotal`
pub struct Args {
    // TODO: Use manifest reference and -a flag for scanning installed apps
    #[clap(help = "The apps to scan for viruses")]
    apps: Vec<package::Reference>,

    #[clap(
        short,
        long,
        help = "Whether or not the pattern should match case-sensitively"
    )]
    case_sensitive: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(
        long,
        help = "Only show apps with a higher status than the specified one"
    )]
    filter: Option<Status>,

    #[clap(from_global)]
    arch: Architecture,

    #[clap(short = 'A', long, help = "Scan all installed apps")]
    all: bool,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    const BETA: bool = true;

    async fn runner(
        self,
        ctx: &impl ScoopContext<Config = config::Scoop>,
    ) -> Result<(), anyhow::Error> {
        let config = ctx.config();
        let api_key = config.virustotal_api_key.clone().calm_expect(
            "No virustotal api key found.\n  Get one at https://www.virustotal.com/gui/my-apikey and set with\n  scoop config virustotal_api_key <API key>",
        );

        let client = vt3::VtClient::new(&api_key).user_agent(USER_AGENT);

        #[allow(clippy::redundant_closure)]
        let manifests = if self.all {
            ctx.installed_apps()?
                .into_par_iter()
                .map(|path| path.join("current").join("manifest.json"))
                .filter(|path| path.exists())
                // The closure is redundant, but it's necessary to avoid a rust-analyzer error
                .map(|path| Manifest::from_path(path))
                .collect::<Result<_, _>>()?
        } else {
            let manifests = self
                .apps
                .iter()
                .map(|reference| async move { reference.list_manifests(ctx).await });

            futures::future::try_join_all(manifests)
                .await?
                .into_par_iter()
                .flatten()
                .collect::<Vec<_>>()
        };

        let pb = ProgressBar::new(manifests.len() as u64)
            .with_style(style(Some(ProgressOptions::PosLen), None));

        let rate_limiter = RateLimiter::new(4, Duration::from_secs(5));

        let matches = manifests
            .into_iter()
            .filter_map(|manifest| {
                let result = if let Some(hash) = manifest.install_config(self.arch).hash {
                    Some(hash.map(SearchType::FileHash).to_vec())
                } else {
                    manifest
                        .install_config(self.arch)
                        .urls
                        .map(|url| url.map(SearchType::Url).to_vec())
                };

                result.map(|result| {
                    result
                        .into_iter()
                        .map(|r| (StrippedManifest::new(&manifest, r.clone()), r))
                        .collect::<Vec<_>>()
                })
            })
            .flatten()
            .map(|(manifest, search_type)| {
                let client = client.clone();
                let pb = pb.clone();
                let rate_limiter = rate_limiter.clone();
                async move {
                    rate_limiter.wait().await;

                    let result = match search_type {
                        SearchType::FileHash(hash) => {
                            let result = tokio::task::spawn_blocking(move || {
                                client.file_info(&hash.to_string())
                            })
                            .await?
                            .recoverable();

                            if let Some(result) = result {
                                serde_json::to_value(result?)?
                            } else {
                                return anyhow::Ok((manifest, None));
                            }
                        }
                        SearchType::Url(url) => {
                            let result = tokio::task::spawn_blocking(move || client.url_info(&url))
                                .await?
                                .recoverable();

                            if let Some(result) = result {
                                serde_json::to_value(result?)?
                            } else {
                                return anyhow::Ok((manifest, None));
                            }
                        }
                    };

                    let (detected, total) = extract_info(&serde_json::to_value(result)?)?;

                    pb.inc(1);

                    anyhow::Ok((
                        manifest,
                        Some((Status::from_stats(detected, total), detected, total)),
                    ))
                }
            });

        let matches = futures::future::try_join_all(matches).await?;

        for (manifest, result) in matches {
            if let Some((file_status, detected, total)) = result {
                self.handle_output(manifest, file_status, detected, total)?;
            } else {
                eprintln!(
                    "Error while getting info for {}. Try again later.",
                    manifest.name
                );
            }
        }

        Ok(())
    }
}

impl Args {
    fn handle_output(
        &self,
        manifest: StrippedManifest,
        file_status: Status,
        detected: u64,
        total: u64,
    ) -> std::fmt::Result {
        if let Some(filter) = self.filter {
            if file_status <= filter {
                return Ok(());
            }
        }

        let mut info = format!("{}/{}: {detected}/{total}", manifest.bucket, manifest.name,);

        if let SearchType::FileHash(hash) = manifest.search_type {
            use std::fmt::Write;

            write!(
                info,
                ". See more at https://www.virustotal.com/gui/url/{hash}"
            )?;
        }

        match file_status {
            Status::Malicious => eprintln_red!("{info}"),
            Status::Suspicious => eprintln_yellow!("{info}"),
            Status::Undetected => eprintln_green!("{info}"),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_ord() {
        assert!(Status::Malicious > Status::Suspicious);
        assert!(Status::Suspicious > Status::Undetected);
    }
}
