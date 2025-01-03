use shadow_rs::Shadow;

use super::lock::Lockfile;

#[derive(Debug, Copy, Clone)]
pub struct SprinklesVersion<'a> {
    version: &'a str,
    source: &'a str,
    git_rev: Option<&'a str>,
}

impl<'a> SprinklesVersion<'a> {
    pub fn from_doc(doc: &'a Lockfile) -> Self {
        let sprinkles = doc.get_package("sprinkles-rs").unwrap();

        let version = sprinkles.get("version").unwrap().as_str().unwrap();
        let source = sprinkles.get("source").unwrap().as_str().unwrap();

        Self {
            version,
            source,
            git_rev: source
                .starts_with("git+")
                .then(|| source.split('#').nth(1).unwrap()),
        }
    }

    pub fn print_variables(&self) {
        let Self {
            version,
            source,
            git_rev,
        } = self;

        println!("cargo:rustc-env=SPRINKLES_VERSION={version}");
        println!("cargo:rustc-env=SPRINKLES_SOURCE={source}");
        println!("cargo:rustc-env=SPRINKLES_GIT_SOURCE={}", git_rev.is_some());
        println!(
            "cargo:rustc-env=SPRINKLES_GIT_REV={}",
            git_rev.unwrap_or_default()
        );
    }

    pub fn long_version(&self, shadow: &Shadow) -> String {
        let map = &shadow.map;

        let sprinkles_rev = if let Some(git_rev) = self.git_rev() {
            format!(" (git rev: {})", git_rev)
        } else {
            " (crates.io published version)".to_string()
        };

        let (major, minor, patch) = git2::Version::get().libgit2_version();

        format!(
            "{pkg_version} \n\
            sprinkles {sprinkles_version}{sprinkles_rev} \n\
            branch:{branch} \n\
            tag:{tag} \n\
            commit_hash:{short_commit} \n\
            build_time:{build_time} \n\
            build_env:{rust_version},{rust_channel} \n\
            libgit2:{major}.{minor}.{patch}",
            sprinkles_version = self.version(),
            branch = &map.get("BRANCH").expect("missing BRANCH").v,
            build_time = &map.get("BUILD_TIME").expect("missing BUILD_TIME").v,
            pkg_version = &map.get("PKG_VERSION").expect("missing PKG_VERSION").v,
            rust_channel = &map.get("RUST_CHANNEL").expect("missing RUST_CHANNEL").v,
            rust_version = &map.get("RUST_VERSION").expect("missing RUST_VERSION").v,
            short_commit = &map.get("SHORT_COMMIT").expect("missing SHORT_COMMIT").v,
            tag = &map.get("TAG").expect("missing TAG").v,
        )
    }

    pub fn version(&self) -> &str {
        self.version
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn git_rev(&self) -> Option<&str> {
        self.git_rev
    }
}
