use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use pulldown_cmark::{html, Parser};

#[derive(Debug, Clone, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/article/*id")]
    Article{id: String},
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub enum FetchState<T> {
    NotFetching,
    Fetching,
    Success(T),
    Failed(String),
}

enum Msg {
    SetMarkdownFetchState(FetchState<String>),
    GetMarkdown,
    GetError,
}

#[function_component(Home)]
fn home() -> Html {
    let navigator = use_navigator().unwrap();

    let onclick_callback = Callback::from(move |_| navigator.push(&Route::Home));
    html! {
        <div>
            <h1>{ "Secure" }</h1>
            <button onclick={onclick_callback}>{ "Go Home" }</button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

struct Article {
    markdown: FetchState<String>,
}

impl Component for Article {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            markdown: FetchState::NotFetching,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.markdown = fetch_state;
                true
            }
            Msg::GetMarkdown => {
                let id = ctx.props().id.clone();
                ctx.link().send_future(async move {
                    //let a = id.clone();
                    //let a= "ssss".to_string();
                    match fetch_markdown(&id).await {
                        Ok(md) => Msg::SetMarkdownFetchState(FetchState::Success(md)),
                        Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed(err)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
            Msg::GetError => {
                let id = ctx.props().id.clone();
                ctx.link().send_future(async move{
                    match fetch_markdown(&id).await {
                        Ok(md) => Msg::SetMarkdownFetchState(FetchState::Success(md)),
                        Err(err) => Msg::SetMarkdownFetchState(FetchState::Failed(err)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetMarkdownFetchState(FetchState::Fetching));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.markdown {
            FetchState::NotFetching => {
                ctx.link().callback(|_:Msg| {Msg::GetMarkdown});
                html! {"Getting markdown..."}
            },
            FetchState::Fetching => html! { "Getting markdown..." },
            FetchState::Success(data) => html! { render_markdown(data) },
            FetchState::Failed(err) => html! { "404" },
        }
    }
}

fn render_markdown(data: &str) -> Html {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    let parser = Parser::new_ext(data, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html!(html_output)
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! {
            <Home />
        },
        Route::Article{id} => html! {
            <Article id={id.clone()}/>
        },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

async fn fetch_markdown(url: &str) -> Result<String, String> {
    let body = reqwest::get(url).await.map_err(|err| {
        err.to_string()
    })?.text().await.map_err(|err|{err.to_string()})?;

    Ok(body)
}

fn main() {
    yew::Renderer::<App>::new().render();
}