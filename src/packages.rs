use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;

use crate::get_scoop_path;

pub mod install;
pub mod manifest;

pub use install::Manifest as InstallManifest;
pub use manifest::Manifest;

pub trait CreateManifest
where
    Self: Default + for<'a> Deserialize<'a>,
{
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Self::from_str(contents)
    }

    fn from_str(contents: String) -> std::io::Result<Self> {
        let trimmed = contents.trim_start_matches('\u{feff}');

        let parsed = serde_json::from_str(trimmed).unwrap_or_else(|err| {
            println!("Error parsing manifest:\n {trimmed}");
            println!("{err}");

            Default::default()
        });

        Ok(parsed)
    }
}

impl CreateManifest for Manifest {}

impl CreateManifest for InstallManifest {}

/// Check if the manifest path is installed, and optionally confirm the bucket
///
/// # Panics
/// - The file was not valid UTF-8
pub fn is_installed(manifest_name: impl AsRef<Path>, bucket: Option<impl AsRef<str>>) -> bool {
    let scoop_path = get_scoop_path();
    let installed_path = scoop_path
        .join("apps")
        .join(manifest_name)
        .join("current/install.json");

    match InstallManifest::from_path(installed_path) {
        Ok(manifest) => {
            if let Some(bucket) = bucket {
                manifest.get_source() == bucket.as_ref()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
