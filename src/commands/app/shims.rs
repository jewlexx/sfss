use clap::Parser;
use sprinkles::{Architecture, contexts::ScoopContext, packages::reference::package};

#[derive(Debug, Clone, Parser)]
/// List the apps shims
pub struct Args {
    #[clap(help = "The manifest to list shims from")]
    package: package::Reference,

    #[clap(from_global)]
    arch: Architecture,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let handle = self.package.open_handle(ctx).await?;
        let shims = handle.list_shims(self.arch)?;

        // if self.json {
        //     println!("{}", serde_json::to_string(&shims)?);
        // } else {
        for shim in shims {
            println!("{}", shim.path(ctx).display());
        }
        // }

        Ok(())
    }
}
