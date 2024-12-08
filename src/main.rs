#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    rust_2024_compatibility
)]
// Allow using single match instead of if let
// This allows us to circumvent the lifetime changes coming in rust 2024
#![allow(clippy::single_match_else)]
// Ignore this lint for now. Cases are not an issue,
// and it cannot be disabled for a single line AFAIK
#![allow(tail_expr_drop_order)]
#![feature(trait_alias)]

// TODO: Replace regex with glob

mod calm_panic;
mod commands;
mod diagnostics;
mod errors;
mod handlers;
mod limits;
mod logging;
mod models;
mod output;
mod validations;
mod wrappers;

use std::{
    io::IsTerminal,
    sync::atomic::{AtomicBool, Ordering},
};

use clap::Parser;

use commands::Commands;
use logging::Logger;
use sprinkles::contexts::{AnyContext, ScoopContext, User};

#[cfg(feature = "contexts")]
use sprinkles::contexts::Global;
use validations::Validate;

shadow_rs::shadow!(shadow);

mod versions {
    #![allow(clippy::needless_raw_string_hashes)]

    use konst::eq_str;

    // TODO: Move this into build script
    pub const SFSU_LONG_VERSION: &str = {
        use crate::shadow::{
            BRANCH, BUILD_TIME, PKG_VERSION, RUST_CHANNEL, RUST_VERSION, SHORT_COMMIT, TAG,
        };
        use shadow_rs::formatcp;

        const LIBGIT2_VERSION: &str = env!("LIBGIT2_VERSION");

        const SPRINKLES_VERSION: &str = env!("SPRINKLES_VERSION");
        const SPRINKLES_GIT_SOURCE: &str = env!("SPRINKLES_GIT_SOURCE");
        const SPRINKLES_GIT_REV: &str = env!("SPRINKLES_GIT_REV");

        const SPRINKLES_REV: &str = if eq_str(SPRINKLES_GIT_SOURCE, "true") {
            formatcp!(" (git rev: {SPRINKLES_GIT_REV})")
        } else {
            " (crates.io published version)"
        };

        formatcp!(
            r#"{PKG_VERSION}
sprinkles {SPRINKLES_VERSION}{SPRINKLES_REV}
branch:{BRANCH}
tag:{TAG}
commit_hash:{SHORT_COMMIT}
build_time:{BUILD_TIME}
build_env:{RUST_VERSION},{RUST_CHANNEL}
libgit2:{LIBGIT2_VERSION}"#
        )
    };
}

#[macro_use]
extern crate log;

// TODO: Add dry-run option for debugging

/// Scoop utilities that can replace the slowest parts of Scoop, and run anywhere from 30-100 times faster
#[derive(Debug, Parser)]
#[clap(about, long_about, version, long_version = versions::SFSU_LONG_VERSION, author)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[clap(
        long,
        global = true,
        help = "Disable terminal formatting",
        env = "NO_COLOR"
    )]
    no_color: bool,

    #[clap(
        long,
        global = true,
        help = "Print in the raw JSON output, rather than a human readable format"
    )]
    json: bool,

    #[clap(short, long, global = true, help = "Show more information in outputs")]
    verbose: bool,

    #[clap(long, global = true, help = "Enable debug logging")]
    debug: bool,

    #[clap(
        long,
        global = true,
        help = "Disable using git commands for certain parts of the program. Allows sfsu to work entirely if you don't have git installed, but can negatively affect performance.",
        env = "DISABLE_GIT"
    )]
    disable_git: bool,

    #[cfg(feature = "contexts")]
    #[clap(short, long, global = true, help = "Use the global Scoop context")]
    global: bool,

    #[clap(
        global = true,
        short = 'y',
        long,
        help = "Assume \"yes\" as answer to prompts"
    )]
    assume_yes: bool,
}

pub(crate) static COLOR_ENABLED: AtomicBool = AtomicBool::new(true);

#[cfg(feature = "contexts")]
impl TryFrom<&Args> for AnyContext {
    type Error = anyhow::Error;

    fn try_from(args: &Args) -> anyhow::Result<Self> {
        Ok(if args.global {
            AnyContext::Global(Global::new()?)
        } else {
            AnyContext::User(User::new()?)
        })
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    logging::panics::handle();

    let args = Args::parse();

    let ctx: AnyContext = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "contexts")] {
                (&args).try_into()?
            } else {
                AnyContext::User(User::new())
            }
        }
    };

    // Spawn a task to cleanup logs in the background
    tokio::task::spawn_blocking({
        let ctx = ctx.clone();
        move || Logger::cleanup_logs(&ctx)
    });

    Logger::init(&ctx, cfg!(debug_assertions) || args.verbose).await?;

    if args.no_color || !std::io::stdout().is_terminal() {
        debug!("Colour disabled globally");
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
        COLOR_ENABLED.store(false, Ordering::Relaxed);
    }

    ctx.config().validate()?;

    debug!("Running command: {:?}", args.command);

    args.command.run(&ctx).await?;

    Ok(())
}

// /// Get the owner of a file path
// ///
// /// # Errors
// /// - Interacting with system I/O
// ///
// /// # Panics
// /// - Owner's name isn't valid utf8
// NOTE: This currently does now work
// pub fn file_owner(path: impl AsRef<Path>) -> std::io::Result<String> {
//     use std::{fs::File, os::windows::io::AsRawHandle};
//     use windows::{
//         core::{PCSTR, PSTR},
//         Win32::{
//             Foundation::{HANDLE, PSID},
//             Security::{
//                 Authorization::{GetSecurityInfo, SE_FILE_OBJECT},
//                 LookupAccountSidA, OWNER_SECURITY_INFORMATION,
//             },
//         },
//     };

//     let file = File::open(path.as_ref().join("current/install.json"))?;
//     let handle = HANDLE(file.as_raw_handle() as isize);

//     let owner_psid: MaybeUninit<PSID> = MaybeUninit::uninit();

//     unsafe {
//         GetSecurityInfo(
//             handle,
//             SE_FILE_OBJECT,
//             OWNER_SECURITY_INFORMATION,
//             Some(owner_psid.as_ptr().cast_mut()),
//             None,
//             None,
//             None,
//             None,
//         )?;
//     }

//     let owner_name = PSTR::null();

//     unsafe {
//         LookupAccountSidA(
//             PCSTR::null(),
//             owner_psid.assume_init(),
//             owner_name,
//             std::ptr::null_mut(),
//             PSTR::null(),
//             std::ptr::null_mut(),
//             std::ptr::null_mut(),
//         )?;
//     }

//     Ok(unsafe { owner_name.to_string().expect("valid utf8 name") })
// }
