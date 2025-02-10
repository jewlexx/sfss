//! Scoop diagnostics helpers

use std::{ffi::OsString, os::windows::ffi::OsStringExt};

use anyhow::Error;
use itertools::Itertools;
use serde::Serialize;
use sprinkles::{buckets::Bucket, contexts::ScoopContext};

#[derive(Debug, Copy, Clone, Serialize)]
/// The status of long paths
pub enum LongPathsStatus {
    /// Long paths are enabled
    Enabled,
    /// This version of windows does not support long paths
    OldWindows,
    /// Long paths are disabled
    Disabled,
}

#[derive(Debug, Copy, Clone, Serialize)]
/// A helper program
pub struct Helper {
    /// The executable name
    pub exe: &'static str,
    /// The name of the program
    pub name: &'static str,
    /// The reason the program is needed
    pub reason: &'static str,
    /// The packages that provide the program
    pub packages: &'static [&'static str],
}

const EXPECTED_HELPERS: &[Helper] = &[
    Helper {
        exe: "7z",
        name: "7-Zip",
        reason: "unpacking most programs",
        packages: &["7zip"],
    },
    Helper {
        exe: "innounp",
        name: "Inno Setup Unpacker",
        reason: "unpacking InnoSetup files",
        packages: &["innounp"],
    },
    Helper {
        exe: "dark",
        name: "Dark",
        reason: "unpacking installers created with the WiX toolkit",
        packages: &["dark", "wixtoolset"],
    },
];

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize)]
/// Diagnostics information
pub struct Diagnostics {
    /// If git is installed
    pub git_installed: bool,
    /// The status of long paths
    pub long_paths: LongPathsStatus,
    /// If the main bucket exists
    pub main_bucket: bool,
    /// If the user has developer mode enabled
    pub windows_developer: bool,
    /// If Windows Defender is ignoring the Scoop directory
    pub windows_defender: bool,
    /// The missing helper programs
    pub missing_helpers: Vec<Helper>,
    /// If the Scoop directory is on an NTFS filesystem
    pub scoop_ntfs: bool,
}

impl Diagnostics {
    /// Collect all diagnostics
    ///
    /// # Errors
    /// - Unable to check long paths
    /// - Unable to check main bucket
    /// - Unable to check windows developer status
    /// - Unable to check windows defender status
    pub fn collect(ctx: &impl ScoopContext) -> Result<Self, Error> {
        let git_installed = Self::git_installed();
        debug!("Check git is installed");
        let main_bucket = Self::check_main_bucket(ctx)?;
        debug!("Checked main bucket");
        let long_paths = Self::check_long_paths()?;
        debug!("Checked long paths");
        let windows_developer = Self::get_windows_developer_status()?;
        debug!("Checked developer mode");

        let windows_defender = if quork::root::is_root()? {
            Self::check_windows_defender(ctx)?
        } else {
            false
        };
        debug!("Checked windows defender");

        let missing_helpers = EXPECTED_HELPERS
            .iter()
            .filter(|helper| which::which(helper.exe).is_err())
            .copied()
            .collect();

        let scoop_ntfs = Self::is_ntfs(ctx)?;

        Ok(Self {
            git_installed,
            long_paths,
            main_bucket,
            windows_developer,
            windows_defender,
            missing_helpers,
            scoop_ntfs,
        })
    }

    #[allow(unreachable_code)]
    /// Check if Windows Defender is ignoring the Scoop directory
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to open the registry key
    /// - Unable to check if the key exists
    pub fn check_windows_defender(ctx: &impl ScoopContext) -> windows::core::Result<bool> {
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

        let scoop_dir = ctx.path();
        let key = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\Microsoft\Windows Defender\Exclusions\Paths")?;

        let scoop_dir_key = key.open_subkey(scoop_dir);

        Ok(scoop_dir_key.is_ok())
    }

    /// Check if the main bucket exists
    ///
    /// # Errors
    /// - Unable to list buckets
    pub fn check_main_bucket(ctx: &impl ScoopContext) -> Result<bool, Error> {
        let buckets = Bucket::list_all(ctx)?;

        Ok(buckets.into_iter().any(|bucket| bucket.name() == "main"))
    }

    /// Check if long paths are enabled
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to read the OS version
    pub fn check_long_paths() -> windows::core::Result<LongPathsStatus> {
        use windows_version::OsVersion;
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

        let version = OsVersion::current();

        let major_version = version.major;
        debug!("Windows Major Version: {major_version}");

        if major_version < 10 {
            return Ok(LongPathsStatus::OldWindows);
        }

        let hlkm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hlkm.open_subkey(r"SYSTEM\CurrentControlSet\Control\FileSystem")?;

        if key.get_value::<u32, _>("LongPathsEnabled")? == 0 {
            Ok(LongPathsStatus::Disabled)
        } else {
            Ok(LongPathsStatus::Enabled)
        }
    }

    /// Check if the user has developer mode enabled
    ///
    /// # Errors
    /// - Unable to read the registry
    /// - Unable to read the value
    pub fn get_windows_developer_status() -> windows::core::Result<bool> {
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

        let hlkm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hlkm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock")?;

        Ok(key.get_value::<u32, _>("AllowDevelopmentWithoutDevLicense")? == 1)
    }

    /// Check if the Scoop directory is on an NTFS filesystem
    ///
    /// # Errors
    /// - Unable to get the volume information
    /// - Unable to check the filesystem
    /// - Unable to get the root path
    pub fn is_ntfs(ctx: &impl ScoopContext) -> windows::core::Result<bool> {
        use windows::{
            Win32::{Foundation::MAX_PATH, Storage::FileSystem::GetVolumeInformationW},
            core::HSTRING,
        };

        let path = ctx.path();

        let root = {
            let mut current = path;

            while let Some(parent) = current.parent() {
                current = parent;
            }

            debug!("Checking filesystem of: {}", current.display());

            current
        };

        let mut fs_name = [0u16; MAX_PATH as usize];

        unsafe {
            GetVolumeInformationW(
                &HSTRING::from(root),
                None,
                None,
                // &mut max_component_length,
                None,
                // &mut flags,
                None,
                Some(&mut fs_name),
            )?;
        }

        debug!("Filesystem: {:?}", OsString::from_wide(&fs_name));

        Ok(fs_name.starts_with(&"NTFS".encode_utf16().collect_vec()))
    }

    #[must_use]
    /// Check if the user has git installed, and in their path
    pub fn git_installed() -> bool {
        which::which("git").is_ok()
    }
}
