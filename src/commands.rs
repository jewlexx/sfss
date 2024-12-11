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

// TODO: Replace strip macro with a custom enum
// Use match to ensure all variants are covered
// This will allow more flexibility with hooks in future
// as sfsu invocation deviates from scoop's
#[derive(Debug, Clone, Subcommand, quork::macros::Strip)]
#[stripped(ident = CommandHooks)]
#[stripped_meta(derive(Debug, Copy, Clone, quork::macros::ListVariants, PartialEq, Eq))]
pub enum Commands {
    App(app::Args),
    #[cfg(not(feature = "v2"))]
    Cat(app::cat::Args),
    #[cfg(all(feature = "download", not(feature = "v2")))]
    Download(app::download::Args),
    #[cfg(not(feature = "v2"))]
    Home(app::home::Args),
    #[cfg(not(feature = "v2"))]
    Info(app::info::Args),
    #[cfg(not(feature = "v2"))]
    List(app::list::Args),

    #[stripped(ignore)]
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
    #[cfg_attr(not(feature = "v2"), stripped(ignore))]
    Update(update::Args),
    Export(export::Args),
    Checkup(checkup::Args),
    #[cfg(feature = "download")]
    Cache(cache::Args),
    #[clap(alias = "virustotal")]
    Scan(virustotal::Args),
    #[stripped(ignore)]
    Credits(credits::Args),
    #[stripped(ignore)]
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
            #[cfg(not(feature = "v2"))]
            Commands::Cat(args) => args.run(ctx).await,
            #[cfg(all(feature = "download", not(feature = "v2")))]
            Commands::Download(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::Home(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::Info(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::List(args) => args.run(ctx).await,
            Commands::Hook(args) => args.run(ctx).await,
            Commands::Search(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::UnusedBuckets(args) => args.run(ctx).await,
            Commands::Bucket(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
            Commands::Describe(args) => args.run(ctx).await,
            #[cfg(not(feature = "v2"))]
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

impl CommandHooks {
    pub const fn command<'a>(self) -> &'a str {
        match self {
            CommandHooks::App => "app",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Cat => "app cat",
            #[cfg(all(feature = "download", not(feature = "v2")))]
            CommandHooks::Download => "app download",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Home => "app home",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Info => "app info",
            #[cfg(not(feature = "v2"))]
            CommandHooks::List => "app list",
            CommandHooks::Search => "search",
            #[cfg(not(feature = "v2"))]
            CommandHooks::UnusedBuckets => "unused-buckets",
            CommandHooks::Bucket => "bucket",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Describe => "describe",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Outdated => "outdated",
            CommandHooks::Depends => "depends",
            CommandHooks::Status => "status",
            CommandHooks::Export => "export",
            CommandHooks::Checkup => "checkup",
            CommandHooks::Cache => "cache",
            CommandHooks::Scan => "scan",
            #[cfg(feature = "v2")]
            CommandHooks::Update => "update",
        }
    }

    pub const fn hook<'a>(self) -> &'a str {
        match self {
            CommandHooks::App => "app",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Cat => "cat",
            #[cfg(all(feature = "download", not(feature = "v2")))]
            CommandHooks::Download => "download",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Home => "home",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Info => "info",
            #[cfg(not(feature = "v2"))]
            CommandHooks::List => "list",
            CommandHooks::Search => "search",
            #[cfg(not(feature = "v2"))]
            CommandHooks::UnusedBuckets => "unused-buckets",
            CommandHooks::Bucket => "bucket",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Describe => "describe",
            #[cfg(not(feature = "v2"))]
            CommandHooks::Outdated => "outdated",
            CommandHooks::Depends => "depends",
            CommandHooks::Status => "status",
            CommandHooks::Export => "export",
            CommandHooks::Checkup => "checkup",
            CommandHooks::Cache => "cache",
            CommandHooks::Scan => "virustotal",
            #[cfg(feature = "v2")]
            CommandHooks::Update => "update",
        }
    }
}

impl From<String> for CommandHooks {
    fn from(string: String) -> Self {
        match string.as_str() {
            "app" => CommandHooks::App,
            #[cfg(not(feature = "v2"))]
            "cat" => CommandHooks::Cat,
            #[cfg(all(feature = "download", not(feature = "v2")))]
            "download" => CommandHooks::Download,
            #[cfg(not(feature = "v2"))]
            "home" => CommandHooks::Home,
            #[cfg(not(feature = "v2"))]
            "info" => CommandHooks::Info,
            #[cfg(not(feature = "v2"))]
            "list" => CommandHooks::List,
            "search" => CommandHooks::Search,
            #[cfg(not(feature = "v2"))]
            "unused-buckets" => CommandHooks::UnusedBuckets,
            "bucket" => CommandHooks::Bucket,
            #[cfg(not(feature = "v2"))]
            "describe" => CommandHooks::Describe,
            #[cfg(not(feature = "v2"))]
            "outdated" => CommandHooks::Outdated,
            "depends" => CommandHooks::Depends,
            "status" => CommandHooks::Status,
            "export" => CommandHooks::Export,
            "checkup" => CommandHooks::Checkup,
            "cache" => CommandHooks::Cache,
            "virustotal" => CommandHooks::Scan,
            #[cfg(feature = "v2")]
            "update" => CommandHooks::Update,
            _ => panic!("Invalid command name: {string}"),
        }
    }
}
