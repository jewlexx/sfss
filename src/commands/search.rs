use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

use sprinkles::{
    Architecture,
    buckets::Bucket,
    contexts::ScoopContext,
    packages::{Manifest, MergeDefaults, SearchMode},
    version::Version,
};

use crate::{
    calm_panic::CalmUnwrap,
    output::{
        sectioned::{Children, Section, Sections, Text},
        warning,
    },
};

#[derive(Debug, Clone)]
#[must_use = "MatchCriteria has no side effects"]
/// The criteria for a match
pub struct MatchCriteria {
    name: bool,
    bins: Vec<String>,
}

impl MatchCriteria {
    /// Create a new match criteria
    pub const fn new() -> Self {
        Self {
            name: false,
            bins: vec![],
        }
    }

    /// Check if the name matches
    pub fn matches(
        file_name: &str,
        pattern: &Regex,
        list_binaries: impl FnOnce() -> Vec<String>,
        mode: SearchMode,
    ) -> Self {
        let mut output = MatchCriteria::new();

        if mode.match_names() {
            output.match_names(pattern, file_name);
        }

        if mode.match_binaries() {
            output.match_binaries(pattern, list_binaries());
        }

        output
    }

    fn match_names(&mut self, pattern: &Regex, file_name: &str) -> &mut Self {
        if pattern.is_match(file_name) {
            self.name = true;
        }
        self
    }

    fn match_binaries(&mut self, pattern: &Regex, binaries: Vec<String>) -> &mut Self {
        let binary_matches = binaries
            .into_iter()
            .filter(|binary| pattern.is_match(binary))
            .filter_map(|b| {
                if pattern.is_match(&b) {
                    Some(b.clone())
                } else {
                    None
                }
            });

        self.bins.extend(binary_matches);

        self
    }
}

impl Default for MatchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

struct MatchedManifest {
    manifest: Manifest,
    installed: bool,
    name_matched: bool,
    bins: Vec<String>,
    exact_match: bool,
}

impl MatchedManifest {
    pub fn new(
        ctx: &impl ScoopContext,
        manifest: Manifest,
        pattern: &Regex,
        mode: SearchMode,
        arch: Architecture,
    ) -> MatchedManifest {
        // TODO: Better display of output
        let bucket = unsafe { manifest.bucket() };

        let match_output = MatchCriteria::matches(
            unsafe { manifest.name() },
            pattern,
            // Function to list binaries from a manifest
            // Passed as a closure to avoid this parsing if bin matching isn't required
            || {
                manifest
                    .architecture
                    .merge_default(manifest.install_config.clone(), arch)
                    .bin
                    .map(|b| b.to_vec())
                    .unwrap_or_default()
            },
            mode,
        );

        let installed = manifest.is_installed(ctx, Some(bucket));
        let exact_match = unsafe { manifest.name() } == pattern.to_string();

        MatchedManifest {
            manifest,
            installed,
            name_matched: match_output.name,
            bins: match_output.bins,
            exact_match,
        }
    }

    pub fn should_match(&self, installed_only: bool) -> bool {
        if !self.installed && installed_only {
            return false;
        }
        if !self.name_matched && self.bins.is_empty() {
            return false;
        }

        true
    }

    pub fn to_section(&self) -> Section<Text<String>> {
        let styled_package_name = if self.exact_match {
            console::style(unsafe { self.manifest.name() })
                .bold()
                .to_string()
        } else {
            unsafe { self.manifest.name() }.to_string()
        };

        let installed_text = if self.installed { "[installed] " } else { "" };

        let title = format!(
            "{styled_package_name} ({}) {installed_text}",
            self.manifest.version
        );

        if self.bins.is_empty() {
            Section::new(Children::None)
        } else {
            let bins = self
                .bins
                .iter()
                .map(|output| {
                    Text::new(format!(
                        "{}{}",
                        crate::output::WHITESPACE,
                        console::style(output).bold()
                    ))
                })
                .collect_vec();

            Section::new(Children::from(bins))
        }
        .with_title(title)
    }

    pub fn into_output(self) -> MatchedOutput {
        MatchedOutput {
            name: unsafe { self.manifest.name() }.to_string(),
            bucket: unsafe { self.manifest.bucket() }.to_string(),
            version: self.manifest.version.clone(),
            installed: self.installed,
            bins: self.bins,
        }
    }
}

impl std::fmt::Display for MatchedManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.to_section(), f)
    }
}

#[derive(Debug, serde::Serialize)]
struct MatchedOutput {
    name: String,
    bucket: String,
    version: Version,
    installed: bool,
    bins: Vec<String>,
}

#[derive(Debug, Clone, Parser)]
/// Search for a package
pub struct Args {
    #[clap(help = "The regex pattern to search for, using Rust Regex syntax")]
    pattern: String,

    #[clap(
        short,
        long,
        help = "Whether or not the pattern should match case-sensitively"
    )]
    case_sensitive: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(short, long, help = "Only search installed packages")]
    installed: bool,

    #[clap(short, long, help = "Search mode to use", default_value_t)]
    mode: SearchMode,

    #[clap(from_global)]
    arch: Architecture,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let (bucket, raw_pattern) = if let Some((bucket, raw_pattern)) =
            self.pattern.split_once('/')
        {
            warning!("bucket/package syntax is deprecated. Please use the --bucket flag instead");
            (
                Some({
                    // Bucket flag overrides bucket/package syntax
                    if let Some(bucket) = self.bucket {
                        warning!("Using bucket flag instead of bucket/package syntax");
                        bucket
                    } else {
                        bucket.to_string()
                    }
                }),
                raw_pattern.to_string(),
            )
        } else {
            (self.bucket, self.pattern)
        };

        let pattern = {
            Regex::new(&format!(
                "{}{raw_pattern}",
                if self.case_sensitive { "" } else { "(?i)" },
            ))
            .calm_expect(
                "Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info",
            )
        };

        let matching_buckets: Vec<Bucket> =
            if let Some(Ok(bucket)) = bucket.map(|name| Bucket::from_name(ctx, name)) {
                vec![bucket]
            } else {
                Bucket::list_all(ctx)?
            };

        let buckets: HashMap<String, Vec<MatchedManifest>> = matching_buckets
            .par_iter()
            .filter_map(
                |bucket| match bucket.matches(ctx, self.installed, &pattern, self.mode) {
                    Ok(manifests) => {
                        let matches = manifests
                            .into_par_iter()
                            .map(|manifest| {
                                MatchedManifest::new(ctx, manifest, &pattern, self.mode, self.arch)
                            })
                            .filter(|matched_manifest| {
                                matched_manifest.should_match(self.installed)
                            })
                            .collect::<Vec<_>>();

                        if matches.is_empty() {
                            None
                        } else {
                            Some((bucket.name().to_string(), matches))
                        }
                    }
                    _ => None,
                },
            )
            .collect();

        if self.json {
            let json_matches: HashMap<String, Vec<MatchedOutput>> = buckets
                .into_iter()
                .map(|(bucket, matches)| {
                    let bucket_matches: Vec<MatchedOutput> = matches
                        .into_iter()
                        .map(MatchedManifest::into_output)
                        .collect();

                    (bucket, bucket_matches)
                })
                .collect();

            serde_json::to_writer_pretty(std::io::stdout(), &json_matches)?;
        } else {
            let mut matches: Sections<_> = buckets
                .into_iter()
                .map(|(bucket, matches)| {
                    let mut sections = vec![];

                    matches
                        .par_iter()
                        .map(MatchedManifest::to_section)
                        .collect_into_vec(&mut sections);

                    Section::new(Children::from(sections)).with_title(format!("'{bucket}' bucket:"))
                })
                .collect();

            matches.par_sort();

            print!("{matches}");
        }

        Ok(())
    }
}
