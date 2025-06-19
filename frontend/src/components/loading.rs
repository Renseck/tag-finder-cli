use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LoadingProps {
    pub message: String,
}

#[function_component(Loading)]
pub fn loading(props: &LoadingProps) -> Html {
    html! {
        <div class="loading">
            <div class="spinner"></div>
            <span>{ &props.message }</span>
        </div>
    }
}