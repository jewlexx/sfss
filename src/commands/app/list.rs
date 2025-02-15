use clap::{Parser, ValueEnum};
use rayon::prelude::*;

use sprinkles::contexts::ScoopContext;

use crate::{models::min::Info, output::structured::Structured};

#[derive(Debug, Clone, Parser)]
/// List all installed packages
pub struct Args {
    #[cfg(not(feature = "v2"))]
    #[clap(
        help = format!("The pattern to search for (can be a regex). {}", console::style("DEPRECATED: Use sfsu search --installed. Will be removed in v2").yellow())
    )]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively list packages in")]
    bucket: Option<String>,

    #[clap(long, help = "Sort by the given field", default_value = "name")]
    sort_by: SortBy,

    #[clap(long, help = "Sort in descending order")]
    descending: bool,

    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum SortBy {
    Name,
    Version,
    Source,
    Updated,
    Notes,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let mut outputs = Info::list_installed(ctx, self.bucket.as_ref())?;

        outputs.par_sort_by(|a, b| match self.sort_by {
            SortBy::Name => a.name.cmp(&b.name),
            SortBy::Version => a.version.cmp(&b.version),
            SortBy::Source => a.source.cmp(&b.source),
            SortBy::Updated => a.updated.cmp(&b.updated),
            SortBy::Notes => a.notes.cmp(&b.notes),
        });

        if self.descending {
            outputs.reverse();
        }

        if self.json {
            let output_json = serde_json::to_string_pretty(&outputs)?;

            println!("{output_json}");
        } else {
            if outputs.is_empty() {
                println!("No packages found.");
                return Ok(());
            }

            let values = outputs
                .into_par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs = Structured::new(&values);

            print!("{outputs}");
        }

        Ok(())
    }
}
