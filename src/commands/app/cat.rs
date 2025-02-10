use std::{fs::File, io::Read, sync::atomic::Ordering};

use clap::Parser;
use sprinkles::{contexts::ScoopContext, packages::reference::package};

use crate::{COLOR_ENABLED, abandon};

#[derive(Debug, Clone, Parser)]
/// Show content of specified manifest
pub struct Args {
    #[clap(help = "The manifest to display")]
    package: package::Reference,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let manifests = self.package.list_manifest_paths(ctx);

        if manifests.is_empty() {
            abandon!("No manifests found for {}", self.package);
        }

        let manifest = &manifests[0];

        let manifest_content = {
            let mut buf = vec![];

            let mut file = File::open(manifest)?;
            file.read_to_end(&mut buf)?;

            buf
        };

        if COLOR_ENABLED.load(Ordering::Relaxed) {
            use bat::PrettyPrinter;

            PrettyPrinter::new()
                .input_from_bytes(&manifest_content)
                .language("json")
                .print()?;
        } else {
            print!("{}", String::from_utf8_lossy(&manifest_content));
        }

        Ok(())
    }
}
