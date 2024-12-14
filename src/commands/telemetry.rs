use clap::Parser;
use sprinkles::contexts::ScoopContext;

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum TelemetryOption {
    On,
    Off,
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "Enable or disable telemetry")]
    option: TelemetryOption,
}

impl super::Command for Args {
    async fn runner(self, _ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let mut config = crate::config::Config::load()?;
        match self.option {
            TelemetryOption::On => {
                config.enable_telemetry();
                config.save()?;
                println!("Telemetry enabled");
                println!("You can opt-out of telemetry by setting the `SFSU_TELEMETRY_DISABLED` environment variable to `1`, by passing the `--no-telemetry` flag, or running `sfsu telemetry off`. Read more about telemetry at https://github.com/winpax/sfsu/blob/trunk/TELEMETRY.md");
                println!("Thank you for helping us make sfsu better!");
            }
            TelemetryOption::Off => {
                config.disable_telemetry();
                config.save()?;
                println!("Telemetry disabled");
            }
        }

        Ok(())
    }
}
