use axum::{
    extract::State,
    routing::get,
    http::StatusCode,
    response::IntoResponse,
    Router, extract::Path,Json
};

use serde::{Deserialize, Serialize};

use std::{net::SocketAddr, sync::Arc};

//use std::collections::HashMap;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};

struct AppState {
    root_dir: String,
}

#[derive(Debug, Serialize, Clone)]
struct Article {
    hash: String,
    author: String,
    content: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
struct ArticleManager {
    count: u32,
    start: u32,
    articles: Vec<Article>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        root_dir:String::from("/home/xml/blog"),
    });
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/articles", get(get_article_list))
        .route("/article/*id", get(handle_article))
        .with_state(shared_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn markdown_list(path:&str) -> Vec<String> {
    let mut list = Vec::new();
    for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir()) {
        
        let f_name = String::from(entry.path().to_str().expect("is not valid path"));
        list.push(f_name);
    }
    return list;
}

fn get_markdown_path(path:&str,hash: &str) -> Option<std::path::PathBuf> {
    //let mut md_reviews = Vec::new();
    for item in markdown_list(path) {
        let hash_string= Sha256::digest(item.clone())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>();

        println!("hash_string: {}, hash: {}, path: {}", hash_string, hash, item);
        if hash_string == hash.to_string() {
            let p = std::path::PathBuf::from(item);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    
    return None;
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_article_list(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, StatusCode> {
    let list = markdown_list(&state.root_dir);
    let mut articles = Vec::<Article>::new();
    for item in list {
        let hash_string= Sha256::digest(item.clone())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>();
        articles.push(Article { hash: hash_string, author: "xml".to_string(), content: None });
    }
    let articles = ArticleManager {
        count: 100,
        start: 30,
        articles,
    };
    Ok(Json(articles))
}

async fn handle_article(Path(id): Path<String>,State(state): State<Arc<AppState>>) -> Result<String, StatusCode> {
    if let Some(path) = get_markdown_path(&state.root_dir,&id) {
        if let Result::Ok(s) = std::fs::read_to_string(path.as_path()) {
            return Ok(s);
        }
    }
    
    return Err(StatusCode::BAD_REQUEST);
}
