use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Create {
        #[clap(default_value_t=String::from("n/a"), long, value_parser)]
        topic: String,
        #[clap(long, value_parser)]
        task: String,
        #[clap(long, value_parser)]
        task_description: Option<String>,
        #[clap(long, value_parser)]
        link: Option<String>,
    },
    Read {
        #[clap(long, value_parser)]
        topic: Option<String>,
        #[clap(long, value_parser)]
        task: Option<String>,
    },
    // Possibly do later if worth it
    // Update {}
    Delete {
        #[clap(default_value_t=String::from("n/a"), long, value_parser)]
        topic: String,
        #[clap(long, value_parser)]
        task: Option<String>,
    },
    Randomise {
        #[clap(long, value_parser)]
        topic: Option<String>,
    },
}
