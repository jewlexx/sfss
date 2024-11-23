use std::io;

use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
};

use crate::output::colours::{bright_red, green, yellow};

/// This will get all apps if all is true and no apps were passed.
///
/// If all is false and apps are passed, it will only return those apps.
///
/// If all is true and apps are passed, it will prompt the user to select which collection to choose.
///
/// If the return value is empty, that means no apps were provided and all was false.
pub fn handle_installed_apps(
    ctx: &impl ScoopContext,
    all: bool,
    apps: Vec<package::Reference>,
) -> io::Result<Option<Vec<package::Reference>>> {
    if all {
        let installed_apps: Vec<package::Reference> = {
            let installed_apps = ctx.installed_apps()?;
            let manifest_paths = installed_apps.into_iter().filter_map(|path| {
                let manifest_path = path.join("current").join("manifest.json");

                manifest_path
                    .try_exists()
                    .ok()
                    .and_then(|exists| exists.then_some(manifest_path))
            });

            let references = manifest_paths
                .map(manifest::Reference::File)
                .map(manifest::Reference::into_package_ref);

            references.collect()
        };

        if apps.is_empty() {
            Ok(Some(installed_apps))
        } else {
            let choices = [
                (
                    bright_red!("All installed apps - {}", installed_apps.len()).to_string(),
                    installed_apps,
                ),
                (
                    green!("Provided apps - {} (see command invocation)", apps.len()).to_string(),
                    apps,
                ),
            ];

            let Some( choice_index) = dialoguer::Select::new()
                .with_prompt(yellow!("You have provided apps, but also selected to cleanup all installed apps. Which collection would you like to cleanup?").to_string())
                .items(&[&choices[0].0, &choices[1].0])
                .default(1)
                .interact_opt()
                .map_err(|dialoguer::Error::IO(error)| error)? else {
                    return Ok(None);
                };

            let (_, apps) = {
                let mut choices = choices;

                std::mem::replace(
                    &mut choices[choice_index],
                    const { (String::new(), Vec::new()) },
                )
            };

            Ok(Some(apps))
        }
    } else {
        Ok(Some(apps))
    }
}
