use futures::stream::StreamExt;
use log::{error, info};

use changeset::Changeset;
use mongodb::{
    bson::{self, doc, Bson},
    error::Error,
    options::{ClientOptions, Credential, FindOptions, StreamAddress},
    Client,
};
use std::process;

mod changeset;

extern crate log;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

pub async fn list_database_names(raw_format: bool) -> Result<(), Error> {
    let databases = mongo_list_updatarium_databases().await;
    if raw_format {
        display_databases_in_raw(databases);
    } else {
        display_databases_in_table(databases);
    }
    Ok(())
}

pub async fn show_history_for(raw_format: bool, database: String) -> Result<(), Error> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let filter = doc! {};
    let find_options = FindOptions::builder().sort(doc! { "lockDate": 1 }).build();

    let collection = db.collection("changeset");
    let mut cursor = collection.find(filter, find_options).await?;
    let mut vec = Vec::new();
    while let Some(doc) = cursor.next().await {
        let changeset: Changeset = bson::from_bson(Bson::Document(doc?))?;
        vec.push(changeset);
    }
    info!("History from {}", database);
    if raw_format {
        display_changesets_in_raw(vec);
    } else {
        display_changesets_in_table(vec);
    }

    Ok(())
}

pub async fn info_for(
    raw_format: bool,
    database: String,
    changeset_id: String,
) -> Result<(), Error> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let collection = db.collection("changeset");
    let doc_filter = doc! { "_id": bson::oid::ObjectId::with_string(&changeset_id).unwrap()};
    let cursor = collection.find_one(Some(doc_filter), None).await?;
    match cursor {
        Some(doc) => {
            let changeset: Changeset = bson::from_bson(Bson::Document(doc))?;

            if raw_format {
                display_changeset_in_raw(database, changeset)
            } else {
                display_changeset_in_table(database, changeset)
            }
        }
        None => error!(
            "Changeset {} on database {} not found.",
            changeset_id, database
        ),
    }

    Ok(())
}

fn get_mongo_client() -> Result<Client, Error> {
    let mongo_root_pwd = std::env::var("MONGODB_ROOT_PASSWD");
    if mongo_root_pwd.is_err() {
        error!("env var `MONGODB_ROOT_PASSWD` was not initialized.");
        process::exit(1);
    }

    let mongo_username: String = "root".into();
    let mongo_password: String = mongo_root_pwd.unwrap();
    let client_options = ClientOptions::builder()
        .hosts(vec![StreamAddress {
            hostname: "localhost".into(),
            port: Some(27017),
        }])
        .credential(
            Credential::builder()
                .username(mongo_username)
                .password(mongo_password)
                .build(),
        )
        .build();

    Client::with_options(client_options)
}

fn display_changesets_in_table(changesets: Vec<Changeset>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Changeset ID")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Author")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Status")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Date")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
        ]);
    for c in changesets {
        table.add_row(vec![
            Cell::new(format!("{} ", c.id)).fg(get_status_color(&c.status)),
            Cell::new(format!("{} ", c.change_set_id)),
            Cell::new(format!("{} ", c.author)),
            Cell::new(format!("{} ", c.status)).fg(get_status_color(&c.status)),
            Cell::new(format!("{} ", c.lock_date.to_rfc3339())),
        ]);
    }

    println!("{}", table);
}

fn display_changesets_in_raw(changesets: Vec<Changeset>) {
    info!("ID - Changeset ID - Author - Status - Date");
    for c in changesets {
        info!(
            "{} - {} - {} - {} - {}",
            c.id,
            c.change_set_id,
            c.author,
            c.status,
            c.lock_date.to_rfc3339()
        );
    }
}

fn display_changeset_in_raw(database: String, changeset: Changeset) {
    info!("Changeset {} from database {}", changeset.id, database);
    info!("{:#?}", changeset);
}

fn display_changeset_in_table(database: String, c: Changeset) {
    info!("Changeset {} from database {}", c.id, database);

    let mut table = Table::new();

    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Key")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Value")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
        ]);

    table.add_row(vec![Cell::new("ID"), Cell::new(format!("{} ", c.id))]);
    table.add_row(vec![
        Cell::new("Changetset ID "),
        Cell::new(format!("{} ", c.change_set_id)),
    ]);
    table.add_row(vec![
        Cell::new("Author"),
        Cell::new(format!("{} ", c.author)),
    ]);
    table.add_row(vec![
        Cell::new("Status"),
        Cell::new(format!("{} ", c.status)).fg(get_status_color(&c.status)),
    ]);
    table.add_row(vec![
        Cell::new("Lock date"),
        Cell::new(format!("{} ", c.lock_date.to_rfc3339())),
    ]);
    table.add_row(vec![
        Cell::new("Status date"),
        Cell::new(match c.status_date {
            Some(status_date) => status_date.to_rfc3339(),
            None => "Null".into(),
        }),
    ]);

    table.add_row(vec![Cell::new("Force"), Cell::new(format!("{} ", c.force))]);

    println!("{}", table);
}
fn get_status_color(status: &str) -> Color {
    match status {
        "OK" => Color::Green,
        "FAIL" => Color::Red,
        _ => Color::White,
    }
}
fn display_databases_in_table(databases: Vec<String>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![Cell::new("Database")
            .fg(Color::DarkMagenta)
            .add_attribute(Attribute::Bold)]);
    for d in databases {
        table.add_row(vec![format!("> {} ", d)]);
    }
    println!("{}", table);
}

fn display_databases_in_raw(databases: Vec<String>) {
    info!("Databases:");
    for d in databases {
        info!("> {} ", d);
    }
}

async fn mongo_list_updatarium_databases() -> Vec<String> {
    let client = get_mongo_client().expect("Client should be created");
    let databases: Vec<String> = client
        .list_database_names(None, None)
        .await
        .unwrap()
        .into_iter()
        .filter(|name| name.to_lowercase().contains("updatarium"))
        .collect();

    databases
}
