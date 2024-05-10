use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

use sprinkles::{
    buckets::Bucket,
    calm_panic::CalmUnwrap,
    config,
    contexts::ScoopContext,
    output::sectioned::{Children, Section, Sections},
    packages::SearchMode,
    Architecture,
};

#[derive(Debug, Clone, Parser)]
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
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
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
                    Ok(manifest) => {
                        let sections = manifest
                            .into_par_iter()
                            .filter_map(|manifest| {
                                manifest.parse_output(
                                    ctx,
                                    &manifest.bucket,
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
