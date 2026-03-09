use clap::{Args, Parser, Subcommand};

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
    /// A collection of helpful actions related to topics
    #[command(subcommand)]
    Topics(TopicCommands),

    /// add/delete/list config options
    #[command(subcommand)]
    Projects(ProjectCommands),

    #[command()]
    Init(InitArgs),
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    Add(ProjectArgs),
    Delete { name: String },
    List,
    SetActive { name: String },
    GetActive,
}

#[derive(Args)]
pub struct ProjectArgs {
    #[arg(
        long,
        help = "{name} of the pubsub project in /projects/{name}/topics/example_topic"
    )]
    pub name: String,
    #[arg(
        long,
        env = "PUBSUB_EMULATOR_HOST",
        help = "the ip address and port where the pubsub emulator is running"
    )]
    pub host: String,
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
    /// Attempts to create topics with the given names
    /// If a topic can not be made an error message is output, however the remaining topics will be
    /// attempted
    Create {
        #[arg(required=true, num_args= 1..)]
        names: Vec<String>,
    },
    /// Lists the topics that are associated with the project
    List,
    /// Returns a topic's fully qualified name + list of subscriptions attached to topic
    Info { name: String },
    /// Attempts to delete a topic
    Delete { name: String },
}
