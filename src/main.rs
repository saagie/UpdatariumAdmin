use structopt::StructOpt;
use updadmin::{list_database_names, show_history_for};

#[tokio::main]
async fn main() {
    let command = Command::from_args();

    let rust_log = std::env::var("RUST_LOG");
    if rust_log.is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    match command {
        Command::List => list_database_names().await,
        Command::History { database } => show_history_for(database).await,
    }
    .expect("COMMAND FAILED");
}

#[derive(Debug, StructOpt, PartialEq)]
enum Command {
    /// List all availables databases
    List,
    /// History of changes
    History {
        #[structopt(name = "database", long = "database", short = "d")]
        database: String,
    },
}
