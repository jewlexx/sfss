use itertools::Itertools;
use rayon::prelude::*;

use clap::Parser;
use regex::{Match, Regex};

use sprinkles::{
    buckets::Bucket,
    contexts::ScoopContext,
    packages::{Manifest, MergeDefaults, SearchMode},
    Architecture,
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

struct MatchedManifest<'m> {
    manifest: &'m Manifest,
    bucket: String,
    installed: bool,
    name_matched: bool,
    bins: Vec<String>,
    exact_match: bool,
}

impl<'m> MatchedManifest<'m> {
    pub fn new(
        ctx: &impl ScoopContext,
        manifest: &'m Manifest,
        bucket: impl AsRef<str>,
        pattern: &Regex,
        mode: SearchMode,
        arch: Architecture,
    ) -> MatchedManifest<'m> {
        // TODO: Better display of output

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

        let installed = manifest.is_installed(ctx, Some(bucket.as_ref()));
        let exact_match = unsafe { manifest.name() } == pattern.to_string();

        MatchedManifest {
            manifest,
            bucket: bucket.as_ref().to_string(),
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
    // TODO: Add json option
    // #[clap(from_global)]
    // json: bool,
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

        let mut matches: Sections<_> = matching_buckets
            .par_iter()
            .filter_map(
                |bucket| match bucket.matches(ctx, self.installed, &pattern, self.mode) {
                    Ok(manifests) => {
                        let sections = manifests
                            .into_par_iter()
                            .map(|manifest| {
                                MatchedManifest::new(
                                    ctx,
                                    &manifest,
                                    unsafe { manifest.bucket() },
                                    &pattern,
                                    self.mode,
                                    Architecture::ARCH,
                                )
                            })
                            .filter(|matched_manifest| {
                                matched_manifest.should_match(self.installed)
                            })
                            .collect::<Vec<_>>();

                        if sections.is_empty() {
                            None
                        } else {
                            let section = Section::new(Children::from(sections))
                                .with_title(format!("'{}' bucket:", bucket.name()));

                            Some(section)
                        }
                    }
                    _ => None,
                },
            )
            .collect();

        matches.par_sort();

        print!("{matches}");

        Ok(())
    }
}
