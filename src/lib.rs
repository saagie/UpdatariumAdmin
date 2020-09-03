extern crate log;

use std::process;

use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use futures::stream::StreamExt;
use log::{error, info};
use mongodb::{
    bson::{self, doc, Bson},
    error::Error,
    options::{ClientOptions, Credential, FindOptions, StreamAddress},
    Client,
};
use Attribute::{Bold, Dim};

use changeset::Changeset;
use chrono::Utc;
use mongodb::options::FindOneOptions;

mod changeset;

#[macro_use]
extern crate anyhow;

pub async fn list_database_names(raw: bool) -> anyhow::Result<()> {
    let databases = mongo_list_updatarium_databases().await;
    if raw {
        display_databases_in_raw(databases);
    } else {
        display_databases_in_table(databases);
    }
    Ok(())
}

pub async fn show_history_for(raw: bool, database: String, all: bool) -> anyhow::Result<()> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let filter = if all {
        doc! {}
    } else {
        doc! {"status": { "$ne":"EXECUTE"}}
    };
    let find_options = FindOptions::builder().sort(doc! { "lockDate": 1 }).build();

    let collection = db.collection("changeset");
    let mut cursor = collection.find(filter, find_options).await?;
    let mut vec = Vec::new();
    while let Some(doc) = cursor.next().await {
        let changeset: Changeset = bson::from_bson(Bson::Document(doc?))?;
        vec.push(changeset);
    }
    info!("History from {}", database);
    if raw {
        display_changesets_in_raw(vec);
    } else {
        display_changesets_in_table(vec);
    }

    Ok(())
}

pub async fn info_for(raw: bool, database: String, changeset_id: String) -> anyhow::Result<()> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let collection = db.collection("changeset");
    let doc_filter = doc! { "changeSetId": {"$regex": format!("{}.*",&changeset_id)} };
    let mut cursor = collection.find(Some(doc_filter), None).await?;
    let mut vec = Vec::new();
    while let Some(doc) = cursor.next().await {
        let changeset: Changeset = bson::from_bson(Bson::Document(doc?))?;
        vec.push(changeset);
    }
    info!("History from {}", database);
    if raw {
        display_changesets_in_raw(vec)
    } else {
        display_changesets_in_table(vec)
    }

    Ok(())
}

pub async fn create_new_document_from_existing(
    raw: bool,
    database: String,
    changeset_id: String,
    author: String,
    comment: Vec<String>,
    status: String,
) -> anyhow::Result<()> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let doc_filter = doc! { "changeSetId": &changeset_id };
    let find_options = FindOneOptions::builder()
        .sort(doc! { "lockDate": -1 })
        .build();

    let collection = db.collection("changeset");
    let opt_doc = collection.find_one(Some(doc_filter), find_options).await?;

    let _: anyhow::Result<()> = match opt_doc {
        Some(doc) => {
            let changeset: Changeset = bson::from_bson(Bson::Document(doc))?;
            ensure!(
                changeset.status == "FAIL",
                "Changeset is not in a FAIL state"
            );

            let doc = doc! {
               "changeSetId": changeset.change_set_id,
               "author": author,
               "status": status,
               "force": false,
               "lockDate": Utc::now(),
               "statusDate": Utc::now(),
               "log": comment,
            };
            let result = collection.insert_one(doc, None).await;
            if result.is_ok() {
                info_for(raw, database, changeset_id).await.unwrap();
                Ok(())
            } else {
                bail!("Unable to create a new mongodb document")
            }
        }
        None => bail!("Changeset not found"),
    };
    Ok(())
}

pub async fn logs_for(database: String, id: String) -> anyhow::Result<()> {
    let client = get_mongo_client().expect("Client should be created");
    let db = client.database(&database);

    let doc_filter = doc! { "_id": bson::oid::ObjectId::with_string(&id)? };

    let collection = db.collection("changeset");
    let opt_doc = collection.find_one(Some(doc_filter), None).await?;
    let _: anyhow::Result<()> = match opt_doc {
        Some(doc) => {
            let changeset: Changeset = bson::from_bson(Bson::Document(doc))?;
            info!("Logs : {:#?}", changeset.log);
            Ok(())
        }
        None => bail!("Changeset not found"),
    };

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
            Cell::new("Status")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Date")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Author")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
            Cell::new("Force")
                .fg(Color::DarkMagenta)
                .add_attribute(Attribute::Bold),
        ]);
    for c in changesets {
        table.add_row(vec![
            Cell::new(c.id),
            Cell::new(c.change_set_id),
            Cell::new(&c.status)
                .add_attributes(get_status_attributes(&c.status))
                .fg(get_status_color(&c.status)),
            Cell::new(c.lock_date.format("%Y-%m-%d %H:%M:%S")),
            Cell::new(c.author),
            Cell::new(c.force),
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
            c.status,
            c.lock_date.format("%Y-%m-%d %H:%M:%S"),
            c.author
        );
    }
}

fn get_status_color(status: &str) -> Color {
    match status {
        "OK" | "MANUAL_OK" => Color::Green,
        "RETRY" => Color::DarkYellow,
        "FAIL" => Color::Red,
        _ => Color::White,
    }
}

fn get_status_attributes(status: &str) -> Vec<Attribute> {
    vec![match status {
        "FAIL" => Bold,
        _ => Dim,
    }]
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
