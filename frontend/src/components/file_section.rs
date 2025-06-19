use yew::prelude::*;
use crate::types::UnusedClass;
use crate::services::TauriService;

#[derive(Properties, PartialEq)]
pub struct FileSectionProps {
    pub file_name: String,
    pub unused_classes: Vec<UnusedClass>,
}

#[function_component(FileSection)]
pub fn file_section(props: &FileSectionProps) -> Html {
    let open_file = {
        Callback::from(move |(file_path, line): (String, usize)| {
            let callback = Callback::from(|result: Result<(), String>| {
                if let Err(err) = result {
                    log::error!("Failed to open file: {}", err);
                }
            });
            
            TauriService::open_file_at_line(file_path, line, callback);
        })
    };

    html! {
        <details class="file-section">
            <summary>
                { format!("📁 {} ({} unused classes)", props.file_name, props.unused_classes.len()) }
            </summary>
            <div class="class-list">
                { for props.unused_classes.iter().map(|unused_class| {
                    let css_class = &unused_class.class;
                    let file_path = css_class.file.clone();
                    let line = css_class.line;
                    let open_file = open_file.clone();
                    
                    html! {
                        <div class="class-item">
                            <div>
                                <span class="class-name">{ format!(".{}", css_class.name) }</span>
                                <span class="line-number">{ format!("(line {})", css_class.line) }</span>
                            </div>
                            <button 
                                class="btn btn-small" 
                                onclick={open_file.reform(move |_| (file_path.clone(), line))}>
                                {"📝 Open"}
                            </button>
                        </div>
                    }
                }) }
            </div>
        </details>
    }
}