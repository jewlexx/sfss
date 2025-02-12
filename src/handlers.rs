use std::borrow::Cow;

use sprinkles::packages::reference::package;

use crate::output::colours::{bright_red, green, yellow};

pub trait ListApps<C: ?Sized> = Fn(&C) -> anyhow::Result<Option<Vec<package::Reference>>>;

pub struct AppsDecider<'c, C: ?Sized, F: ListApps<C>> {
    ctx: &'c C,
    list_all: F,
    provided: Vec<package::Reference>,
}

impl<'c, C: ?Sized, F: ListApps<C>> AppsDecider<'c, C, F> {
    pub fn new(ctx: &'c C, all: F, provided: Vec<package::Reference>) -> Self {
        Self {
            ctx,
            list_all: all,
            provided,
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
        let Some(installed_apps) = (self.list_all)(self.ctx)? else {
            return Ok(Some(self.provided));
        };

        if self.provided.is_empty() {
            return Ok(Some(installed_apps));
        }

        let choices = [
            (
                bright_red!(
                    "{} - {}",
                    upper_first_char(const { CollectionNames::all() }),
                    installed_apps.len()
                )
                .to_string(),
                installed_apps,
            ),
            (
                green!(
                    "{} - {} (see command invocation)",
                    upper_first_char(const { CollectionNames::provided() }),
                    self.provided.len()
                )
                .to_string(),
                self.provided,
            ),
        ];

        let prompt = yellow!(
            "You have {provided}, but also selected {all}. Which collection would you like to choose?",
            provided = const { CollectionNames::provided() },
            all = const { CollectionNames::all() },
        ).to_string();

        let Some(choice_index) = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(&[&choices[0].0, &choices[1].0])
            .default(1)
            .interact_opt()
            .map_err(|dialoguer::Error::IO(error)| error)?
        else {
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

#[derive(Debug, Copy, Clone)]
pub struct CollectionNames {
    all: &'static str,
    provided: &'static str,
}

impl CollectionNames {
    pub const DEFAULT: Self = Self {
        all: "all installed apps",
        provided: "provided apps",
    };

    pub const fn all() -> &'static str {
        Self::DEFAULT.all
    }

    pub const fn provided() -> &'static str {
        Self::DEFAULT.provided
    }
}

impl Default for CollectionNames {
    fn default() -> Self {
        Self::DEFAULT
    }
}

pub fn upper_first_char(s: &str) -> Cow<'_, str> {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return Cow::Borrowed(s);
    };

    if first.is_lowercase() {
        Cow::Owned(format!("{}{}", first.to_uppercase(), chars.as_str()))
    } else {
        Cow::Borrowed(s)
    }
}
