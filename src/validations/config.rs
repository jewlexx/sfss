impl super::Validate for sprinkles::config::Scoop {
    fn validate(&self) -> anyhow::Result<()> {
        if self.no_junction {
            anyhow::bail!("Junction links (symlinks) are required for sfsu to function currently. Please disable `no_junction` in your Scoop
            config");
        }

        Ok(())
    }
}
