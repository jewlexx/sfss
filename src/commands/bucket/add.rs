use tokio::process::Command;

use anyhow::Context;
use clap::Parser;
use sprinkles::contexts::ScoopContext;

use crate::{abandon, calm_panic::CalmUnwrap};

#[derive(Debug, Clone, Parser)]
/// Add a bucket
pub struct Args {
    #[clap(help = "The name of the bucket to add")]
    name: String,

    #[clap(help = "The url of the bucket to add")]
    repo: Option<String>,

    #[clap(from_global)]
    disable_git: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let repo_url = self
            .repo
            .clone()
            .context("No repo provided")
            .unwrap_or_else(|_| {
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
            abandon!("Bucket {name} already exists. Remove it first if you want to add it again: `sfsu bucket rm {name}`", name = self.name);
        }

        if self.disable_git {
            let root = prodash::tree::Root::new();
            let handle = crate::progress::render::Renderer::Line.launch_ambient_gui(
                root.clone(),
                false,
                "Cloning repository".into(),
            )?;

            let clone_progress = root.add_child_with_id("Cloning repository", *b"CLON");

            sprinkles::git::clone::clone(&repo_url, dest_path, clone_progress)?;

            handle.abort();
        } else {
            let git_path = sprinkles::git::which().calm_expect("git not found");

            let exit_status = Command::new(git_path)
                .current_dir(ctx.buckets_path())
                .arg("clone")
                .arg(repo_url)
                .arg(self.name)
                .spawn()?
                .wait_with_output()
                .await?;

            match exit_status.status.code() {
                Some(0) => {}
                Some(code) => {
                    return Err(anyhow::anyhow!(
                        "git exited with code {}.\nOutput:\n{}",
                        code,
                        String::from_utf8_lossy(&exit_status.stdout)
                    ))
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "git exited without a status code.\nOutput:\n{}",
                        String::from_utf8_lossy(&exit_status.stdout)
                    ))
                }
            }
        };

        Ok(())
    }
}
