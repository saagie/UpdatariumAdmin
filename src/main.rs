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
        Command::List { raw_format } => list_database_names(raw_format).await,
        Command::History {
            raw_format,
            database,
        } => show_history_for(raw_format, database).await,
    }
    .expect("COMMAND FAILED");
}

#[derive(Debug, StructOpt, PartialEq)]
enum Command {
    /// List all availables databases
    List {
        /// Display in raw format
        #[structopt(short, long)]
        raw_format: bool,
    },
    /// History of changes
    History {
        /// Display in raw format
        #[structopt(short, long)]
        raw_format: bool,

        #[structopt(name = "database", long = "database", short = "d")]
        database: String,
    },
}
