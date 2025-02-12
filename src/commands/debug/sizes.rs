use clap::Parser;
use sprinkles::contexts::ScoopContext;

use crate::commands;

#[derive(Debug, Clone, Parser)]
pub struct Args;

impl super::Command for Args {
    async fn runner(self, _ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let bucket_args_size = std::mem::size_of::<commands::bucket::Args>();
        let bucket_add_size = std::mem::size_of::<commands::bucket::add::Args>();
        let bucket_list_size = std::mem::size_of::<commands::bucket::list::Args>();
        let bucket_remove_size = std::mem::size_of::<commands::bucket::remove::Args>();
        let bucket_unused_size = std::mem::size_of::<commands::bucket::unused::Args>();
        let cache_remove_size = std::mem::size_of::<commands::cache::remove::Args>();
        let cache_list_size = std::mem::size_of::<commands::cache::list::Args>();
        let credits_size = std::mem::size_of::<commands::credits::Args>();
        let debug_save_size = std::mem::size_of::<commands::debug::save::Args>();
        let debug_sizes = std::mem::size_of::<commands::debug::sizes::Args>();
        let depends_size = std::mem::size_of::<commands::depends::Args>();
        let download_size = std::mem::size_of::<commands::app::download::Args>();
        let export_size = std::mem::size_of::<commands::export::Args>();
        let home_size = std::mem::size_of::<commands::app::home::Args>();
        let info_size = std::mem::size_of::<commands::app::info::Args>();
        let list_size = std::mem::size_of::<commands::app::list::Args>();
        let outdated_size = std::mem::size_of::<commands::outdated::Args>();
        let search_size = std::mem::size_of::<commands::search::Args>();
        let status_size = std::mem::size_of::<commands::status::Args>();
        let uninstall_size = std::mem::size_of::<commands::uninstall::Args>();
        let update_size = std::mem::size_of::<commands::bucket::update::Args>();
        let virustotal_size = std::mem::size_of::<commands::virustotal::Args>();

        let command_args_size = std::mem::size_of::<commands::Commands>();

        println!("Commands: Args = {command_args_size}");
        println!(
            "Bucket: Args = {bucket_args_size}, Add = {bucket_add_size}, List = {bucket_list_size}, Remove = {bucket_remove_size}, Unused = {bucket_unused_size}"
        );
        println!("Cache: Remove = {cache_remove_size}, List = {cache_list_size}");
        println!("Credits: Args = {credits_size}");
        println!("Debug: Save = {debug_save_size}, Sizes = {debug_sizes}");
        println!("Depends: Args = {depends_size}");
        println!("Download: Args = {download_size}");
        println!("Export: Args = {export_size}");
        println!("Home: Args = {home_size}");
        println!("Info: Args = {info_size}");
        println!("List: Args = {list_size}");
        println!("Outdated: Args = {outdated_size}");
        println!("Search: Args = {search_size}");
        println!("Status: Args = {status_size}");
        println!("Uninstall: Args = {uninstall_size}");
        println!("Update: Args = {update_size}");
        println!("VirusTotal: Args = {virustotal_size}");

        Ok(())
    }
}
