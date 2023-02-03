use pulldown_cmark::{html, Options, Parser,Event,Tag};

use askama::Template;
use axum::{
    extract::State,
    routing::get,
    http::StatusCode,
    response::Html,
    Router, extract::Path,
};

use std::{net::SocketAddr, sync::Arc, path::PathBuf};

struct AppState {
    root_dir: String,
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
        .route("/article/*path", get(handle_article))
        .with_state(shared_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[derive(Template)]
#[template(path = "index.html",escape = "none")]
pub struct IndexTemplate {
    pub title: String,
    pub markdown: String,
}

async fn handle_article(Path(path): Path<String>,State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    println!("state:{}, path:{}",state.root_dir,path);
    let mut file_path = PathBuf::from(&state.root_dir);
    let path_item = path.split("/");
    for item in path_item {
        file_path.push(item);
    }
    file_path.set_extension("md");
    println!("path:{}",file_path.to_str().expect("not valid"));
    if let Result::Ok(s) = std::fs::read_to_string(file_path.as_path()) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(s.as_str(), options).map(|event|{
            let e = match &event {
                Event::Start(tag) => {
                    let t = match tag {
                        Tag::Heading(heading_level, fragment_identifier, class_list) => {
                            println!("Heading heading_level: {} fragment identifier: {:?} classes: {:?}", heading_level, fragment_identifier, class_list);
                            let mut c = class_list.clone();
                            match heading_level {
                                pulldown_cmark::HeadingLevel::H1 => c.append(&mut vec!["title", "is-1"]),
                                pulldown_cmark::HeadingLevel::H2 => c.append(&mut vec!["title", "is-2"]),
                                pulldown_cmark::HeadingLevel::H3 => c.append(&mut vec!["title", "is-3"]),
                                pulldown_cmark::HeadingLevel::H4 => c.append(&mut vec!["title", "is-4"]),
                                pulldown_cmark::HeadingLevel::H5 => c.append(&mut vec!["title", "is-5"]),
                                pulldown_cmark::HeadingLevel::H6 => c.append(&mut vec!["title", "is-6"]),
                            }
                            Tag::Heading(heading_level.clone(), fragment_identifier.clone(), c)
                        },
                        Tag::Table(column_text_alignment_list,class_list) => {
                            println!("Table column_text_alignment_list: {:?} class_list: {:?}", column_text_alignment_list, class_list);
                            let mut c = class_list.clone();
                            c.push("table");
                            Tag::Table(column_text_alignment_list.clone(),c)
                        },
                        t => t.clone(),
                    };
                    Event::Start(t)
                },
                e => e.clone(),
            };
            e
        });
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        //return Ok(Html(html_output));
        let tpl = IndexTemplate { title:"test markdown".to_string(),markdown:html_output };
        let html = tpl.render().map_err(|_| StatusCode::BAD_REQUEST)?;
        return Ok(Html(html));
    }
    return Err(StatusCode::BAD_REQUEST);
}
