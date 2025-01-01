use human_panic::{setup_panic, Metadata};

pub fn handle() {
    setup_panic! {
        Metadata::new(env!("CARGO_PKG_NAME"), crate::versions::SFSU_LONG_VERSION)
            .authors(env!("CARGO_PKG_AUTHORS").replace(':', ", "))
            .homepage(env!("CARGO_PKG_HOMEPAGE"))
            .support("Open an issue on GitHub: https://github.com/winpax/sfsu/issues/new, and upload the aforementioned report file.")
    };
}
