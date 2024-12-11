//! Much of this code is copied from the prodash examples
//! <https://github.com/Byron/prodash/blob/main/examples/units.rs>

use std::{future::Future, io::IsTerminal, sync::Arc, time::Duration};

use futures::{FutureExt, StreamExt};
use prodash::{
    render::{
        line,
        tui::{self, ticker, Event, Interrupt},
    },
    tree::Root as Tree,
};
use tokio::task;

use crate::warning;

enum Direction {
    Shrink,
    Grow,
}

pub enum Renderer {
    Line,
    Tui,
}

impl Renderer {
    pub fn pick() -> Self {
        if std::io::stderr().is_terminal() {
            Self::Tui
        } else {
            Self::Line
        }
    }

    pub fn launch_ambient_gui(
        self,
        progress: Arc<Tree>,
        throughput: bool,
        title: String,
    ) -> std::result::Result<task::JoinHandle<()>, std::io::Error> {
        let render_fut = self.run(progress, throughput, title)?;

        let handle = task::spawn(render_fut);

        Ok(handle)
    }

    pub fn run(
        self,
        progress: Arc<Tree>,
        throughput: bool,
        title: String,
    ) -> std::io::Result<std::pin::Pin<Box<dyn Future<Output = ()> + Send>>> {
        let mut ticks: usize = 0;
        let mut interruptible = true;

        let fut = match self {
            Renderer::Line => async move {
                let mut handle = line::render(
                    std::io::stderr(),
                    Arc::downgrade(&progress),
                    line::Options {
                        keep_running_if_progress_is_empty: true,
                        throughput,
                        ..Default::default()
                    }
                    .auto_configure(line::StreamKind::Stderr),
                );
                handle.disconnect();
                task::spawn_blocking(move || handle.wait())
                    .await
                    .expect("wait for thread");
            }
            .boxed(),
            Renderer::Tui => {
                if std::io::stdout().is_terminal() {
                    tui::render_with_input(
                        std::io::stdout(),
                        Arc::downgrade(&progress),
                        tui::Options {
                            title,
                            throughput,
                            ..tui::Options::default()
                        },
                        futures::stream::select(
                            window_resize_stream(false),
                            ticker(Duration::from_secs_f32(1.0 / 10.0)).map(move |()| {
                                ticks += 1;
                                if ticks % 2 == 0 {
                                    let is_interruptible = interruptible;
                                    interruptible = !interruptible;
                                    return if is_interruptible {
                                        Event::SetInterruptMode(Interrupt::Instantly)
                                    } else {
                                        Event::SetInterruptMode(Interrupt::Deferred)
                                    };
                                }
                                Event::Tick
                            }),
                        ),
                    )?
                    .boxed()
                } else {
                    warning!("Need a terminal on stdout to draw progress TUI");
                    futures::future::ready(()).boxed()
                }
            }
        };

        Ok(fut)
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn window_resize_stream(animate: bool) -> impl futures::Stream<Item = Event> {
    let mut offset_xy = (0u16, 0u16);
    let mut direction = Direction::Shrink;
    if !animate {
        return futures::stream::pending().boxed();
    }

    ticker(Duration::from_millis(100))
        .map(move |()| {
            let (width, height) = crossterm::terminal::size().unwrap_or((30, 30));
            let (ref mut ofs_x, ref mut ofs_y) = offset_xy;
            let min_size = 2;
            match direction {
                Direction::Shrink => {
                    *ofs_x = ofs_x.saturating_add(
                        (1_f32 * (f32::from(width) / f32::from(height))).ceil() as u16,
                    );
                    *ofs_y = ofs_y.saturating_add(
                        (1_f32 * (f32::from(height) / f32::from(width))).ceil() as u16,
                    );
                }
                Direction::Grow => {
                    *ofs_x = ofs_x.saturating_sub(
                        (1_f32 * (f32::from(width) / f32::from(height))).ceil() as u16,
                    );
                    *ofs_y = ofs_y.saturating_sub(
                        (1_f32 * (f32::from(height) / f32::from(width))).ceil() as u16,
                    );
                }
            }
            let bound = tui::tui_export::layout::Rect {
                x: 0,
                y: 0,
                width: width.saturating_sub(*ofs_x).max(min_size),
                height: height.saturating_sub(*ofs_y).max(min_size),
            };
            if bound.area() <= min_size * min_size || bound.area() == width * height {
                direction = match direction {
                    Direction::Grow => Direction::Shrink,
                    Direction::Shrink => Direction::Grow,
                };
            }
            Event::SetWindowSize(bound)
        })
        .boxed()
}
