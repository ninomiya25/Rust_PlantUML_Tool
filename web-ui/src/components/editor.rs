// Editor component with debounce

use gloo_timers::callback::Timeout;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct EditorProps {
    pub value: String,
    pub on_change: Callback<String>,
}

#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content = use_state(|| props.value.clone());
    let timeout_handle = use_state(|| None::<Timeout>);

    let on_input = {
        let content = content.clone();
        let timeout_handle = timeout_handle.clone();
        let on_change = props.on_change.clone();

        Callback::from(move |e: InputEvent| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let value = input.value();
            content.set(value.clone());

            // Cancel previous timeout by dropping the old handle
            timeout_handle.set(None);

            // Set new timeout for debounce (500ms)
            let on_change = on_change.clone();
            let new_handle = Timeout::new(500, move || {
                on_change.emit(value);
            });
            timeout_handle.set(Some(new_handle));
        })
    };

    html! {
        <textarea
            class="editor-textarea"
            placeholder="PlantUMLソースを入力してください...
例:
@startuml
Alice -> Bob: Hello
Bob --> Alice: Hi!
@enduml"
            oninput={on_input}
            value={(*content).clone()}
        />
    }
}
