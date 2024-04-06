use itertools::Itertools as _;
use quork::traits::truthy::ContainsTruth;
use serde::Serialize;

use crate::{buckets::Bucket, Scoop};

use super::{reference, Manifest, Result};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct Info {
    pub name: String,
    pub current: String,
    pub available: String,
    pub missing_dependencies: Vec<reference::Package>,
    pub info: Option<String>,
}

impl Info {
    /// Parse [`Info`] from a local manifest
    ///
    /// # Errors
    /// - If the local manifest is missing
    /// - If the install manifest is missing
    pub fn from_manifests(local_manifest: &Manifest, bucket: &Bucket) -> Result<Self> {
        let failed = {
            let installed = Scoop::app_installed(&local_manifest.name)?;

            let app_path = Scoop::apps_path()
                .join(&local_manifest.name)
                .join("current");

            !app_path.exists() && installed
        };

        debug!("Local manifest name: {}", local_manifest.name);
        let remote_manifest = bucket.get_manifest(&local_manifest.name)?;

        let install_manifest = local_manifest.install_manifest()?;

        let held = install_manifest.hold.unwrap_or_default();

        let missing_dependencies = local_manifest
            .depends()
            .into_iter()
            .filter(|reference| {
                debug!("Checking if {} is installed.", reference.name());
                !reference::Package::installed(reference).contains_truth()
            })
            .collect_vec();

        let mut info = String::new();

        if failed {
            info += "Install failed";
        }
        if held {
            info += "Held package";
        }

        Ok(Info {
            name: remote_manifest.name.clone(),
            current: local_manifest.version.clone(),
            available: remote_manifest.version.clone(),
            missing_dependencies,
            info: (!info.is_empty()).then_some(info),
        })
    }
}
