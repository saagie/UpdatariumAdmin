use log::error;
use structopt::StructOpt;
use updadmin::{
    create_new_document_from_existing, info_for, list_database_names, show_history_for, logs_for,
};

#[tokio::main]
async fn main() {
    let command = Command::from_args();

    let rust_log = std::env::var("RUST_LOG");
    if rust_log.is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let result = match command {
        Command::List { raw_format } => list_database_names(raw_format).await,
        Command::History {
            raw_format,
            database,
        } => show_history_for(raw_format, database).await,
        Command::Info {
            raw_format,
            database,
            changeset_id,
        } => info_for(raw_format, database, changeset_id).await,
        Command::Fix {
            raw_format,
            database,
            changeset_id,
            author,
            comment,
        } => {
            create_new_document_from_existing(
                raw_format,
                database,
                changeset_id,
                author,
                comment,
                "MANUAL_OK".into(),
            )
            .await
        }

        Command::Retry {
            raw_format,
            database,
            changeset_id,
            author,
            comment,
        } => {
            create_new_document_from_existing(
                raw_format,
                database,
                changeset_id,
                author,
                comment,
                "RETRY".into(),
            )
            .await
        }
        Command::Logs { database, id } => logs_for(database, id).await,
    };

    match result {
        Err(error) => error!("{}", error),
        _ => (),
    }
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

        #[structopt(short, long)]
        database: String,
    },
    /// Display information about a changeset
    Info {
        /// Display in raw format
        #[structopt(short, long)]
        raw_format: bool,

        #[structopt(short, long)]
        database: String,

        #[structopt(short, long)]
        changeset_id: String,
    },
    /// Fix a failed changeset - this changeset will not be run at the next try
    Fix {
        /// Display in raw format
        #[structopt(short, long)]
        raw_format: bool,

        #[structopt(short, long)]
        database: String,

        #[structopt(short, long)]
        changeset_id: String,

        #[structopt(short, long)]
        author: String,

        #[structopt(long)]
        comment: Vec<String>,
    },

    /// Mark as retry a failed changeset - this changeset will be run at the next try
    Retry {
        /// Display in raw format
        #[structopt(short, long)]
        raw_format: bool,

        #[structopt(short, long)]
        database: String,

        #[structopt(short, long)]
        changeset_id: String,

        #[structopt(short, long)]
        author: String,

        #[structopt(long)]
        comment: Vec<String>,
    },

    /// Display logs for a changeset run
    Logs {
        #[structopt(short, long)]
        database: String,

        #[structopt(short, long)]
        id: String,
    },
}
