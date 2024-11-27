use clap::Parser;
use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
};

use crate::{
    handlers::{AppsDecider, ListApps},
    output::colours::eprintln_yellow,
};

#[derive(Debug, Clone, Parser)]
/// Cleanup apps by removing old versions
pub struct Args {
    #[clap(help = "The app(s) to cleanup")]
    apps: Vec<package::Reference>,

    #[clap(short, long, help = "Cleanup all installed apps")]
    all: bool,

    #[clap(from_global)]
    assume_yes: bool,

    #[clap(
        long,
        help = "Print what would be done, but don't actually do anything"
    )]
    dry_run: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let cleanup_apps = match AppsDecider::new(ctx, self.list_apps(), self.apps)
            .decide()?
            .as_deref()
        {
            Some([]) | None => {
                eprintln_yellow!("No apps selected. Exiting now.");
                return Ok(());
            }
            Some(apps) => apps,
        };

        unimplemented!()
    }
}

impl Args {
    fn list_apps<C: ScoopContext>(&self) -> impl ListApps<C> + use<C> {
        let all = self.all;
        move |ctx: &C| {
            if all {
                let installed_apps: Vec<package::Reference> = {
                    let installed_apps = ctx.installed_apps()?;
                    let manifest_paths = installed_apps.into_iter().filter_map(|path| {
                        let manifest_path = path.join("current").join("manifest.json");

                        manifest_path
                            .try_exists()
                            .ok()
                            .and_then(|exists| exists.then_some(manifest_path))
                    });

                    let references = manifest_paths
                        .map(manifest::Reference::File)
                        .map(manifest::Reference::into_package_ref);

                    references.collect()
                };

                anyhow::Ok(Some(installed_apps))
            } else {
                anyhow::Ok(None)
            }
        }
    }
}
