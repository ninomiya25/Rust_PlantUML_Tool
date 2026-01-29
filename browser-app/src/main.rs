// PlantUML Editor Browser Application

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use plantuml_editor_web_ui::App;

#[function_component(Main)]
fn main_component() -> Html {
    html! {
        <App />
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<Main>::new().render();
}

fn main() {
    run_app();
}
