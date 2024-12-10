mod app;
mod bucket;
mod cache;
mod checkup;
mod credits;
mod debug;
mod depends;
mod describe;
mod export;
mod hook;
#[cfg(not(feature = "v2"))]
mod outdated;
mod search;
mod status;
mod update;
mod virustotal;

use clap::Subcommand;

use sfsu_macros::Hooks;
use sprinkles::{config, contexts::ScoopContext};

use crate::{abandon, output::colours::eprintln_yellow};

#[derive(Debug, Clone, Copy)]
pub struct DeprecationWarning {
    /// Deprecation message
    message: DeprecationMessage,
    /// Version to be removed in
    version: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeprecationMessage {
    /// Replacement info
    Replacement(&'static str),
    /// Warning message
    Warning(&'static str),
}

impl std::fmt::Display for DeprecationMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeprecationMessage::Replacement(replacement) => {
                write!(f, "Use `{replacement}` instead")
            }
            DeprecationMessage::Warning(warning) => write!(f, "{warning}"),
        }
    }
}

impl std::fmt::Display for DeprecationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DEPRECATED: ")?;

        std::fmt::Display::fmt(&self.message, f)?;

        if let Some(version) = self.version {
            write!(f, ". Will be removed in v{version}. ")?;
        }

        Ok(())
    }
}

pub trait Runnable
where
    Self: Sized,
{
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()>;
}

// TODO: Run command could return `impl Display` and print that itself
pub trait Command {
    const BETA: bool = false;
    const NEEDS_ELEVATION: bool = false;

    const DEPRECATED: Option<DeprecationWarning> = None;

    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()>;
}

pub trait CommandRunner: Command {
    async fn run(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        if let Some(deprecation_warning) = Self::DEPRECATED {
            eprintln_yellow!("{deprecation_warning}\n");
        }

        if Self::NEEDS_ELEVATION && !quork::root::is_root()? {
            abandon!("This command requires elevation. Please run as an administrator.");
        }

        if Self::BETA {
            eprintln_yellow!(
                "This command is in beta and may not work as expected. Please report any and all bugs you find!\n",
            );
        }

        self.runner(ctx).await
    }
}

impl<T: Command> CommandRunner for T {}

#[derive(Debug, Clone, Subcommand, Hooks)]
pub enum Commands {
    App(app::Args),
    #[cfg(not(feature = "v2"))]
    #[command_name = "app cat"]
    Cat(app::cat::Args),
    #[cfg(all(feature = "download", not(feature = "v2")))]
    #[command_name = "app download"]
    Download(app::download::Args),
    #[cfg(not(feature = "v2"))]
    #[command_name = "app home"]
    Home(app::home::Args),
    #[cfg(not(feature = "v2"))]
    #[command_name = "app info"]
    Info(app::info::Args),
    #[cfg(not(feature = "v2"))]
    #[command_name = "app list"]
    List(app::list::Args),

    #[no_hook]
    Hook(hook::Args),

    Search(search::Args),
    #[cfg(not(feature = "v2"))]
    UnusedBuckets(bucket::unused::Args),
    Bucket(bucket::Args),
    #[cfg(not(feature = "v2"))]
    Describe(describe::Args),
    #[cfg(not(feature = "v2"))]
    Outdated(outdated::Args),
    Depends(depends::Args),
    Status(status::Args),
    #[cfg_attr(not(feature = "v2"), no_hook)]
    Update(update::Args),
    Export(export::Args),
    Checkup(checkup::Args),
    #[cfg(feature = "download")]
    Cache(cache::Args),
    #[hook_name = "virustotal"]
    #[clap(alias = "virustotal")]
    Scan(virustotal::Args),
    #[no_hook]
    Credits(credits::Args),
    #[no_hook]
    #[cfg(debug_assertions)]
    Debug(debug::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl sprinkles::contexts::ScoopContext<Config = sprinkles::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::App(args) => args.run(ctx).await,
            Commands::Cat(args) => args.run(ctx).await,
            Commands::Download(args) => args.run(ctx).await,
            Commands::Home(args) => args.run(ctx).await,
            Commands::Info(args) => args.run(ctx).await,
            Commands::List(args) => args.run(ctx).await,
            Commands::Hook(args) => args.run(ctx).await,
            Commands::Search(args) => args.run(ctx).await,
            Commands::UnusedBuckets(args) => args.run(ctx).await,
            Commands::Bucket(args) => args.run(ctx).await,
            Commands::Describe(args) => args.run(ctx).await,
            Commands::Outdated(args) => args.run(ctx).await,
            Commands::Depends(args) => args.run(ctx).await,
            Commands::Status(args) => args.run(ctx).await,
            Commands::Update(args) => args.run(ctx).await,
            Commands::Export(args) => args.run(ctx).await,
            Commands::Checkup(args) => args.run(ctx).await,
            Commands::Cache(args) => args.run(ctx).await,
            Commands::Scan(args) => args.run(ctx).await,
            Commands::Credits(args) => args.run(ctx).await,
            Commands::Debug(args) => args.run(ctx).await,
        }
    }
}
