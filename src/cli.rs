use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: PubsubCommands,
    // #[arg(
    //     long,
    //     env("PUBSUB_EMULATOR_HOST"),
    //     help = "address of pubsub emulator host [example: http://localhost:port]"
    // )]
    // pub host: String,
}

#[derive(Subcommand)]
pub enum PubsubCommands {
    #[command(subcommand)]
    Topics(TopicCommands),

    #[command()]
    Init(InitArgs),
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    #[arg(
        short,
        long,
        help = "path to file containing initial state",
        value_name = "FILE_PATH"
    )]
    pub file: String,

    #[arg(
        short,
        long,
        value_name = "SECONDS",
        default_value = "5",
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "how long (in seconds) to poll the pubsub emulator host before giving up."
    )]
    pub timeout: u8,
}

#[derive(Subcommand)]
pub enum TopicCommands {
    Create {
        #[arg(required=true, num_args= 1..)]
        names: Vec<String>,
    },
    List,
    Info {
        name: String,
    },
    Delete {
        name: String,
    },
}
