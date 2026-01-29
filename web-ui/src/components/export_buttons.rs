// Export buttons component for downloading diagrams

use yew::prelude::*;
use plantuml_editor_core::ImageFormat;

#[derive(Properties, PartialEq)]
pub struct ExportButtonsProps {
    pub on_export: Callback<ImageFormat>,
}

#[function_component(ExportButtons)]
pub fn export_buttons(props: &ExportButtonsProps) -> Html {
    let dropdown_open = use_state(|| false);
    
    let toggle_dropdown = {
        let dropdown_open = dropdown_open.clone();
        Callback::from(move |_| {
            dropdown_open.set(!*dropdown_open);
        })
    };
    
    let on_export_png = {
        let on_export = props.on_export.clone();
        let dropdown_open = dropdown_open.clone();
        Callback::from(move |_| {
            on_export.emit(ImageFormat::Png);
            dropdown_open.set(false);
        })
    };
    
    let on_export_svg = {
        let on_export = props.on_export.clone();
        let dropdown_open = dropdown_open.clone();
        Callback::from(move |_| {
            on_export.emit(ImageFormat::Svg);
            dropdown_open.set(false);
        })
    };
    
    html! {
        <div class={classes!("export-dropdown", dropdown_open.then(|| "open"))}>
            <button class="export-btn" onclick={toggle_dropdown}>
                {"エクスポート"}
                <span>{"▼"}</span>
            </button>
            <div class="export-options">
                <button class="export-option" onclick={on_export_png}>
                    {"PNG形式で保存"}
                </button>
                <button class="export-option" onclick={on_export_svg}>
                    {"SVG形式で保存"}
                </button>
            </div>
        </div>
    }
}
