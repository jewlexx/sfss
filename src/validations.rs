pub mod config;

pub trait Validate {
    fn validate(&self) -> anyhow::Result<()>;
}
