use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: String,
    pub placeholder: String,
    pub onchange: Callback<String>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

/// Controlled Text Input Component
#[function_component(TextInput)]
pub fn text_input(props: &Props) -> Html {
    let Props {
        value,
        placeholder,
        onchange,
    } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        onchange.emit(get_value_from_input_event(input_event));
    });

    html! {
        <input class="input" type="search" {value} {oninput} {placeholder} />
    }
}
