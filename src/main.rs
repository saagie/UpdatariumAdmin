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
        Command::List { raw } => list_database_names(raw).await,
        Command::History {
            raw,
            database,
            all
        } => show_history_for(raw, database, all).await,
        Command::Info {
            raw,
            database,
            changeset_id,
        } => info_for(raw, database, changeset_id).await,
        Command::Fix {
            raw,
            database,
            changeset_id,
            author,
            comment,
        } => {
            create_new_document_from_existing(
                raw,
                database,
                changeset_id,
                author,
                comment,
                "MANUAL_OK".into(),
            )
            .await
        }

        Command::Retry {
            raw,
            database,
            changeset_id,
            author,
            comment,
        } => {
            create_new_document_from_existing(
                raw,
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
        raw: bool,
    },
    /// History of changes
    History {
        /// Display in raw format
        #[structopt(short, long)]
        raw: bool,

        /// database selected
        #[structopt(short, long)]
        database: String,

         /// No filter - display all status
         #[structopt(short, long)]
         all: bool,
    },
    /// Display information about a changeset
    Info {
        /// Display in raw format
        #[structopt(short, long)]
        raw: bool,

        /// database selected
        #[structopt(short, long)]
        database: String,

        /// changeset id
        #[structopt(short, long)]
        changeset_id: String,
    },
    /// Fix a failed changeset - this changeset will not be run at the next try
    Fix {
        /// Display in raw format
        #[structopt(short, long)]
        raw: bool,

        /// database selected
        #[structopt(short, long)]
        database: String,

        /// changeset id
        #[structopt(short, long)]
        changeset_id: String,

        /// author of the fix (you !) 
        #[structopt(short, long)]
        author: String,

        /// Describe the problem, the fix and what you want to say about the fix of the fail
        #[structopt(long)]
        comment: Vec<String>,
    },

    /// Mark as retry a failed changeset - this changeset will be run at the next try
    Retry {
        /// Display in raw format
        #[structopt(short, long)]
        raw: bool,

        /// database selected
        #[structopt(short, long)]
        database: String,

        /// changeset id
        #[structopt(short, long)]
        changeset_id: String,

        /// author of the fix (you !) 
        #[structopt(short, long)]
        author: String,

        /// Describe the problem and what you want to say about this retry
        #[structopt(long)]
        comment: Vec<String>,
    },

    /// Display logs for a changeset run
    Logs {
        /// database selected
        #[structopt(short, long)]
        database: String,

        /// id
        #[structopt(short, long)]
        id: String,
    },
}
