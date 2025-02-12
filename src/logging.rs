use std::{fs::File, io::Write, path::PathBuf};

use anyhow::Context;
use chrono::Local;
use log::{Level, LevelFilter};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sprinkles::contexts::ScoopContext;

use crate::output::colours::{eprintln_red, eprintln_yellow};

pub mod panics;

pub struct Logger {
    file: Option<File>,
    verbose: bool,
}

#[allow(dead_code)]
impl Logger {
    const LEVEL_FILTER: LevelFilter = LevelFilter::Trace;

    pub async fn new(ctx: &impl ScoopContext, verbose: bool) -> Self {
        let file = async move {
            let logs_dir = if cfg!(debug_assertions) {
                let dir = std::env::current_dir().unwrap().join("logs");
                if !dir.exists() {
                    std::fs::create_dir(&dir).unwrap();
                }

                dir
            } else {
                ctx.logging_dir()?
            };
            let date = Local::now();
            let log_file = async {
                let mut i = 0;
                loop {
                    i += 1;

                    let log_path =
                        logs_dir.join(format!("sfsu-{}-{i}.log", date.format("%Y-%m-%d-%H-%M-%S")));

                    if !log_path.exists() {
                        break File::create(log_path);
                    }
                }
            };
            let timeout = async {
                use std::time::Duration;
                use tokio::time;

                time::sleep(Duration::from_secs(5)).await;
            };
            let log_file = tokio::select! {
                res = log_file => anyhow::Ok(res),
                () = timeout => anyhow::bail!("Timeout creating new log"),
            }??;

            anyhow::Ok(log_file)
        }
        .await
        .ok();

        Self::from_file(file, verbose)
    }

    pub fn from_file(file: Option<File>, verbose: bool) -> Self {
        Self { file, verbose }
    }

    pub async fn init(ctx: &impl ScoopContext, verbose: bool) -> Result<(), log::SetLoggerError> {
        log::set_boxed_logger(Box::new(Logger::new(ctx, verbose).await))?;
        log::set_max_level(Self::LEVEL_FILTER);

        debug!("Initialized logger");

        Ok(())
    }

    pub fn cleanup_logs(ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let logging_dir = ctx.logging_dir()?;

        // Cleanup legacy log paths
        let legacy_logs_dirs: &[PathBuf] =
            &[ctx.apps_path().join("sfsu").join("current").join("logs")];

        for legacy_dir in legacy_logs_dirs {
            if legacy_dir.exists() {
                // Copy all files to the new location
                for entry in std::fs::read_dir(legacy_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .context("missing file name for log entry")?
                        .to_string_lossy();

                    let new_path = logging_dir.join(name.as_ref());

                    if !new_path.exists() {
                        std::fs::rename(&path, &new_path)?;
                    }
                }

                // Remove the old directory
                std::fs::remove_dir_all(legacy_dir)?;
            }
        }

        let logs = std::fs::read_dir(logging_dir)?.collect::<Result<Vec<_>, _>>()?;

        logs.into_iter()
            .rev()
            .skip(10)
            .par_bridge()
            .try_for_each(|entry| std::fs::remove_file(entry.path()))?;

        Ok(())
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if self.verbose {
            true
        } else {
            metadata.level() < Level::Debug
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            match record.metadata().level() {
                Level::Error => eprintln_red!("ERROR: {}", record.args()),
                Level::Warn => eprintln_yellow!("WARN: {}", record.args()),
                _ => {
                    if let Some(mut file) = self.file.as_ref() {
                        // TODO: Add a queue of sorts because this doesn't work well with multiple threads
                        writeln!(file, "{}: {}", record.level(), record.args())
                            .expect("writing to log file");
                    }
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = self.file.as_ref() {
            file.try_clone()
                .expect("cloning log file")
                .flush()
                .expect("flushing log file");
        }
    }
}

pub mod macros {
    /// Prints and returns the value of a given expression for quick and dirty
    /// debugging.
    ///
    /// An example:
    ///
    /// ```rust
    /// let a = 2;
    /// let b = dbg!(a * 2) + 1;
    /// //      ^-- prints: [src/main.rs:2:9] a * 2 = 4
    /// assert_eq!(b, 5);
    /// ```
    ///
    /// The macro works by using the `Debug` implementation of the type of
    /// the given expression to print the value to [stderr] along with the
    /// source location of the macro invocation as well as the source code
    /// of the expression.
    ///
    /// Invoking the macro on an expression moves and takes ownership of it
    /// before returning the evaluated expression unchanged. If the type
    /// of the expression does not implement `Copy` and you don't want
    /// to give up ownership, you can instead borrow with `dbg!(&expr)`
    /// for some expression `expr`.
    ///
    /// The `dbg!` macro works exactly the same in release builds.
    /// This is useful when debugging issues that only occur in release
    /// builds or when debugging in release mode is significantly faster.
    ///
    /// Note that the macro is intended as a debugging tool and therefore you
    /// should avoid having uses of it in version control for long periods
    /// (other than in tests and similar).
    /// Debug output from production code is better done with other facilities
    /// such as the [`debug!`] macro from the [`log`] crate.
    ///
    /// # Stability
    ///
    /// The exact output printed by this macro should not be relied upon
    /// and is subject to future changes.
    ///
    /// # Panics
    ///
    /// Panics if writing to `io::stderr` fails.
    ///
    /// # Further examples
    ///
    /// With a method call:
    ///
    /// ```rust
    /// fn foo(n: usize) {
    ///     if let Some(_) = dbg!(n.checked_sub(4)) {
    ///         // ...
    ///     }
    /// }
    ///
    /// foo(3)
    /// ```
    ///
    /// This prints to [stderr]:
    ///
    /// ```text,ignore
    /// [src/main.rs:2:22] n.checked_sub(4) = None
    /// ```
    ///
    /// Naive factorial implementation:
    ///
    /// ```rust
    /// fn factorial(n: u32) -> u32 {
    ///     if dbg!(n <= 1) {
    ///         dbg!(1)
    ///     } else {
    ///         dbg!(n * factorial(n - 1))
    ///     }
    /// }
    ///
    /// dbg!(factorial(4));
    /// ```
    ///
    /// This prints to [stderr]:
    ///
    /// ```text,ignore
    /// [src/main.rs:2:8] n <= 1 = false
    /// [src/main.rs:2:8] n <= 1 = false
    /// [src/main.rs:2:8] n <= 1 = false
    /// [src/main.rs:2:8] n <= 1 = true
    /// [src/main.rs:3:9] 1 = 1
    /// [src/main.rs:7:9] n * factorial(n - 1) = 2
    /// [src/main.rs:7:9] n * factorial(n - 1) = 6
    /// [src/main.rs:7:9] n * factorial(n - 1) = 24
    /// [src/main.rs:9:1] factorial(4) = 24
    /// ```
    ///
    /// The `dbg!(..)` macro moves the input:
    ///
    /// ```compile_fail
    /// /// A wrapper around `usize` which importantly is not Copyable.
    /// #[derive(Debug)]
    /// struct NoCopy(usize);
    ///
    /// let a = NoCopy(42);
    /// let _ = dbg!(a); // <-- `a` is moved here.
    /// let _ = dbg!(a); // <-- `a` is moved again; error!
    /// ```
    ///
    /// You can also use `dbg!()` without a value to just print the
    /// file and line whenever it's reached.
    ///
    /// Finally, if you want to `dbg!(..)` multiple values, it will treat them as
    /// a tuple (and return it, too):
    ///
    /// ```
    /// assert_eq!(dbg!(1usize, 2u32), (1, 2));
    /// ```
    ///
    /// However, a single argument with a trailing comma will still not be treated
    /// as a tuple, following the convention of ignoring trailing commas in macro
    /// invocations. You can use a 1-tuple directly if you need one:
    ///
    /// ```
    /// assert_eq!(1, dbg!(1u32,)); // trailing comma ignored
    /// assert_eq!((1,), dbg!((1u32,))); // 1-tuple
    /// ```
    ///
    /// [stderr]: https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)
    /// [`debug!`]: https://docs.rs/log/*/log/macro.debug.html
    /// [`log`]: https://crates.io/crates/log
    #[macro_export]
    macro_rules! ddbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        debug!("[{}:{}:{}]", file!(), line!(), column!())
    };
    ($val:expr_2021 $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                debug!("[{}:{}:{}] {} = {:#?}",
                    file!(), line!(), column!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr_2021),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

    pub use ddbg;
}
