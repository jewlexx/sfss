use std::{path::Path, process::Command};

use clap::Parser;
use itertools::Itertools;
use quork::traits::truthy::ContainsTruth;
use sprinkles::{
    Architecture,
    contexts::ScoopContext,
    handles::packages::PackageHandle,
    packages::{
        models::manifest::SingleOrArray,
        reference::{manifest, package},
    },
    scripts::PowershellScript,
};

use crate::output::colours::eprintln_red;

#[derive(Debug, Clone, Parser)]
/// Uninstall an app
pub struct Args {
    /// The package(s) to uninstall
    packages: Vec<package::Reference>,

    #[clap(short, long)]
    /// Remove all persistent data
    purge: bool,

    #[clap(from_global)]
    global: bool,

    #[clap(short, long)]
    /// Print what would be done, but don't actually do anything
    dry_run: bool,
}

impl super::Command for Args {
    fn needs_elevation(&self) -> bool {
        self.global
    }

    async fn runner(mut self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let valid_packages = std::mem::take(&mut self.packages).into_iter().filter(|package| {
            if matches!(
                package.manifest,
                manifest::Reference::BucketNamePair { .. } | manifest::Reference::Name(_)
            ) {
                if package.name().is_some_and(|name| name == "scoop") {
                    eprintln_red!("Uninstalling Scoop is not supported yet. Please run `scoop.ps1 uninstall scoop` instead");
                    false
                } else if package.installed(ctx).contains_truth() {
                    true
                } else {
                    eprintln_red!("'{}' not installed", package.name().expect("name exists. this is a bug please report it"));
                    false
                }
            } else {
                eprintln_red!("Invalid package reference. You cannot reference a file or url for uninstallation");
                false
            }
        }).collect_vec();

        if valid_packages.is_empty() {
            eprintln_red!("No packages provided");
            return Ok(());
        }

        let packages_with_manifest = {
            let packages_future = valid_packages
                .into_iter()
                .map(|package| async { package.open_handle(ctx).await });

            futures::future::try_join_all(packages_future).await?
        };

        for handle in packages_with_manifest {
            self.uninstall_handle(ctx, &handle)?;
        }

        todo!()
    }
}

impl Args {
    fn uninstall_handle<C: ScoopContext>(
        &self,
        ctx: &C,
        handle: &PackageHandle<'_, C>,
    ) -> anyhow::Result<()> {
        if !self.dry_run {
            handle.unlink_current()?;
        }

        let version_dir = handle.version_dir();
        // let persist_dir = handle.persist_dir();

        let manifest = handle.local_manifest()?;

        let install_config = manifest.install_config(Architecture::ARCH);

        if self.dry_run {
            // Run the pre-uninstall script
            if let Some(ref pre_uninstall) = install_config.pre_uninstall {
                let script_runner = pre_uninstall.save(ctx)?;
                script_runner.run()?;
            }
        }

        if handle.running() {
            eprintln_red!(
                "{} is running. Please stop it before uninstalling",
                unsafe { handle.name() }
            );
            return Ok(());
        }

        if !check_for_permissions(&version_dir) {
            eprintln_red!(
                "Access Denied: {}. Try again, or fix permissions on the directory",
                version_dir.display(),
            );
            return Ok(());
        }

        if let Some(uninstaller) = install_config.uninstaller {
            if let Some(host) = uninstaller.host(ctx) {
                host.run()?;
            }
        }

        todo!()
    }
}

fn check_for_permissions(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    let Ok(metadata) = path.metadata() else {
        return false;
    };

    if metadata.permissions().readonly() {
        return false;
    }

    true
}
