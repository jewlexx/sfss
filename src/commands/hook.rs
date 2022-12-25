use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long, help = "Disable the `scoop search` hook")]
    no_search: bool,

    #[clap(long, help = "Disable the `scoop list` hook")]
    no_list: bool,
}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        print!("function scoop {{ ");

        if !self.no_search {
            print!(
                "if ($args[0] -eq 'search') {{ sfss.exe @($args | Select-Object -Skip 1) }} else"
            );
        }

        if !self.no_list {
            print!("if ($args[0] -eq 'list') {{ sfsl.exe --json @($args | Select-Object -Skip 1) | ConvertFrom-Json }} else");
        }

        print!(" {{ scoop.ps1 @args }} }}");

        Ok(())
    }
}
