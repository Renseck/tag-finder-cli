use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ControlsProps {
    pub selected_directory: String,
    pub search_word: String,
    pub is_loading: bool,
    pub on_browse: Callback<()>,
    pub on_analyze: Callback<()>,
    pub on_search_word_change: Callback<String>,
    pub on_find_word: Callback<()>,
}

#[function_component(Controls)]
pub fn controls(props: &ControlsProps) -> Html {
    let search_word_ref = use_node_ref();
    
    let on_search_input = {
        let on_search_word_change = props.on_search_word_change.clone();
        let search_word_ref = search_word_ref.clone();
        
        Callback::from(move |_: InputEvent| {
            if let Some(input) = search_word_ref.cast::<web_sys::HtmlInputElement>() {
                on_search_word_change.emit(input.value());
            }
        })
    };

    let on_search_keypress = {
        let on_find_word = props.on_find_word.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_find_word.emit(());
            }
        })
    };

    html! {
        <div class="controls">
            <div class="input-group">
                <input 
                    type="text" 
                    value={props.selected_directory.clone()} 
                    placeholder="Select directory..." 
                    readonly=true />
                <button 
                    class="btn btn-primary" 
                    onclick={props.on_browse.reform(|_| ())}
                    disabled={props.is_loading}>
                    {"📁 Browse"}
                </button>
                <button 
                    class="btn btn-success" 
                    onclick={props.on_analyze.reform(|_| ())}
                    disabled={props.selected_directory.is_empty() || props.is_loading}>
                    {"🔍 Analyze CSS"}
                </button>
            </div>
            
            <div class="input-group">
                <input 
                    ref={search_word_ref}
                    type="text" 
                    value={props.search_word.clone()}
                    placeholder="Search for specific word..."
                    oninput={on_search_input}
                    onkeypress={on_search_keypress} />
                <button 
                    class="btn btn-primary" 
                    onclick={props.on_find_word.reform(|_| ())}
                    disabled={props.selected_directory.is_empty() || props.search_word.is_empty() || props.is_loading}>
                    {"🔍 Find Word"}
                </button>
            </div>
        </div>
    }
}