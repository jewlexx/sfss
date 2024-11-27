use anyhow::Context;
use chrono::{DateTime, Local};
use quork::traits::truthy::ContainsTruth;
use rayon::prelude::*;
use serde::Serialize;
use sprinkles::{
    contexts::ScoopContext,
    packages::{CreateManifest, InstallManifest, Manifest},
};
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::wrappers::time::NicerTime;

#[derive(Debug, Serialize)]
/// Minimal package info
pub struct Info {
    /// The name of the package
    pub name: String,
    /// The version of the package
    pub version: String,
    /// The package's source (eg. bucket name)
    pub source: String,
    /// The last time the package was updated
    pub updated: NicerTime<Local>,
    /// The package's notes
    pub notes: String,
}

impl Info {
    /// Parse minmal package info for every installed app
    ///
    /// # Errors
    /// - Invalid file names
    /// - File metadata errors
    /// - Invalid time
    pub fn list_installed(
        ctx: &impl ScoopContext,
        bucket: Option<&String>,
    ) -> anyhow::Result<Vec<Self>> {
        let apps = ctx.installed_apps()?;

        apps.par_iter()
            .map(Self::from_path)
            .filter(|package| {
                if let Ok(pkg) = package {
                    if let Some(bucket) = bucket {
                        return &pkg.source == bucket;
                    }
                }
                // Keep errors so that the following line will return the error
                true
            })
            .collect()
    }

    /// Parse minimal package into from a given path
    ///
    /// # Errors
    /// - Invalid file names
    /// - File metadata errors
    /// - Invalid time
    ///
    /// # Panics
    /// - Date time invalid or out of range
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let package_name = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .context("Missing file name")?;

        let updated_time = {
            let updated = {
                let updated_sys = path.metadata()?.modified()?;

                updated_sys.duration_since(UNIX_EPOCH)?.as_secs()
            };

            #[allow(clippy::cast_possible_wrap)]
            DateTime::from_timestamp(updated as i64, 0)
                .expect("invalid or out-of-range datetime")
                .with_timezone(&Local)
        };

        let app_current = path.join("current");

        let (manifest_broken, manifest) =
            match Manifest::from_path(app_current.join("manifest.json")) {
                Ok(manifest) => (false, manifest),
                _ => (true, Manifest::default()),
            };

        let (install_manifest_broken, install_manifest) =
            match InstallManifest::from_path(app_current.join("install.json")) {
                Ok(install_manifest) => (false, install_manifest),
                _ => (true, InstallManifest::default()),
            };

        let broken = manifest_broken || install_manifest_broken;

        let mut notes = vec![];

        if broken {
            notes.push("Install failed".to_string());
        }
        if install_manifest.hold.contains_truth() {
            notes.push("Held package".to_string());
        }

        Ok(Self {
            name: package_name.to_string(),
            version: manifest.version.to_string(),
            source: install_manifest.get_source(),
            updated: updated_time.into(),
            notes: notes.join(", "),
        })
    }
}
