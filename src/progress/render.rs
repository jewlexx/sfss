//! Much of this code is copied from the prodash examples
//! <https://github.com/Byron/prodash/blob/main/examples/units.rs>

use std::sync::Arc;

use futures::FutureExt;
use prodash::{render::line, tree::Root as Tree};
use tokio::task;

pub struct LineRenderer;

impl LineRenderer {
    pub fn run(progress: Arc<Tree>, throughput: bool) -> task::JoinHandle<()> {
        task::spawn(
            async move {
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
        )
    }
}
