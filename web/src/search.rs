use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use tonic_web_wasm_client::Client;
use wasm_bindgen::JsValue;
use web_sys::console;
use yew::prelude::*;

use crate::text_input::TextInput;

use crate::api::pms_service_client::PmsServiceClient;
use crate::api::{SearchRequest, SearchResponse};

// Something wrong has occurred while searching
#[derive(Debug, Clone, PartialEq)]
pub struct SearchError {
    err: JsValue,
}
impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for SearchError {}

impl From<JsValue> for SearchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}
impl From<tonic::Status> for SearchError {
    fn from(status: tonic::Status) -> Self {
        Self {
            err: JsValue::from_str(&status.to_string()),
        }
    }
}

async fn search_screens(
    query_client: &mut PmsServiceClient<Client>,
    query: &str,
) -> Result<SearchResponse, SearchError> {
    let start_time = prost_types::Timestamp {
        seconds: 0 as i64,
        nanos: 0,
    };
    let end_time = prost_types::Timestamp {
        seconds: 0 as i64,
        nanos: 0,
    };
    let response = query_client
        .search_screens(SearchRequest {
            query: query.to_string(),
            start_time: Some(start_time),
            end_time: Some(end_time),
        })
        .await?;
    return Ok(response.into_inner());
}

pub enum SearchMsg {
    Search(),
    SetQuery(String),
    Response(Option<SearchResponse>),
}

pub enum SearchState {
    NotSearching,
    Searching(),
}

pub struct SearchComponent {
    search_state: SearchState,
    query: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct SearchProps {
    pub onresult: Callback<(String, Option<SearchResponse>)>,
}

impl Component for SearchComponent {
    type Message = SearchMsg;
    type Properties = SearchProps;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            search_state: SearchState::NotSearching,
            query: "".to_string(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SearchMsg::Search() => {
                let mut query_client = PmsServiceClient::new(tonic_web_wasm_client::Client::new(
                    "http://localhost:50001".to_string(),
                ));
                let query_cp = self.query.clone();
                ctx.link().send_future(async move {
                    console::log_1(&"Sending request".into());
                    let response = search_screens(&mut query_client, &query_cp).await;
                    console::log_1(&"Got response".into());
                    match response {
                        Ok(_) => return SearchMsg::Response(response.ok()),
                        Err(_) => return SearchMsg::Response(None),
                    }
                });
                self.search_state = SearchState::Searching();
                true
            }
            SearchMsg::Response(response) => {
                self.search_state = SearchState::NotSearching;
                ctx.props().onresult.emit((self.query.clone(), response));
                true
            }
            SearchMsg::SetQuery(query) => {
                self.query = query;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().callback(SearchMsg::SetQuery);

        html! {
                <form class="navbar-item" action="javascript:void(0);">
                <div class="navbar-item">
                    <TextInput {onchange} placeholder="Search query" value={self.query.clone()} />
                </div>
                <div class="navbar-item">
                {
                    match &self.search_state {
                        SearchState::NotSearching => html! {
                            <button class="button" type="submit" onclick={ctx.link().callback(|_| SearchMsg::Search())}> {"Search"} </button>
                        },
                        SearchState::Searching() => html! {
                            <button class="button" type="submit" disabled=true> {"Searching..."} </button>
                        },
                    }
                }
                </div>
                </form>
        }
    }
}
