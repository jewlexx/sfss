use clap::Parser;
use quork::traits::list::ListVariants;
use sprinkles::{contexts::ScoopContext, shell::Shell};

use super::CommandHooks as CommandsHooks;

#[derive(Debug, Clone, Parser)]
/// Generate hooks for the given shell
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<CommandsHooks>,

    #[clap(short = 'E', long, help = "The commands to exclusively enable")]
    enabled: Vec<CommandsHooks>,

    #[clap(short, long, help = "Print hooks for the given shell", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl super::Command for Args {
    async fn runner(self, _: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let shell = self.shell;
        let shell_config = shell.config();
        let enabled_hooks: Vec<CommandsHooks> = {
            // Explicit binding here fixes type inference, as we explicitly cast it to a slice
            let enabled_hooks: &[CommandsHooks] = if self.enabled.is_empty() {
                &CommandsHooks::VARIANTS
            } else {
                &self.enabled
            };

            enabled_hooks
        }
        .iter()
        .filter(|variant| !self.disable.contains(variant))
        .copied()
        .collect();

        match shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                // I would love to make this all one condition, but Powershell doesn't seem to support that elegantly
                for command in enabled_hooks {
                    print!(
                        "  '{hook}' {{ return sfsu.exe {command} @($args | Select-Object -Skip 1) }} ",
                        hook = command.hook(),
                        command = command.command()
                    );
                }

                println!("default {{ scoop.ps1 @args }} }} }}");

                // TODO: Figure out a way to put these in that PowerShell won't throw a fit about
                // println!("# To add this to your config, add the following line to the end of your PowerShell profile:");
                // println!("#     Invoke-Expression (&sfsu hook)");
            }
            Shell::Bash | Shell::Zsh => {
                println!(
                    "SCOOP_EXEC=$(which scoop) \n\
                    scoop () {{ \n\
                    case $1 in"
                );

                for command in enabled_hooks {
                    println!(
                        "({hook}) sfsu.exe {command} ${{@:2}} ;;",
                        hook = command.hook(),
                        command = command.command()
                    );
                }

                println!(
                    "(*) $SCOOP_EXEC $@ ;; \n\
                    esac \n\
                    }} \n\n\
                    # Add the following to the end of your ~/.{shell_config} \n\
                    #   source <(sfsu.exe hook --shell {shell})"
                );
            }
            Shell::Nu => {
                for command in enabled_hooks {
                    println!(
                        "def --wrapped \"scoop {hook}\" [...rest] {{ sfsu {command} ...$rest }}",
                        hook = command.hook(),
                        command = command.command()
                    );
                }

                println!(
                    "\n# To add this to your config, run `sfsu hook --shell {shell} | save ~/.cache/sfsu.nu`\n\
                        # And then in your {shell_config} add the following line to the end:\n\
                        #   source ~/.cache/sfsu.nu"
                );
            }
        }

        Ok(())
    }
}
