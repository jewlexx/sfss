use std::{
    collections::VecDeque,
    fmt::Display,
    io::stdout,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use itertools::Itertools;
use parking_lot::Mutex;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List},
};
use serde::Serialize;
use sprinkles::contexts::ScoopContext;

use crate::shadow;

mod titles {
    use shadow_rs::formatcp;

    pub const TITLE: &str = concat!(
        "🚀 sfsu v",
        env!("CARGO_PKG_VERSION"),
        ", created by Juliette Cordor 🚀"
    );
    pub const CONTRIBUTORS: &str = "💖 Many thanks to all our incredible contributors 💖";
    pub const SFSU_CONTRIBUTORS: &str = "🛹 In sfsu 🛹";
    pub const SPRINKLES_CONTRIBUTORS: &str = "🍨 And in sprinkles 🍨";
    pub const PACKAGES: &str = formatcp!(
        "📦 And all the incredible {} crates we use 📦",
        crate::shadow::PACKAGES.len()
    );
}

#[derive(Debug)]
struct Timer {
    timeout: Mutex<Duration>,
    current: Mutex<Duration>,
    now: Mutex<std::time::Instant>,
}

impl Timer {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout: Mutex::new(timeout),
            current: Mutex::new(Duration::ZERO),
            now: Mutex::new(Instant::now()),
        }
    }

    pub fn tick(&self) -> bool {
        let now = Instant::now();
        let delta = now - *self.now.lock();
        *self.now.lock() = now;

        let mut current = self.current.lock();
        *current += delta;

        if *current >= *self.timeout.lock() {
            *current = Duration::ZERO;
            true
        } else {
            false
        }
    }

    pub fn set_timeout(&self, timeout: Duration) {
        *self.timeout.lock() = timeout;
    }
}

struct Url<T: Display> {
    text: T,
    url: String,
}

impl<T: Display> Url<T> {
    fn new(text: T, url: String) -> Self {
        Self { text, url }
    }
}

impl<T: Display> Display for Url<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\",
            self.url, self.text
        )
    }
}

#[derive(Debug, Clone, Parser)]
/// Show credits
pub struct Args {
    #[clap(short, long, help = "Show packages")]
    packages: bool,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, _: &impl ScoopContext) -> anyhow::Result<()> {
        if self.json {
            #[derive(Debug, Clone, Serialize)]
            struct JsonOutput<'a> {
                contributors: Vec<Contributor<'a>>,
                sprinkles_contributors: Vec<Contributor<'a>>,

                #[serde(skip_serializing_if = "Vec::is_empty")]
                packages: Vec<Package<'a>>,
            }

            #[derive(Debug, Clone, Serialize)]
            struct Contributor<'a> {
                name: &'a str,
                url: &'a str,
            }

            #[derive(Debug, Clone, Serialize)]
            struct Package<'a> {
                name: &'a str,
                version: &'a str,
            }

            let contributors = shadow::sfsu::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Contributor { name, url })
                .collect_vec();

            let sprinkles_contributors = shadow::sprinkles::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Contributor { name, url })
                .collect_vec();

            let packages = if self.packages {
                shadow::PACKAGES
                    .into_iter()
                    .map(|(name, version)| Package { name, version })
                    .collect_vec()
            } else {
                vec![]
            };

            let output = JsonOutput {
                contributors,
                sprinkles_contributors,
                packages,
            };

            let output = serde_json::to_string_pretty(&output)?;

            println!("{output}");
        } else if console::colors_enabled() {
            self.terminal_ui()?;
        } else {
            println!("{}", titles::TITLE);
            println!();
            println!("{}", titles::CONTRIBUTORS);

            println!();
            println!("{}", titles::SFSU_CONTRIBUTORS);
            println!();

            for (name, url) in shadow::sfsu::CONTRIBUTORS {
                let url = Url::new(name, url.to_string());
                println!("{url}");
            }

            println!();
            println!("{}", titles::SPRINKLES_CONTRIBUTORS);
            println!();

            for (name, url) in shadow::sprinkles::CONTRIBUTORS {
                let url = Url::new(name, url.to_string());
                println!("{url}");
            }

            if self.packages {
                println!();
                println!("{}", titles::PACKAGES);
                println!();

                for (name, version) in shadow::PACKAGES {
                    let url = Url::new(name, format!("https://crates.io/crates/{name}"));
                    println!("{url}: {version}");
                }
            }
        }

        Ok(())
    }
}

impl Args {
    fn terminal_ui(&self) -> anyhow::Result<()> {
        const TITLE_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let (rows, _) = console::Term::stdout().size();
        let rows = rows as usize;

        let prefix_items = vec![Text::raw(""); rows / 4];

        let mut items = prefix_items;

        items.extend(vec![
            Text::styled(titles::CONTRIBUTORS, TITLE_STYLE),
            Text::raw(""),
        ]);

        items.extend(vec![
            Text::styled(titles::SFSU_CONTRIBUTORS, TITLE_STYLE),
            Text::raw(""),
        ]);

        items.extend(
            shadow::sfsu::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Text::from(format!("{name} ({url})"))),
        );

        items.extend(vec![
            Text::raw(""),
            Text::styled(titles::SPRINKLES_CONTRIBUTORS, TITLE_STYLE),
            Text::raw(""),
        ]);

        items.extend(
            shadow::sprinkles::CONTRIBUTORS
                .into_iter()
                .map(|(name, url)| Text::from(format!("{name} ({url})"))),
        );

        if self.packages {
            items.extend(vec![
                Text::raw(""),
                Text::styled(titles::PACKAGES, TITLE_STYLE),
                Text::raw(""),
            ]);

            items.extend(shadow::PACKAGES.into_iter().map(|(name, version)| {
                Text::from(Line::from(vec![
                    Span::styled(name, Style::default()),
                    Span::raw(" - "),
                    Span::styled(version, Style::default().fg(Color::Yellow)),
                ]))
            }));
        }

        let mut items: VecDeque<Text<'_>> = items.into();

        let timer = if items.len() > rows {
            Some(Timer::new(Duration::from_millis(743)))
        } else {
            None
        };

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|f| {
                should_quit = Self::ui(f, timer.as_ref(), titles::TITLE, &mut items);
            })?;

            if !should_quit {
                should_quit = Self::handle_events(timer.as_ref())?;
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_events(draw_timer: Option<&Timer>) -> anyhow::Result<bool> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }

                if key.code == KeyCode::Down {
                    if key.kind == event::KeyEventKind::Press {
                        if let Some(draw_timer) = draw_timer {
                            draw_timer.set_timeout(Duration::from_millis(16));
                        }
                    }

                    if key.kind == event::KeyEventKind::Release {
                        if let Some(draw_timer) = draw_timer {
                            draw_timer.set_timeout(Duration::from_millis(743));
                        }
                    }
                }
            }
        }
        Ok(false)
    }

    fn ui(
        frame: &mut Frame<'_>,
        timer: Option<&Timer>,
        title: &str,
        items: &mut VecDeque<Text<'_>>,
    ) -> bool {
        if let Some(timer) = timer {
            if timer.tick() {
                items.pop_front();
            }
        }

        let mut footer = "Press Q to exit".to_string();

        if timer.is_some() {
            footer += " | Press Down to speed up the credits scrolling";
        }

        if items.is_empty() {
            return true;
        }

        frame.render_widget(
            List::new(
                items
                    .iter()
                    .cloned()
                    .map(|text| text.alignment(Alignment::Center)),
            )
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .title_bottom(footer)
                    .borders(Borders::ALL),
            ),
            frame.area(),
        );

        false
    }
}
