use itertools::Itertools;
use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

use sprinkles::{
    buckets::Bucket,
    contexts::ScoopContext,
    packages::{Manifest, MergeDefaults, SearchMode},
    Architecture,
};

use crate::{
    calm_panic::CalmUnwrap,
    output::sectioned::{Children, Section, Sections, Text},
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
        manifest: Option<&Manifest>,
        mode: SearchMode,
        pattern: &Regex,
        arch: Architecture,
    ) -> Self {
        let file_name = file_name.to_string();

        let mut output = MatchCriteria::new();

        if mode.match_names() && pattern.is_match(&file_name) {
            output.name = true;
        }

        if let Some(manifest) = manifest {
            let binaries = manifest
                .architecture
                .merge_default(manifest.install_config.clone(), arch)
                .bin
                .map(|b| b.to_vec())
                .unwrap_or_default();

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

            output.bins.extend(binary_matches);
        }

        output
    }
}

impl Default for MatchCriteria {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_output(
    manifest: &Manifest,
    ctx: &impl ScoopContext,
    bucket: impl AsRef<str>,
    installed_only: bool,
    pattern: &Regex,
    mode: SearchMode,
    arch: Architecture,
) -> Option<Section<Text<String>>> {
    // TODO: Better display of output

    // This may be a bit of a hack, but it works

    let match_output = MatchCriteria::matches(
        unsafe { manifest.name() },
        if mode.match_binaries() {
            Some(manifest)
        } else {
            None
        },
        mode,
        pattern,
        arch,
    );

    if !match_output.name && match_output.bins.is_empty() {
        return None;
    }

    let is_installed = manifest.is_installed(ctx, Some(bucket.as_ref()));
    if installed_only && !is_installed {
        return None;
    }

    let styled_package_name = if unsafe { manifest.name() } == pattern.to_string() {
        console::style(unsafe { manifest.name() })
            .bold()
            .to_string()
    } else {
        unsafe { manifest.name() }.to_string()
    };

    let installed_text = if is_installed && !installed_only {
        "[installed] "
    } else {
        ""
    };

    let title = format!(
        "{styled_package_name} ({}) {installed_text}",
        manifest.version
    );

    let package = if mode.match_binaries() {
        let bins = match_output
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
    } else {
        Section::new(Children::None)
    }
    .with_title(title);

    Some(package)
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
        let (bucket, raw_pattern) =
            if let Some((bucket, raw_pattern)) = self.pattern.split_once('/') {
                // Bucket flag overrides bucket/package syntax
                (
                    Some(self.bucket.unwrap_or(bucket.to_string())),
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

        let matching_buckets: Vec<Bucket> = match bucket.map(|name| Bucket::from_name(ctx, name)) {
            Some(Ok(bucket)) => vec![bucket],
            _ => Bucket::list_all(ctx)?,
        };

        let mut matches: Sections<_> = matching_buckets
            .par_iter()
            .filter_map(
                |bucket| match bucket.matches(ctx, self.installed, &pattern, self.mode) {
                    Ok(manifest) => {
                        let sections = manifest
                            .into_par_iter()
                            .filter_map(|manifest| {
                                parse_output(
                                    &manifest,
                                    ctx,
                                    unsafe { manifest.bucket() },
                                    self.installed,
                                    &pattern,
                                    self.mode,
                                    Architecture::ARCH,
                                )
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
