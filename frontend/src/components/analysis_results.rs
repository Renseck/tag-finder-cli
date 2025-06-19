use yew::prelude::*;
use crate::types::UnusedReport;
use crate::components::FileSection;

#[derive(Properties, PartialEq)]
pub struct AnalysisResultsProps {
    pub report: UnusedReport,
}

#[function_component(AnalysisResults)]
pub fn analysis_results(props: &AnalysisResultsProps) -> Html {
    let unused_by_file = props.report.unused_by_file();

    html! {
        <>
            <div class="summary">
                <h3>{"📊 Analysis Results"}</h3>
                <p><strong>{"Total classes:"}</strong> { props.report.total_classes }</p>
                <p><strong>{"Unused classes:"}</strong> { props.report.unused_classes.len() }</p>
                <p><strong>{"Used classes:"}</strong> { props.report.used_classes.len() }</p>
                <p><strong>{"Unused percentage:"}</strong> { format!("{:.1}%", props.report.unused_percentage()) }</p>
            </div>
            
            if props.report.unused_classes.is_empty() {
                <div class="success">
                    <h3>{"🎉 Great job!"}</h3>
                    <p>{"No unused CSS classes found!"}</p>
                </div>
            } else {
                { for unused_by_file.iter().map(|(file_name, unused_classes)| {
                    html! {
                        <FileSection 
                            file_name={file_name.clone()} 
                            unused_classes={unused_classes.clone()} />
                    }
                }) }
            }
        </>
    }
}