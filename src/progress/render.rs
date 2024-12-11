use std::{future::Future, io::IsTerminal, ops::RangeInclusive, sync::Arc, time::Duration};

use futures::{FutureExt, StreamExt};
use prodash::{
    render::{
        line,
        tui::{self, ticker, Event, Interrupt, Line},
    },
    tree::Root as Tree,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

pub mod args;
use tokio::task;

use crate::warning;

enum Direction {
    Shrink,
    Grow,
}

const TITLES: &[&str] = &[" Dashboard Demo ", " 仪表板演示 "];

enum Renderer {
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

    pub fn run(
        self,
        progress: Arc<Tree>,
        throughput: bool,
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
                            title: TITLES.choose(&mut thread_rng()).copied().unwrap().into(),
                            throughput,
                            ..tui::Options::default()
                        },
                        futures::stream::select(
                            window_resize_stream(false),
                            ticker(Duration::from_secs_f32(1.0)).map(move |()| {
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
                                if thread_rng().gen_bool(0.5) {
                                    Event::SetTitle(
                                        (*TITLES.choose(&mut thread_rng()).unwrap()).to_string(),
                                    )
                                } else {
                                    Event::SetInformation(generate_statistics())
                                }
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

pub fn launch_ambient_gui(
    progress: Arc<Tree>,
    throughput: bool,
) -> std::result::Result<task::JoinHandle<()>, std::io::Error> {
    let renderer = Renderer::pick();

    let render_fut = renderer.run(progress, throughput)?;

    let handle = task::spawn(render_fut);

    Ok(handle)
}

fn generate_statistics() -> Vec<Line> {
    let mut lines = vec![
        Line::Text("You can put here what you want".into()),
        Line::Text("as long as it fits one line".into()),
        Line::Text("until a certain limit is reached".into()),
        Line::Text("which is when truncation happens".into()),
        Line::Text("这是中文的一些文字。".into()),
        Line::Text("鹅、鹅、鹅 曲项向天歌 白毛浮绿水 红掌拨清波".into()),
        Line::Text("床前明月光, 疑是地上霜。举头望明月，低头思故乡。".into()),
        Line::Text("锄禾日当午，汗滴禾下土。谁知盘中餐，粒粒皆辛苦。".into()),
        Line::Text("春眠不觉晓，处处闻啼鸟。夜来风雨声，花落知多少".into()),
        Line::Text("煮豆燃豆萁，豆在釜中泣。本自同根生，相煎何太急".into()),
        Line::Text(
            "and this line is without any doubt very very long and it really doesn't want to stop"
                .into(),
        ),
    ];
    lines.shuffle(&mut thread_rng());
    lines.insert(0, Line::Title("Hello World".into()));

    lines.extend(vec![
        Line::Title("Statistics".into()),
        Line::Text(format!(
            "lines of unsafe code: {}",
            thread_rng().gen_range(0usize..=1_000_000)
        )),
        Line::Text(format!(
            "wasted space in crates: {} Kb",
            thread_rng().gen_range(100usize..=1_000_000)
        )),
        Line::Text(format!(
            "unused dependencies: {} crates",
            thread_rng().gen_range(100usize..=1_000)
        )),
        Line::Text(format!(
            "average #dependencies: {} crates",
            thread_rng().gen_range(0usize..=500)
        )),
        Line::Text(format!(
            "bloat in code: {} Kb",
            thread_rng().gen_range(100usize..=5_000)
        )),
    ]);
    lines
}

fn window_resize_stream(animate: bool) -> impl futures::Stream<Item = Event> {
    let mut offset_xy = (0u16, 0u16);
    let mut direction = Direction::Shrink;
    if !animate {
        return futures::stream::pending().boxed();
    }

    ticker(Duration::from_millis(100))
        .map(move |_| {
            let (width, height) = crossterm::terminal::size().unwrap_or((30, 30));
            let (ref mut ofs_x, ref mut ofs_y) = offset_xy;
            let min_size = 2;
            match direction {
                Direction::Shrink => {
                    *ofs_x = ofs_x
                        .saturating_add((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y
                        .saturating_add((1_f32 * (height as f32 / width as f32)).ceil() as u16);
                }
                Direction::Grow => {
                    *ofs_x = ofs_x
                        .saturating_sub((1_f32 * (width as f32 / height as f32)).ceil() as u16);
                    *ofs_y = ofs_y
                        .saturating_sub((1_f32 * (height as f32 / width as f32)).ceil() as u16);
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
