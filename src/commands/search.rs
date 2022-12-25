use std::{
    ffi::{OsStr, OsString},
    fs::{read_dir, DirEntry},
    io::Error,
};

use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

use crate::{
    buckets,
    packages::{is_installed, FromPath, Manifest},
};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(help = "The regex pattern to search for, using Rust Regex syntax")]
    pattern: String,

    #[clap(
        short = 'C',
        long,
        help = "Whether or not the pattern should match case-sensitively"
    )]
    case_sensitive: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,
}

// TODO: Add installed marker

fn parse_output(file: &DirEntry, bucket: impl AsRef<str>) -> String {
    // This may be a bit of a hack, but it works
    let path = file.path().with_extension("");
    let file_name = path.file_name();
    let package = file_name.unwrap().to_string_lossy().to_string();

    match Manifest::from_path(file.path()) {
        Ok(manifest) => {
            format!(
                "{} ({}) {}",
                package,
                manifest.version,
                if is_installed(&package, Some(bucket)) {
                    "[installed]"
                } else {
                    ""
                }
            )
        }
        Err(_) => {
            format!("{package} - Invalid")
        }
    }
}

impl super::Command for Args {
    type Error = std::io::Error;

    fn run(self) -> Result<(), Self::Error> {
        let scoop_buckets_path = buckets::Bucket::get_path();

        let pattern = {
            Regex::new(&format!(
                "{}{}",
                if self.case_sensitive { "" } else { "(?i)" },
                self.pattern
            ))
            .expect("Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info")
        };

        let all_scoop_buckets =
            read_dir(scoop_buckets_path)?.collect::<Result<Vec<_>, Self::Error>>()?;

        let scoop_buckets = {
            if let Some(bucket) = self.bucket {
                all_scoop_buckets
                    .into_iter()
                    .filter(|scoop_bucket| {
                        let path = scoop_bucket.path();
                        match path.components().last() {
                            Some(x) => x.as_os_str() == bucket.as_str(),
                            None => false,
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                all_scoop_buckets
            }
        };

        let mut matches = scoop_buckets
            .par_iter()
            .filter_map(|bucket| {
                let bucket_path = {
                    let bk_path = bucket.path().join("bucket");

                    if bk_path.exists() {
                        bk_path
                    } else {
                        bucket.path()
                    }
                };

                let bucket_contents = read_dir(bucket_path)
                    .unwrap()
                    .collect::<Result<Vec<_>, Self::Error>>()
                    .unwrap();

                let matches = bucket_contents
                    .par_iter()
                    .filter_map(|file| {
                        let path_raw = file.path();
                        let path = path_raw.to_string_lossy();

                        let is_valid_extension = matches!(
                            file.path().extension().and_then(OsStr::to_str),
                            Some("json")
                        );

                        if pattern.is_match(&path) && is_valid_extension {
                            Some(parse_output(file, bucket.file_name().to_string_lossy()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if matches.is_empty() {
                    None
                } else {
                    Some(Ok::<_, Error>((bucket.file_name(), matches)))
                }
            })
            .collect::<Result<Vec<_>, Self::Error>>()?;

        matches.par_sort_by_key(|x| x.0.clone());

        let mut old_bucket = OsString::new();

        for (bucket, matches) in matches {
            if bucket != old_bucket {
                // Do not print the newline on the first bucket
                if old_bucket != OsString::new() {
                    println!();
                }

                println!("'{}' bucket:", bucket.to_string_lossy());

                old_bucket = bucket;
            }

            for mtch in matches {
                println!("{mtch}");
            }
        }

        Ok(())
    }
}
