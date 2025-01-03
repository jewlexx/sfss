use std::{error::Error, fs::File, io::Write, path::PathBuf};

use contribs::contributors::Contributors;
use dotenv::dotenv;

#[path = "build/mod.rs"]
mod build;

use build::tools::{colours::Colours, lock::Lockfile, version::SprinklesVersion};

const WIN_MANIFEST: &str = include_str!("./sfsu.exe.manifest");

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

#[allow(unused_variables, unreachable_code)]
fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    println!("cargo:rerun-if-changed=sfsu.exe.manifest");
    let mut res = winres::WindowsResource::new();
    res.set_manifest(WIN_MANIFEST);

    res.compile().expect("Failed to compile Windows resources");

    let lockfile = Lockfile::new();

    let shadow = shadow_rs::ShadowBuilder::builder()
        .hook(append_shadow_hooks)
        .build_pattern(shadow_rs::BuildPattern::RealTime)
        .build()?;

    std::fs::write(
        out_dir.join("long_version.txt"),
        SprinklesVersion::from_doc(&lockfile).long_version(&shadow),
    )?;

    Ok(())
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

    let lockfile = Lockfile::new();

    writeln!(file, "{}", lockfile.get_packages())?;

    Colours::colours_hook(file)?;

    Ok(())
}
