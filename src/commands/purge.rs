use clap::Parser;
use dialoguer::Confirm;
use sprinkles::{config, contexts::ScoopContext, packages::reference::package};

use crate::output::colours::println_yellow;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to purge")]
    app: package::Reference,

    #[clap(from_global)]
    assume_yes: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let app = self.app.first_installed(ctx)?;
        let persist_path = ctx.persist_path().join(unsafe { app.name() });

        if !persist_path.exists() {
            println_yellow!("Persist folder does not exist for {}", unsafe {
                app.name()
            });
            return Ok(());
        }

        if !self.assume_yes
            && !Confirm::new()
                .with_prompt(format!(
                    "Are you sure you want to purge the persist folder for \"{}\"?",
                    unsafe { app.name() }
                ))
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        Ok(())
    }
}
