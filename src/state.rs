
use std::{collections::VecDeque, sync::Arc};

use futures::lock::Mutex;

use aws_sdk_dynamodb::types::AttributeValue;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::routes::{Node, Repo, ScanStat};

// DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub search_repos: Vec<Repo>,
    pub dash_repos: Vec<Repo>,
    pub nodes_items: VecDeque<Node>,
    pub scan_stats: Vec<ScanStat>,
    pub slots: Vec<usize>,
    pub has_repos: bool,
    pub ratelimit_count: usize,
    pub search_link: String,
    pub search_current: usize,
    pub username: String,
}

pub type DB = Arc<Mutex<Database>>;

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    pub key: [u8; 16],
    pub client_id: String,
    pub client_secret: String,
    pub hb: Handlebars<'a>,
    pub db: DB,
    pub client_s3: aws_sdk_s3::Client,
    pub client_db: aws_sdk_dynamodb::Client,
}

pub async fn put_state(client: &aws_sdk_dynamodb::Client, db: &Database) -> Option<()> {
    let request = client
        .put_item()
        .table_name("rmap_repo_user_states")
        .item("username", AttributeValue::S(db.username.clone()))
        .item(
            "states",
            AttributeValue::S(serde_json::to_string(&db).unwrap()),
        )
        .send()
        .await;

    if let Ok(put_item) = request {
        println!("Added state {:?}", put_item);
        return Some(());
    }

    let error = request.err().unwrap();

    println!("error putting state {:?}",error);

    Some(())
}

pub async fn get_state(
    client: &aws_sdk_dynamodb::Client,
    username: &str,
) -> Option<Option<Database>> {
    let response = client
        .get_item()
        .table_name("rmap_repo_user_states")
        .attributes_to_get("states")
        .key("username", AttributeValue::S(username.to_owned()))
        .send()
        .await;

    match response {
        Ok(get_item) => {
            let result = get_item.item();
            if let Some(item) = result {
                let states = item.get("states");

                if let Some(val) = states {
                    if let Ok(name_value) = val.as_s() {
                        let db: Database = serde_json::from_str(&name_value).unwrap();
                        return Some(Some(db));
                    }
                }
            }

            Some(None)
        }
        Err(_) =>{ 
            let error = response.err().unwrap();
            println!("error getting state {:?}",error);
            
            Some(None)},
    }
}

pub fn copy_db(source: &Option<Database>, dest: &mut futures::lock::MutexGuard<'_, Database>) {
    if let Some(db) = source {
        dest.username = db.username.to_string();
        dest.ratelimit_count = db.ratelimit_count;
        dest.nodes_items = db.nodes_items.clone();
        dest.scan_stats = db.scan_stats.clone();
        dest.slots = db.slots.clone();
        dest.search_current = db.search_current;
        dest.search_repos = db.search_repos.clone();
        dest.search_link = db.search_link.clone();
        dest.has_repos = db.has_repos.clone();
        dest.dash_repos = db.dash_repos.clone();
    }
}
