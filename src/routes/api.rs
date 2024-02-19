use std::{
    collections::{HashSet, VecDeque},
    ffi::OsStr,
    path::Path,
    str::FromStr,
};

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Form, Json,
};
use chrono::{serde::ts_seconds_option, DateTime, Utc};
use handlebars::to_json;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::{
    error::Error,
    // state::{copy_db, get_state, put_state, AppState},
    util::capitalize,
};
use crate::{state::AppState, util::ModChar};
use serde_json::json;
use std::fmt;
use strum_macros::EnumString;

use super::auth::{decrypt_token, verify_user, User};

pub async fn home_page(
    session: Session,
    State(state): State<AppState<'_>>,
) -> Result<Redirect, Response> {
    match session.get_value("token").await.unwrap() {
        Some(_token) => {
            Ok(Redirect::to("/dashboard"))
        },
        None => Err(Html::from(
            state
                .hb
                .render(
                    "layout",
                    &json!({"title": "rmap.rs | Visualize your codebase", "navigation": "index__nav", "page": "index"}),
                )
                .unwrap(),
        )
        .into_response()),
    }
}

pub async fn search(
    session: Session,
    State(state): State<AppState<'_>>,
    Form(query): Form<RepoQuery>,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    // let _username = session.get_value("rmap_username").await.unwrap().unwrap();

    // let state_result = get_state(&state.client_db, &username.value()).await;
    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }
    let some = session.get_value("token").await.unwrap();
    //TODO verify token

    let response; 
    if let Some(token) = some {
        response = search_repos_w_token(&query.q, token.as_str().unwrap(), &state.key).await;
    } else {
        response = search_repos(&query.q).await;
    }

    match response {
        Ok(response) => {
            println!("res  {:?}", response);

            let headers = response.headers().clone();
            let repos = response.json::<RepoItems>().await.unwrap();
            state_db.search_repos = repos.items.clone();
            state_db.search_current = 1;

            if headers.contains_key("link") {
                state_db.search_link = headers.get("link").unwrap().to_str().unwrap().to_string();
            }

            // retrieve first ten of repos
            let first_ten;
            if repos.items.len() < 10 {
                first_ten = repos.items.clone();
            } else {
                first_ten = repos.items[0..10].to_vec()
            }

            // let _ = put_state(&state.client_db, &state_db).await;
            Ok(Html::from(
                        state
                            .hb
                            .render(
                                "search__results",
                                &json!({"repos": to_json(first_ten), "search": format!("\"{}\"", query.q), "count": repos.total_count}),
                            )
                            .unwrap(),
                    )
                    .into_response())
        }
        Err(_) => Err(Error::InternalServerError),
    }
}

pub async fn get_repo(
    session: Session,
    State(state): State<AppState<'_>>,
    Query(repo_item): Query<RepoItem>,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;
    let _user = session.get_value("rmap_username").await.unwrap().unwrap();
    let token = session.get_value("token").await.unwrap().unwrap();
    // let state_result = get_state(&state.client_db, &user).await;
    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }

    let unprocessed_repos = state_db
        .nodes_items
        .iter()
        .filter(|s| s.is_none())
        .collect::<Vec<_>>();

    if unprocessed_repos.len() >= 4 {
        return Ok(Json("Unprocessed repo limit reached, try again in a few").into_response());
    }

    let processed_repos = state_db
        .nodes_items
        .iter()
        .filter(|s| s.contains(&repo_item.full_name) && s.is_some())
        .collect::<Vec<_>>();

    if processed_repos.len() >= 1 {
        return Ok(Json("Repo already processed and exist in buffer").into_response());
    }

    let mut repos = state_db.search_repos.clone();
    repos.extend(state_db.dash_repos.clone());

    let some = repos
        .into_iter()
        .find(|r| r.full_name == repo_item.full_name);

    let repo = some.unwrap();
    let response = get_tree(&repo.clone(), token.as_str().unwrap(), &state.key).await;
    let headers = response.as_ref().unwrap().headers();
    let ratelimit = headers.get("x-ratelimit-remaining").unwrap();

    state_db.ratelimit_count = ratelimit.to_str().unwrap().parse().unwrap();

    match response {
        Ok(response) => {
            let content_length = response.content_length().unwrap();
            if content_length >= 1_500_000 {
                return Ok(Json("Too big of a response, try using rtree-cli").into_response());
            }

            let mut tree = response.json::<Tree>().await.unwrap();
            let any = state_db
                .nodes_items
                .iter()
                .any(|n| n.contains(&repo_item.full_name));

            if !any {
                if state_db.nodes_items.len() == 4 {
                    let node = state_db.nodes_items.pop_front().unwrap();
                    state_db.scan_stats = state_db
                        .scan_stats
                        .clone()
                        .into_iter()
                        .filter(|s| s.name != node.name)
                        .collect::<Vec<_>>();
                    state_db.slots.push(node.id);
                }
                let slots = &mut state_db.slots;

                slots.reverse();
                let id = slots.pop().unwrap();
                slots.reverse();

                // println!("before nodes {:?}", state_db.nodes_items.len());
                state_db.nodes_items.push_back(Node {
                    name: repo_item.full_name.clone(),
                    links: None,
                    last_modified: Some(Utc::now()),
                    id,
                });

                let tree_len = tree.tree.len();
                tree.tree = tree
                    .tree
                    .into_iter()
                    .filter(|x| {
                        let ext = Path::new(&x.path)
                            .extension()
                            .and_then(OsStr::to_str)
                            .unwrap_or("");

                        x.r#type.contains("blob")
                            && BlobKind::from_str(ext).unwrap_or(BlobKind::Unknown)
                                != BlobKind::Unknown
                    })
                    .collect::<Vec<_>>();

                state_db.scan_stats.push(ScanStat {
                    name: repo_item.full_name,
                    task: "skipped (files)".to_string(),
                    value: tree_len - tree.tree.len(),
                });

                // println!("attempting to save nodes  in dynamo {:?}", state_db);
                // let _ = put_state(&state.client_db, &state_db).await;
                return Ok(Json(RepoTree { tree, id }).into_response());
            }

            Err(Error::BadRequest)
        }
        Err(_) => Err(Error::InternalServerError),
    }
}

pub async fn get_scan(
    session: Session,
    State(state): State<AppState<'_>>,
    Form(scan): Form<ScanItem>,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    let username = session.get_value("rmap_username").await.unwrap().unwrap();
    // let state_result = get_state(&state.client_db, &username.value()).await;
    // if let Some(db_state) = state_result {
    //     if let Some(db) = db_state.clone() {
    //         println!("attempting to get nodes from dynamo {:?}", db);
    //     }
    //     copy_db(&db_state, &mut state_db);
    // }

    let mut nodes = state_db
        .nodes_items
        .clone()
        .into_iter()
        .filter(|nodes| nodes.id == scan.id)
        .collect::<Vec<_>>();

    // println!(" nodes_items {:?}", state_db.nodes_items.len());
    if nodes.len() > 1 || nodes.len() == 0 {
        println!("deleting unwanted nodes");

        // state_db.nodes_items.clear();
        // state_db.scan_stats.clear();
        // state_db.slots = vec![1,2,3,4];
        // let _ = put_state(&state.client_db, &state_db).await;
        return Err(Error::BadRequest);
    }

    let node = nodes.pop().unwrap();

    let scan;
    if node.is_some() {
        scan = Scan {
            name: node.name.clone(),
            class_name: node.name.split(char::is_punct).collect(),
            is_done: true,
            is_error: false,
            id: node.id,
        };
    } else {
        scan = Scan {
            name: node.name.clone(),
            class_name: node.name.split(char::is_punct).collect(),
            is_done: false,
            is_error: false,
            id: node.id,
        };
    }

    // println!("\nscan res {:?}", scan);
    // println!("attempting to save nodes  in dynamo {:?}", state_db);
    // let _ = put_state(&state.client_db, &state_db).await;
    let scan_temp = state
        .hb
        .render("scans", &json!({"scan": to_json(scan)}))
        .unwrap();

    Ok(Html::from(scan_temp).into_response())
}

pub async fn get_blob(
    session: Session,
    State(state): State<AppState<'_>>,
    Query(blob): Query<BlobUrl>,
) -> Result<Response, Error> {
    // let state_db = state.db.lock().await;
    let token = session.get_value("token").await.unwrap().unwrap();

    let response = fetch_blob(&blob.url, token.as_str().unwrap(), &state.key).await;

    // let headers = response.as_ref().unwrap().headers();
    // let ratelimit = headers.get("x-ratelimit-remaining").unwrap();
    // state_db.ratelimit_count = ratelimit.to_str().unwrap().parse().unwrap();

    let content_length = response.as_ref().unwrap().content_length().unwrap();
    if content_length >= 900_000 {
        print!(".");
        return Ok("".into_response());
    }

    let blob = response.unwrap().json::<Content>().await.unwrap();

    if blob.content.is_some() {
        return Ok(blob.content.unwrap().into_response());
    }

    return Ok("".into_response());
}

pub async fn get_dashboard(
    session: Session,
    State(state): State<AppState<'_>>,
) -> Result<Response, Error> {
    println!("here entering dash session {:?}\n\n", session);
    let result = session.save().await;
    if let Err(err) = result {
        println!("fellow short error {:?}", err);
    }
    let _ = session.load().await;
    let _ = session.load().await;
    let _ = session.load().await;
    let _ = session.load().await;
    let _ = session.load().await;
    let _ = session.load().await;
    let _ = session.load().await;

    println!("here entering dash session after lock {:?}\n\n", session);
    let some = session.get_value("token").await.unwrap();
    let key = session.get::<String>("token").await.unwrap();

    println!("here entering dash some {:?} \n\n key {:?}", some, key);
    match some {
        Some(token) => {
            let mut state_db = state.db.lock().await;
            print!("here token {:?}\n", token.as_str().unwrap());
            let mut ok =
                verify_user(&decrypt_token(&token.as_str().unwrap(), state.key).await).await;
            let response = ok.unwrap();

            if response.status().is_client_error() {
                let _flushed = session.flush().await;
                println!("\n\nflushing and signing out");

                return Ok(Html::from(
                    state
                        .hb
                        .render(
                            "layout",
                            &json!({"title": "rmap.rs | Visualize your codebase", "navigation": "index__nav", "page": "index"}),
                        )
                        .unwrap(),
                )
                .into_response());
            }

            let headers = response.headers().clone();
            let ratelimit = headers.get("x-ratelimit-remaining").unwrap();

            let user = response.json::<User>().await.unwrap();
            let _ = session.insert("rmap_username", user.login.clone()).await;

            // let some = get_state(&state.client_db, &user.login.clone()).await;
            // if let Some(db_state) = some {
            //     copy_db(&db_state, &mut state_db);
            // }

            state_db.username = user.login.clone();
            state_db.ratelimit_count = ratelimit.to_str().unwrap().parse().unwrap();
            if state_db.has_repos && state_db.dash_repos.len() != 4 {
                ok = get_user_repos(&user.login, &token.as_str().unwrap(), &state.key).await;

                match ok {
                    Ok(res) => {
                        let mut repos = res.json::<Vec<Repo>>().await.unwrap();
                        if repos.len() >= 4 {
                            repos = repos[0..4].to_vec();
                        }
                        state_db.dash_repos = repos;
                    }
                    Err(_) => {
                        state_db.has_repos = false;
                    }
                }
            }

            let mut name = user.login;
            if user.name.is_some() {
                name = capitalize(user.name.unwrap());
            }

            let repos = &state_db.dash_repos;
            let mut scans = state_db
                .nodes_items
                .iter()
                .map(|s| {
                    let class_name: String = s.name.split(char::is_punct).collect();

                    if s.is_some() {
                        return Some(Scan {
                            name: s.name.clone(),
                            class_name,
                            is_done: true,
                            is_error: false,
                            id: s.id,
                        });
                    }

                    Some(Scan {
                        name: s.name.clone(),
                        class_name,
                        is_done: false,
                        is_error: true,
                        id: s.id,
                    })
                })
                .collect::<Vec<_>>();

            scans.extend(state_db.slots.iter().map(|_x| None));

            // let scans = &state_db.slots;

            let topics = repos
                .iter()
                .map(|r| r.topics.to_vec())
                .flatten()
                .collect::<HashSet<_>>();

            let mut interests = repos
                .iter()
                .map(|r| r.language.clone().unwrap_or_default())
                .collect::<HashSet<_>>();

            interests.extend(topics.into_iter());
            interests = interests
                .iter()
                .filter(|i| i.as_str() != "")
                .map(|i| i.to_string())
                .collect::<HashSet<_>>();

            // let _ = put_state(&state.client_db, &state_db).await;
            Ok(Html::from(
                    state
                        .hb
                        .render(
                            "layout",
                            &json!({"title": "rmap.rs | Dashboard", "navigation": "navigation", "page": "dashboard", "repos": to_json(repos), "coins": state_db.ratelimit_count, "name": name, "interests": to_json(interests),  "scans": to_json(scans)}),
                        )
                        .unwrap(),
                )
                .into_response())
        }
        None => Err(Error::Unauthorized),
    }
}

pub async fn post_dashboard(
    session: Session,
    State(state): State<AppState<'_>>,
    Form(dashboard): Form<DashBoardQuery>,
) -> Result<Response, Error> {
    let some = session.get_value("token").await.unwrap();

    if let Some(_token) = some {
        let user = session.get_value("rmap_username").await.unwrap().unwrap();
        match dashboard.action.unwrap() {
            1 => {
                get_links(
                    user.as_str().unwrap(),
                    axum::extract::State(state.clone()),
                    &dashboard.repo_name.unwrap(),
                )
                .await
            }
            2 => {
                get_repo(
                    session,
                    axum::extract::State(state.clone()),
                    Query(RepoItem {
                        full_name: dashboard.repo_name.unwrap(),
                    }),
                )
                .await
            }
            3 => {
                put_links(
                    user.as_str().unwrap(),
                    axum::extract::State(state.clone()),
                    &dashboard.repo_name.unwrap(),
                    &dashboard.links.unwrap(),
                )
                .await
            }
            4 => {
                let stats = ScanStat {
                    task: dashboard.task.unwrap(),
                    name: dashboard.repo_name.unwrap(),
                    value: dashboard.value.unwrap(),
                };

                put_stat(user.as_str().unwrap(), axum::extract::State(state), stats).await
            }
            5 => get_stats(user.as_str().unwrap(), axum::extract::State(state)).await,

            _ => Err(Error::BadRequest),
        }
    } else {
        Err(Error::Unauthorized)
    }
}

pub async fn pagination(
    session: Session,
    State(state): State<AppState<'_>>,
    Form(page_query): Form<PageQuery>,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    match session.get_value("token").await.unwrap() {
        Some(token) => {
            //TODO verify token

            let username = session.get_value("rmap_username").await.unwrap().unwrap();
            // let state_result = get_state(&state.client_db, username).await;

            // if let Some(db_state) = state_result {
            //     copy_db(&db_state, &mut state_db);
            // }

            let items_per_page = 10;
            let page_num = page_query.page;
            let len = state_db.search_repos.len();
            let items = (items_per_page * page_num) % 30; // dont know if 30 should be len

            let current = state_db.search_current;

            let range = ((current * 3) - 3..(current * 3)).collect::<Vec<_>>();
            let first = range[0] as isize;
            let last = range[2] as isize;
            let contains = range.contains(&page_num);

            if !contains {
                if page_num < first as usize && isize::abs(page_num as isize - first) >= 2 {
                    return Err(Error::BadRequest);
                } else if page_num > last as usize && (page_num - last as usize) >= 2 {
                    return Err(Error::BadRequest);
                }
            }

            let repos;
            if contains && (items + items_per_page <= len) {
                repos = items..items + items_per_page;
            } else {
                if page_num < first.try_into().unwrap() {
                    // println!("fetching previous link");
                    match fetch_next_page(
                        &get_link(&state_db.search_link, "prev"),
                        token.as_str().unwrap(),
                        state.key,
                    )
                    .await
                    {
                        Ok(response) => {
                            let headers = response.headers().clone();
                            if headers.contains_key("link") {
                                state_db.search_link =
                                    headers.get("link").unwrap().to_str().unwrap().to_string();
                            }

                            let repos = response.json::<RepoItems>().await.unwrap();
                            state_db.search_repos = repos.items.clone();
                            state_db.search_current = state_db.search_current - 1;
                        }
                        Err(_) => println!("Error fetching search link"),
                    }
                } else {
                    // println!("fetching next link");
                    match fetch_next_page(
                        &get_link(&state_db.search_link, "next"),
                        token.as_str().unwrap(),
                        state.key,
                    )
                    .await
                    {
                        Ok(response) => {
                            let headers = response.headers().clone();
                            if headers.contains_key("link") {
                                state_db.search_link = response
                                    .headers()
                                    .get("link")
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                            }

                            let repos = response.json::<RepoItems>().await.unwrap();
                            state_db.search_repos = repos.items.clone();
                            state_db.search_current = state_db.search_current + 1;
                        }
                        Err(_) => println!("Error fetching search link"),
                    }
                }

                let new_len = state_db.search_repos.len();
                if items + items_per_page < new_len {
                    repos = items..items + items_per_page;
                } else {
                    repos = items..new_len;
                }
            }

            let page = state_db.search_repos[repos].to_vec();

            // let _ = put_state(&state.client_db, &state_db).await;
            Ok(Json(to_json(page)).into_response())
        }
        None => Err(Error::Unauthorized),
    }
}

pub async fn put_links(
    user: &str,
    State(state): State<AppState<'_>>,
    repo_name: &String,
    repo_links: &String,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;
    let links: Vec<Link> = serde_json::from_str(&repo_links).unwrap();

    // let state_result = get_state(&state.client_db, username).await;

    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }

    state_db.nodes_items = state_db
        .nodes_items
        .clone()
        .into_iter()
        .map(|nodes| {
            if nodes.contains(repo_name) {
                return Node {
                    name: nodes.name.clone(),
                    links: Some(links.clone()),
                    last_modified: Some(Utc::now()),
                    id: nodes.id,
                };
            }
            return Node {
                name: nodes.name.clone(),
                links: nodes.links.clone(),
                last_modified: Some(Utc::now()),
                id: nodes.id,
            };
        })
        .collect::<VecDeque<Node>>();

    // let _ = put_state(&state.client_db, &state_db).await;
    Ok(axum::response::IntoResponse::into_response(Json(
        "Persisted link",
    )))
}

pub async fn get_links(
    user: &str,
    State(state): State<AppState<'_>>,
    repo_name: &String,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    // let state_result = get_state(&state.client_db, user).await;

    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }

    let result = state_db
        .nodes_items
        .clone()
        .into_iter()
        .filter(|nodes| nodes.contains(repo_name))
        .collect::<Vec<Node>>();

    let links = result.get(0).unwrap().links.clone().unwrap();
    Ok(axum::response::IntoResponse::into_response(Json(links)))
}

pub async fn get_stats(user: &str, State(state): State<AppState<'_>>) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    // let state_result = get_state(&state.client_db, username).await;

    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }

    Ok(axum::response::IntoResponse::into_response(Json(
        &state_db.scan_stats,
    )))
}

pub async fn put_stat(
    user: &str,
    State(state): State<AppState<'_>>,
    stat: ScanStat,
) -> Result<Response, Error> {
    let mut state_db = state.db.lock().await;

    // let state_result = get_state(&state.client_db, user).await;

    // if let Some(db_state) = state_result {
    //     copy_db(&db_state, &mut state_db);
    // }

    state_db.scan_stats.push(stat);
    // let _ = put_state(&state.client_db, &state_db).await;

    Ok(axum::response::IntoResponse::into_response(Json(
        &state_db.scan_stats,
    )))
}

pub async fn search_repos_w_token(
    query: &str,
    token: &str,
    key: &[u8; 16],
) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .get(format!(
            "https://api.github.com/search/repositories?q={}",
            query
        ))
        .bearer_auth(decrypt_token(token, *key).await)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub async fn search_repos(
    query: &str
) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .get(format!(
            "https://api.github.com/search/repositories?q={}",
            query
        ))
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub async fn get_tree(
    repo: &Repo,
    token: &str,
    key: &[u8; 16],
) -> Result<reqwest::Response, reqwest::Error> {
    let response: Result<reqwest::Response, reqwest::Error> = Client::new()
        .get(format!(
            "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
            repo.full_name, repo.default_branch
        ))
        .bearer_auth(decrypt_token(token, *key).await)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    return response;
}

pub async fn fetch_blob(
    url: &str,
    token: &str,
    key: &[u8; 16],
) -> Result<reqwest::Response, reqwest::Error> {
    let response: Result<reqwest::Response, reqwest::Error> = Client::new()
        .get(url)
        .bearer_auth(decrypt_token(token, *key).await)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub async fn get_user_repos(
    user: &str,
    token: &str,
    key: &[u8; 16],
) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .get(format!(
            "https://api.github.com/users/{}/repos?type=public&sort=pushed",
            user
        ))
        .bearer_auth(decrypt_token(token, *key).await)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub async fn fetch_next_page(
    link: &str,
    token: &str,
    key: [u8; 16],
) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .get(link)
        .bearer_auth(decrypt_token(token, key).await)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub fn get_link(links: &str, which: &str) -> String {
    let mut link = "";

    let mut splits = links.split(&[',', ';']).collect::<Vec<_>>();
    splits.reverse();

    if splits.contains(&format!(" rel=\"{}\"", which).as_str()) {
        link = splits[splits.iter().position(|&r| r == " rel=\"next\"").unwrap() + 1];
        link = link.trim_start_matches(&['<', ' ']);
        link = link.trim_end_matches(&['>', ' ']);
    }

    link.to_string()
}

#[derive(Deserialize, Debug)]
pub struct RepoQuery {
    pub q: String,
}

#[derive(Deserialize)]
pub struct RepoItem {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct RepoItems {
    pub total_count: usize,
    pub items: Vec<Repo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Repo {
    pub full_name: String,
    pub blobs_url: String,
    pub default_branch: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanStat {
    pub task: String,
    pub name: String,
    pub value: usize,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub links: Option<Vec<Link>>,
    #[serde(with = "ts_seconds_option")]
    pub last_modified: Option<DateTime<Utc>>,
    pub id: usize,
}

impl Node {
    pub fn contains(&self, pat: &String) -> bool {
        self.name.contains(pat)
    }

    pub fn is_some(&self) -> bool {
        self.links.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.links.is_none()
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, Ord, PartialOrd, Deserialize)]
pub struct Link {
    pub source: String,
    pub target: String,
    pub full_import: String,
    pub full_path: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub tree: Vec<Blob>,
}

#[derive(Debug, Serialize)]
pub struct RepoTree {
    pub tree: Tree,
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Blob {
    pub path: String,
    pub r#type: String,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Serialize, EnumString, Default, Clone, PartialEq)]
pub enum BlobKind {
    // #[strum(serialize = "java")]
    // Java,
    #[strum(serialize = "js", serialize = "ts")]
    JavaScript,
    // #[strum(serialize = "py")]
    // Python,
    // #[strum(serialize = "rs")]
    // Rust,
    // #[strum(serialize = "cpp")]
    // CPlusPlus,
    // #[strum(serialize = "c")]
    // C,
    // #[strum(serialize = "go")]
    // GoLang,
    #[default]
    Unknown,
}

impl fmt::Display for BlobKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // BlobKind::Java => write!(f, "Java"),
            BlobKind::JavaScript => write!(f, "JavaScript"),
            // BlobKind::Python => write!(f, "Python"),
            // BlobKind::Rust => write!(f, "Rust"),
            // BlobKind::CPlusPlus => write!(f, "CPlusPlus"),
            // BlobKind::C => write!(f, "C"),
            // BlobKind::GoLang => write!(f, "GoLang"),
            BlobKind::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ScanItem {
    pub id: usize,
}

#[derive(Debug, Serialize)]
pub struct Scan {
    pub name: String,
    pub class_name: String,
    pub is_done: bool,
    pub is_error: bool,
    pub id: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Content {
    pub content: Option<String>,
}

#[derive(Deserialize)]
pub struct BlobUrl {
    pub url: String,
}

#[derive(Deserialize)]
pub struct DashBoardQuery {
    pub action: Option<usize>,
    pub query: Option<usize>,
    pub value: Option<usize>,
    pub links: Option<String>,
    pub search: Option<String>,
    pub repo_name: Option<String>,
    pub task: Option<String>,
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: usize,
}
