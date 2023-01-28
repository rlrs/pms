// Simple component to show the results of a search
use crate::api::SearchResponse;
use unicode_segmentation::UnicodeSegmentation;
use yew::prelude::*;

pub enum ResultsMessage {}

pub struct ResultsComponent {}

#[derive(Clone, PartialEq, Properties)]
pub struct ResultsProps {
    pub query: String,
    pub results: Option<SearchResponse>,
}

impl Component for ResultsComponent {
    type Message = ResultsMessage;
    type Properties = ResultsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &ctx.props().results {
            Some(results) => {
                // Show results in a grid
                html! {
                    <>
                    <h2>{ format!("{} search results", results.screens.len()) }</h2>
                    <div class="cards">
                        { for results.screens.iter().map(|result| {
                            let image_str = "data:image/jpeg;base64,".to_string() + &base64::encode(&result.image);
                            let (before, mid, after) = make_snippet(&result.text, &ctx.props().query, 20);
                            html! {
                                <div class="card">
                                    <div class="card-header">
                                        <a href={image_str.clone()} target="_blank"><img class="image" src={ image_str } /></a>
                                    </div>
                                    <div class="card-content">
                                        <div class="content">
                                            { before } <strong> { mid } </strong>{ after }
                                        </div>
                                    </div>
                                    <div class="card-footer">
                                        <div class="card-footer-item">
                                            <time datetime={ result.time.clone().unwrap().to_string() }>{ result.time.clone().unwrap().to_string() }</time>
                                        </div>
                                    </div>
                                </div>
                            }
                        }) }
                    </div>
                    </>
                }
            }
            None => {
                html! {
                    <div>
                        <h2>{ "No search results" }</h2>
                    </div>
                }
            }
        }
    }
}

fn make_snippet(text: &String, query: &String, length: usize) -> (String, String, String) {
    let mut mid = String::new();
    let mut before = String::new();
    let mut after = String::new();
    let lower_text = text.to_lowercase();
    let words = lower_text.unicode_words().collect::<Vec<&str>>();
    // Find the (potential) index of the query in the text, and extract words before and after
    words
        .iter()
        .position(|&word| word == &query[..])
        .map(|index| {
            let start_idx = std::cmp::max(0, index as isize - (length / 2) as isize) as usize;
            let end_idx = std::cmp::min(index + length / 2, words.len());
            before = words[start_idx..index].join(" ");
            mid = format!(" {} ", words[index].to_string());
            after = words[index..end_idx].join(" ");
        });
    (before, mid, after)
}
