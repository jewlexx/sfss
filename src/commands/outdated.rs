use std::{rc::Rc, sync::Arc};

use anyhow::Context;
use clap::Parser;
use rayon::prelude::*;
use sfsu::packages::{CreateManifest, InstallManifest, Manifest};

use crate::ResultIntoOption;

#[derive(Debug, Clone, Parser)]
/// List outdated packages
pub struct Args;

impl super::Command for Args {
    fn run(self) -> anyhow::Result<()> {
        let apps = Manifest::list_installed()?;

        let buckets = sfsu::buckets::Bucket::list_all()?;

        let mut outdated: Vec<OutdatedPackage> = vec![];

        for app in &apps {
            for bucket in &buckets {
                if let Ok(manifest) = bucket.get_manifest(&app.name) {
                    if manifest.version != app.version {
                        outdated.push(OutdatedPackage {
                            name: app.name.clone(),
                            current: app.version.clone(),
                            available: manifest.version.clone(),
                        });
                    }
                }
            }
        }

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OutdatedPackage {
    name: String,
    current: String,
    available: String,
}
