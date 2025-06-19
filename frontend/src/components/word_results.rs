use yew::prelude::*;
use crate::types::ScanResult;

#[derive(Properties, PartialEq)]
pub struct WordResultsProps {
    pub word: String,
    pub result: ScanResult,
}

#[function_component(WordResults)]
pub fn word_results(props: &WordResultsProps) -> Html {
    html! {
        <>
            <div class="summary">
                <h3>{ format!("🔍 Search Results for \"{}\"", props.word) }</h3>
                if props.result.is_css_only {
                    <div class="success">
                        <p><strong>{"✅ Found ONLY in CSS/SCSS files!"}</strong></p>
                        <p>{"This word might be safe to remove."}</p>
                    </div>
                } else if !props.result.css_files.is_empty() || !props.result.other_files.is_empty() {
                    <div class="warning">
                        <p><strong>{"⚠️ Found in multiple file types"}</strong></p>
                        <p>{"This word is still in use - do not remove!"}</p>
                    </div>
                } else {
                    <div class="error">
                        <p><strong>{"❌ Word not found in any files"}</strong></p>
                    </div>
                }
            </div>
            
            if !props.result.css_files.is_empty() {
                <details class="file-section">
                    <summary><strong>{ format!("📁 CSS/SCSS Files ({})", props.result.css_files.len()) }</strong></summary>
                    <div class="class-list">
                        { for props.result.css_files.iter().map(|file| {
                            html! {
                                <div class="class-item">
                                    <span>{ file }</span>
                                </div>
                            }
                        }) }
                    </div>
                </details>
            }
            
            if !props.result.other_files.is_empty() {
                <details class="file-section">
                    <summary><strong>{ format!("📁 Other Files ({})", props.result.other_files.len()) }</strong></summary>
                    <div class="class-list">
                        { for props.result.other_files.iter().map(|file| {
                            html! {
                                <div class="class-item">
                                    <span>{ file }</span>
                                </div>
                            }
                        }) }
                    </div>
                </details>
            }
        </>
    }
}