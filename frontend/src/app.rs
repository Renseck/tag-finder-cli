use yew::prelude::*;
use yew_hooks::prelude::*;

use crate::components::*;
use crate::services::TauriService;
use crate::types::{UnusedReport, ScanResult};

#[derive(Clone, PartialEq)]
pub enum AppState {
    Idle,
    Loading(String),
    ShowingAnalysis(UnusedReport),
    ShowingWordSearch(String, ScanResult),
    Error(String),
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| AppState::Idle);
    let selected_directory = use_state(String::new);
    let search_word = use_state(String::new);

    let is_loading = matches!(*state, AppState::Loading(_));

    // Browse directory callback
    let on_browse = {
        let state = state.clone();
        let selected_directory = selected_directory.clone();
        
        Callback::from(move |_| {
            let state = state.clone();
            let selected_directory = selected_directory.clone();
            
            let callback = Callback::from(move |directory: Option<String>| {
                if let Some(dir) = directory {
                    selected_directory.set(dir);
                    state.set(AppState::Idle);
                } else {
                    state.set(AppState::Error("Failed to select directory".to_string()));
                }
            });
            
            TauriService::select_directory(callback);
        })
    };

    // Analyze CSS callback
    let on_analyze = {
        let state = state.clone();
        let selected_directory = selected_directory.clone();
        
        Callback::from(move |_| {
            let directory = (*selected_directory).clone();
            if directory.is_empty() {
                return;
            }
            
            state.set(AppState::Loading("Analyzing CSS classes...".to_string()));
            
            let state = state.clone();
            let callback = Callback::from(move |result: Result<UnusedReport, String>| {
                match result {
                    Ok(report) => state.set(AppState::ShowingAnalysis(report)),
                    Err(err) => state.set(AppState::Error(err)),
                }
            });
            
            TauriService::analyze_css(directory, callback);
        })
    };

    // Search word change callback
    let on_search_word_change = {
        let search_word = search_word.clone();
        Callback::from(move |word: String| {
            search_word.set(word);
        })
    };

    // Find word callback
    let on_find_word = {
        let state = state.clone();
        let selected_directory = selected_directory.clone();
        let search_word = search_word.clone();
        
        Callback::from(move |_| {
            let directory = (*selected_directory).clone();
            let word = (*search_word).clone();
            
            if directory.is_empty() || word.is_empty() {
                return;
            }
            
            state.set(AppState::Loading("Searching for word...".to_string()));
            
            let state = state.clone();
            let word_clone = word.clone();
            let callback = Callback::from(move |result: Result<ScanResult, String>| {
                match result {
                    Ok(scan_result) => state.set(AppState::ShowingWordSearch(word_clone.clone(), scan_result)),
                    Err(err) => state.set(AppState::Error(err)),
                }
            });
            
            TauriService::find_word(word, directory, callback);
        })
    };

    html! {
        <div class="container">
            <div class="header">
                <h1>{"🎯 Tag Finder"}</h1>
                <p>{"Find unused CSS classes in your project"}</p>
            </div>
            
            <Controls 
                selected_directory={(*selected_directory).clone()}
                search_word={(*search_word).clone()}
                is_loading={is_loading}
                on_browse={on_browse}
                on_analyze={on_analyze}
                on_search_word_change={on_search_word_change}
                on_find_word={on_find_word} />
            
            <div class="results">
                { match &*state {
                    AppState::Idle => html! {},
                    AppState::Loading(message) => html! {
                        <Loading message={message.clone()} />
                    },
                    AppState::ShowingAnalysis(report) => html! {
                        <AnalysisResults report={report.clone()} />
                    },
                    AppState::ShowingWordSearch(word, result) => html! {
                        <WordResults word={word.clone()} result={result.clone()} />
                    },
                    AppState::Error(error) => html! {
                        <div class="error">
                            <strong>{"Error:"}</strong> { error }
                        </div>
                    },
                } }
            </div>
        </div>
    }
}