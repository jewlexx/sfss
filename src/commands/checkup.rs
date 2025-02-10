use clap::Parser;

use itertools::Itertools;
use sprinkles::contexts::ScoopContext;

use crate::diagnostics::{Diagnostics, LongPathsStatus};

#[derive(Debug, Clone, Parser)]
/// Check for common issues
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let diagnostics = Diagnostics::collect(ctx)?;

        if self.json {
            println!("{}", serde_json::to_string_pretty(&diagnostics)?);
            return Ok(());
        }

        if diagnostics.git_installed {
            println!("✅ Git is installed");
        } else {
            println!("❌ Git is not installed");
            println!(
                "\tScoop relies on Git to manage itself and its buckets. sfsu does not expressly require git, but it is still recommended to install it, until sfsu can manage itself entirely without Scoop."
            );
            println!("\tRun `scoop install git` to install it");
        }

        if diagnostics.main_bucket {
            println!("✅ Main bucket is installed");
        } else {
            println!("❌ Main bucket is not installed");
            println!("\tRun `scoop bucket add main` to install it");
        }

        // I've disabled this because it's highly insecure to disable Windows Defender for Scoop
        // if !sprinkles::is_elevated()? {
        //     println!("❓ Windows Defender status could not be checked");
        //     println!("\tRun this command as admin to check Windows Defender status");
        // } else if diagnostics.windows_defender {
        //     println!("✅ Windows Defender is ignoring the Scoop directory");
        // } else {
        //     println!("❌ Windows Defender is not ignoring the Scoop directory");
        //     println!("\tWindows Defender may slow down or disrupt installs with realtime scanning");
        //     println!(
        //         "\tConsider running: `sudo Add-MpPreference -ExclusionPath '{}'`",
        //         User::path().display()
        //     );
        // }

        if diagnostics.windows_developer {
            println!("✅ Windows Developer Mode is enabled");
        } else {
            println!("❌ Windows Developer Mode is not enabled");
            println!(
                "\tWindows Developer Mode is not enabled. Operations relevant to symlinks may fail without proper rights"
            );
        }

        match diagnostics.long_paths {
            LongPathsStatus::Enabled => println!("✅ Long paths are enabled"),
            LongPathsStatus::OldWindows => {
                println!("❌ This version of Windows does not support long paths");
            }
            LongPathsStatus::Disabled => {
                println!("❌ Long paths are disabled");
                println!(
                    "\tRun `Set-ItemProperty 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\FileSystem' -Name 'LongPathsEnabled' -Value 1` as admin to enable it"
                );
            }
        }

        if diagnostics.scoop_ntfs {
            println!("✅ NTFS is the filesystem of the Scoop directory");
        } else {
            println!("❌ NTFS is not the filesystem of the Scoop directory");
            println!(
                "\tScoop requires an NTFS volume to work! Please point `$env:SCOOP or 'root_path' variable in '~/.config/scoop/config.json' to another Drive with NTFS filesystem"
            );
        }

        for helper in diagnostics.missing_helpers {
            println!("❌ Missing helper: {}", helper.name);
            println!(
                "\tInstall it with: {}",
                helper
                    .packages
                    .iter()
                    .map(|pkg| format!("`scoop install {pkg}`"))
                    .join(" or ")
            );
        }

        Ok(())
    }
}
