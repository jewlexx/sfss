use std::{error::Error, fs::File, io::Write, path::PathBuf};

use contribs::contributors::Contributors;
use dotenv::dotenv;
use shadow_rs::Shadow;
use toml_edit::DocumentMut;

const LOCKFILE: &str = include_str!("./Cargo.lock");
const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");
const COLOURS: &[&str] = &[
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

const COLOURS_TXT: &str = r#"
#[macro_export]
#[doc = concat!("Create a colored string with the `", stringify!(#ident), "` color.")]
macro_rules! #ident {
    ($($arg:tt)*) => {{
        console::style(format_args!($($arg)*)).#ident()
    }};
}

#[macro_export]
#[doc = concat!("Create a colored string with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #ident_bright {
    ($($arg:tt)*) => {{
        $crate::output::colours::#ident!($($arg)*).bright()
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string with the `", stringify!(#ident), "` color.")]
macro_rules! #println {
    ($($arg:tt)*) => {{
        println!("{}", $crate::output::colours::#ident!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #println_bright {
    ($($arg:tt)*) => {{
        println!("{}", $crate::output::colours::#ident_bright!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident), "` color.")]
macro_rules! #eprintln {
    ($($arg:tt)*) => {{
        eprintln!("{}", $crate::output::colours::#ident!($($arg)*))
    }};
}

#[macro_export]
#[doc = concat!("Print a colored string to stderr with the `", stringify!(#ident_bright), "` color.")]
macro_rules! #eprintln_bright {
    ($($arg:tt)*) => {{
        eprintln!("{}", $crate::output::colours::#ident_bright!($($arg)*))
    }};
}

pub use #ident;
pub use #ident_bright;
pub use #println;
pub use #println_bright;
pub use #eprintln;
pub use #eprintln_bright;
"#;

fn get_contributors((owner, repo): (&str, &str)) -> Result<String, Box<dyn Error>> {
    // Try and load dotenv file
    _ = dotenv();

    if let Ok(api_key) = std::env::var("CONTRIBUTORS_TOKEN") {
        let contributors = Contributors::new(api_key, owner.into(), repo.into())?;
        let contributors =
            tokio::runtime::Runtime::new()?.block_on(async move { contributors.await })?;

        let contributors = contributors
            .into_iter()
            .filter_map(|contrib| {
                let name = contrib.name.as_ref().or(contrib.login.as_ref())?.clone();

                if name.contains("[bot]") || name == "jewlexx" {
                    return None;
                }

                let login = contrib.login.as_ref()?.clone();
                let url = format!("https://github.com/{login}");

                Some(format!("(\"{name}\",\"{url}\")"))
            })
            .collect::<Vec<_>>();
        let length = contributors.len();

        let contributors = format!("[{}]", contributors.join(", "));
        let contributors_output =
            format!("pub const CONTRIBUTORS: [(&str, &str); {length}] = {contributors};");

        Ok(contributors_output)
    } else {
        if std::env::var("IS_RELEASE").is_ok() {
            panic!("No CONTRIBUTORS_TOKEN found, contributors will not be updated.");
        }

        Ok("pub const CONTRIBUTORS: [(&str, &str); 0] = [];".to_string())
    }
}

fn get_packages(doc: &DocumentMut) -> String {
    let packages = doc.get("package").unwrap();
    let packages = packages.as_array_of_tables().unwrap();

    let mut items = vec![];
    for p in packages {
        let name = p.get("name").unwrap().as_str().unwrap();
        let version = p.get("version").unwrap().as_str().unwrap();

        let item = format!("(\"{name}\",\"{version}\")");
        items.push(item);
    }

    let length = items.len();
    let items = items.join(",");
    let items = format!("[{}]", items);
    format!("pub const PACKAGES: [(&str, &str); {length}] = {items};")
}

#[derive(Debug, Copy, Clone)]
struct SprinklesVersion<'a> {
    version: &'a str,
    source: &'a str,
    git_rev: Option<&'a str>,
}

impl<'a> SprinklesVersion<'a> {
    fn from_doc(doc: &'a DocumentMut) -> Self {
        let sprinkles = doc["package"]
            .as_array_of_tables()
            .unwrap()
            .iter()
            .find(|table| {
                let pp = table["name"].as_str().unwrap();
                pp == "sprinkles-rs"
            })
            .unwrap();

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

    fn print_variables(&self) {
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
}

fn write_colours(mut file: &File) -> std::io::Result<()> {
    writeln!(file, "pub mod colours {{")?;
    writeln!(file, "#![allow(unused_imports)]")?;
    writeln!(file, "// This file is autogenerated")?;

    for colour in COLOURS {
        let output = COLOURS_TXT
            .replace("#ident_bright", &format!("bright_{colour}"))
            .replace("#ident", colour)
            .replace("#eprintln_bright", &format!("eprintln_bright_{colour}"))
            .replace("#eprintln", &format!("eprintln_{colour}"))
            .replace("#println_bright", &format!("println_bright_{colour}"))
            .replace("#println", &format!("println_{colour}"));

        file.write_all(output.as_bytes())?;
    }

    writeln!(file, "}}")?;

    Ok(())
}

#[allow(unused_variables, unreachable_code)]
fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=sfsu.exe.manifest");
    let mut res = winres::WindowsResource::new();
    res.set_manifest(WIN_MANIFEST);

    res.compile().expect("Failed to compile Windows resources");

    let lockfile = LOCKFILE.parse::<DocumentMut>().unwrap();
    println!("cargo:rerun-if-changed=Cargo.lock");

    let shadow = shadow_rs::ShadowBuilder::builder()
        .hook(append_shadow_hooks)
        .build_pattern(shadow_rs::BuildPattern::RealTime)
        .build()?;

    std::fs::write(
        out_dir.join("long_version.txt"),
        get_long_version(&shadow, SprinklesVersion::from_doc(&lockfile)),
    )?;

    Ok(())
}

fn get_long_version(shadow: &Shadow, sprinkles_version: SprinklesVersion<'_>) -> String {
    let map = &shadow.map;

    let sprinkles_rev = if sprinkles_version.source == "true" {
        format!(" (git rev: {})", sprinkles_version.git_rev.unwrap())
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
        sprinkles_version = sprinkles_version.version,
        branch = &map.get("BRANCH").expect("missing BRANCH").v,
        build_time = &map.get("BUILD_TIME").expect("missing BUILD_TIME").v,
        pkg_version = &map.get("PKG_VERSION").expect("missing PKG_VERSION").v,
        rust_channel = &map.get("RUST_CHANNEL").expect("missing RUST_CHANNEL").v,
        rust_version = &map.get("RUST_VERSION").expect("missing RUST_VERSION").v,
        short_commit = &map.get("SHORT_COMMIT").expect("missing SHORT_COMMIT").v,
        tag = &map.get("TAG").expect("missing TAG").v,
    )
}

fn append_shadow_hooks(mut file: &File) -> shadow_rs::SdResult<()> {
    let sfsu_contribs = {
        let contributors = get_contributors(("winpax", "sfsu"));

        match contributors {
            Ok(contributors) => contributors,
            Err(e) if std::env::var("IS_RELEASE").is_ok_and(|v| v == "true") => {
                panic!("Getting contributors failed with error: {e}");
            }
            _ => "pub const CONTRIBUTORS: [(&str, &str); 0] = [];".to_string(),
        }
    };

    writeln!(file, "pub mod sfsu {{\n{sfsu_contribs}\n}}")?;

    let sprinkles_contribs = {
        let contributors = get_contributors(("winpax", "sprinkles"));

        match contributors {
            Ok(contributors) => contributors,
            Err(e) if std::env::var("IS_RELEASE").is_ok_and(|v| v == "true") => {
                panic!("Getting contributors failed with error: {e}");
            }
            _ => "pub const CONTRIBUTORS: [(&str, &str); 0] = [];".to_string(),
        }
    };

    writeln!(file, "pub mod sprinkles {{\n{sprinkles_contribs}\n}}")?;

    let lockfile = LOCKFILE
        .parse::<DocumentMut>()
        .expect("Failed to parse Cargo.lock");

    writeln!(file, "{}", get_packages(&lockfile))?;

    write_colours(file)?;

    Ok(())
}
