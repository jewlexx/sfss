#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    rust_2024_compatibility
)]

// TODO: Replace regex with glob

mod commands;
mod logging;

use clap::Parser;

use commands::Commands;

mod versions {
    #![allow(clippy::needless_raw_string_hashes)]
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));

    pub const SFSU_LONG_VERSION: &str = {
        use shadow_rs::formatcp;

        const LIBGIT2_VERSION: &str = env!("LIBGIT2_VERSION");

        formatcp!(
            r#"{}
sprinkles {}
branch:{}
tag:{}
commit_hash:{}
build_time:{}
build_env:{},{}
libgit2:{}"#,
            PKG_VERSION,
            sprinkles::__versions::VERSION,
            BRANCH,
            TAG,
            SHORT_COMMIT,
            BUILD_TIME,
            RUST_VERSION,
            RUST_CHANNEL,
            LIBGIT2_VERSION
        )
    };
}

#[macro_use]
extern crate log;

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

    #[clap(short, long, global = true, help = "Enable verbose logging")]
    verbose: bool,

    #[clap(
        long,
        global = true,
        help = "Disable using git commands for certain parts of the program. Allows sfsu to work entirely if you don't have git installed, but can negatively affect performance.",
        env = "DISABLE_GIT"
    )]
    disable_git: bool,
}

fn main() -> anyhow::Result<()> {
    logging::panics::handle();

    let args = Args::parse();

    logging::Logger::init(if cfg!(debug_assertions) {
        true
    } else {
        args.verbose
    })?;

    if args.no_color {
        debug!("Colour disabled globally");
        owo_colors::set_override(false);
    }

    debug!("Running command: {:?}", args.command);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async { args.command.run().await })
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
