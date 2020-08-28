use serde::{Deserialize, Serialize};

use bson::{oid::ObjectId, DateTime};

#[derive(Deserialize, Serialize, Debug)]
pub struct Changeset {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    #[serde(rename = "changeSetId")]
    pub change_set_id: String,
    pub author: String,
    pub status: String,
    #[serde(rename = "lockDate")]
    pub lock_date: DateTime,
    #[serde(rename = "statusDate")]
    pub status_date: Option<DateTime>,
    pub force: bool,
}
