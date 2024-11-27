use sprinkles::packages::reference::package;

use crate::output::colours::{bright_red, green, yellow};

type ListApps<C> = Box<dyn Fn(&C) -> anyhow::Result<Option<Vec<package::Reference>>>>;

pub struct AppsDecider<'c, C: ?Sized> {
    ctx: &'c C,
    all: ListApps<C>,
    provided: Vec<package::Reference>,
    collections: CollectionNames,
}

impl<'c, C: ?Sized> AppsDecider<'c, C> {
    pub fn new(ctx: &'c C, all: ListApps<C>, provided: Vec<package::Reference>) -> Self {
        Self {
            ctx,
            all,
            provided,
            collections: CollectionNames::default(),
        }
    }

    /// This will get all apps if all is true and no apps were passed.
    ///
    /// If all is false and apps are passed, it will only return those apps.
    ///
    /// If all is true and apps are passed, it will prompt the user to select which collection to choose.
    ///
    /// If the return value is empty, that means no apps were provided and all was false.
    pub fn decide(self) -> anyhow::Result<Option<Vec<package::Reference>>> {
        let Some(installed_apps) = (self.all)(self.ctx)? else {
            return Ok(Some(self.provided));
        };

        if self.provided.is_empty() {
            Ok(Some(installed_apps))
        } else {
            let choices = [
                (
                    bright_red!("All installed apps - {}", installed_apps.len()).to_string(),
                    installed_apps,
                ),
                (
                    green!(
                        "Provided apps - {} (see command invocation)",
                        self.provided.len()
                    )
                    .to_string(),
                    self.provided,
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
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CollectionNames {
    all: &'static str,
    provided: &'static str,
}

impl Default for CollectionNames {
    fn default() -> Self {
        Self {
            all: "All installed apps",
            provided: "Provided apps",
        }
    }
}
