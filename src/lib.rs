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

pub async fn list_database_names() -> Result<(), Error> {
    let databases = mongo_list_updatarium_databases().await;
    display_databases_in_table(databases);
    Ok(())
}

pub async fn show_history_for(database: String) -> Result<(), Error> {
    info!("History from {}", database);
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
    display_changesets_in_table(vec);

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
        ]);
    for c in changesets {
        table.add_row(vec![
            Cell::new(format!("{} ", c.id)),
            Cell::new(format!("{} ", c.change_set_id)),
            Cell::new(format!("{} ", c.author)),
            Cell::new(format!("{} ", c.status)).fg(get_status_color(&c.status)),
        ]);
    }

    println!("{}", table);
}

fn get_status_color(status: &str) -> Color {
    match status {
        "OK" => Color::Green,
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
