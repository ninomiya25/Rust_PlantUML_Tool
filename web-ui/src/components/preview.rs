// Preview component for displaying diagram

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PreviewProps {
    pub image_data: Option<String>,
    pub loading: bool,
}

#[function_component(Preview)]
pub fn preview(props: &PreviewProps) -> Html {
    html! {
        <div class="diagram-display">
            {
                if props.loading {
                    html! { <div class="loading">{"変換中..."}</div> }
                } else if let Some(data) = &props.image_data {
                    html! { <img class="diagram-image" src={data.clone()} alt="PlantUML Diagram" /> }
                } else {
                    html! { <div class="placeholder">{"ここに生成された図が表示されます"}</div> }
                }
            }
        </div>
    }
}
