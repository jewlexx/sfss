use clap::Parser;
use sprinkles::contexts::ScoopContext;

use crate::abandon;

#[derive(Debug, Clone, Parser)]
/// Add a bucket
pub struct Args {
    #[clap(help = "The name of the bucket to add")]
    name: String,

    #[clap(help = "The url of the bucket to add")]
    repo: Option<String>,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let repo_url = self.repo.clone().unwrap_or_else(|| {
            let known_buckets = ctx.known_buckets();

            if let Some(url) = known_buckets.get(&self.name) {
                (*url).to_string()
            } else {
                abandon!(
                    "No bucket found with the name \"{}\". Try passing the url as well",
                    self.name
                )
            }
        });

        let dest_path = ctx.buckets_path().join(&self.name);

        if dest_path.exists() {
            abandon!(
                "Bucket {name} already exists. Remove it first if you want to add it again: `sfsu bucket rm {name}`",
                name = self.name
            );
        }

        let root = prodash::tree::Root::new();
        let handle = crate::progress::render::LineRenderer::run(root.clone(), true);

        let clone_progress = root.add_child_with_id("Cloning repository", *b"REPO");

        sprinkles::git::clone::clone(&repo_url, dest_path, clone_progress)?;

        handle.await?;

        Ok(())
    }
}
