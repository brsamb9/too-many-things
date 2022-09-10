pub mod cli;
pub mod error_hander;
pub mod task_object;
use log::info;

use clap::Parser;
use cli::{Cli, Commands};
use error_hander::Result;

use crate::task_object::{TaskContainer, TasksIOWrapper};

// TODO:
// - Add interesting policies: simple weighting, time based weightings, ...;
// Bonus brownie points:
// - Add import / exports of items;
// - CLI as a service - wasm runtime and hosted on the cloud?

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    // KISS w/ JSON hidden file instead of database
    info!("Obtaining file from disk");
    let mut task_interface = TaskContainer::new()?;

    // Suboptimal handling of enum variant.
    match cli.command {
        Commands::Create {
            topic,
            task,
            task_description,
        } => task_interface.handle_create(topic, task, task_description),
        Commands::Read { topic, task } => task_interface.handle_read(topic, task),
        Commands::Delete { topic, task } => task_interface.handle_delete(topic, task),
        Commands::Randomise { topic } => task_interface.handle_randomise(topic),
    }?;

    task_interface.write_to_disk()?;

    Ok(())
}
