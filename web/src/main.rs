use web::api::SearchResponse;
use web::results::ResultsComponent;
use web::search::SearchComponent;
use yew::prelude::*;

struct App {
    query: String,
    search_results: Option<SearchResponse>,
}

enum AppMessage {
    SearchResponse(String, Option<SearchResponse>),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            query: "".to_string(),
            search_results: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::SearchResponse(query, response) => {
                self.query = query;
                self.search_results = response;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onresult = ctx
            .link()
            .callback(|(q, r)| AppMessage::SearchResponse(q, r));
        html! {
            <>
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <h2 class="navbar-item">{ "Photographic Memory Search" }</h2>
                    <div class="navbar-item">
                        <SearchComponent {onresult} />
                    </div>
                </div>

            </nav>
            <div class="container">
                <ResultsComponent results={self.search_results.clone()} query={self.query.clone()} />
            </div>
            <footer>
            </footer>
            </>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
