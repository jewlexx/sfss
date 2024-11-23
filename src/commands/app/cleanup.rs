use clap::Parser;
use sprinkles::{contexts::ScoopContext, packages::reference::package};

use crate::{
    commands::outdated::apps, handlers::handle_installed_apps, output::colours::eprintln_yellow,
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
        let cleanup_apps = match handle_installed_apps(ctx, self.all, self.apps)?.as_deref() {
            Some([]) | None => {
                eprintln_yellow!("No apps selected. Exiting now.");
                return Ok(());
            }
            Some(apps) => apps,
        };

        unimplemented!()
    }
}
