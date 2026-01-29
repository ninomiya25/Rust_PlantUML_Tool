// Slot list component for loading saved documents

use plantuml_editor_storageservice::{LocalStorageBackend, StorageService};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SlotListProps {
    pub on_load: Callback<usize>,
    pub on_delete: Callback<usize>,
}

#[function_component(SlotList)]
pub fn slot_list(props: &SlotListProps) -> Html {
    let service = StorageService::new(LocalStorageBackend::new());
    let slots = use_state(|| service.list_slots());

    let refresh_slots = {
        let slots = slots.clone();
        Callback::from(move |_| {
            let service = StorageService::new(LocalStorageBackend::new());
            slots.set(service.list_slots());
        })
    };

    let render_slot = |slot_num: usize| {
        let service = StorageService::new(LocalStorageBackend::new());
        let slot_data = service.load_from_slot(slot_num).ok().flatten();

        let on_load = props.on_load.clone();
        let on_delete = props.on_delete.clone();
        let refresh = refresh_slots.clone();

        let on_load_click = {
            Callback::from(move |_| {
                on_load.emit(slot_num);
            })
        };

        let on_delete_click = {
            Callback::from(move |_| {
                on_delete.emit(slot_num);
                refresh.emit(());
            })
        };

        if let Some(text) = slot_data {
            let preview = text.lines().next().unwrap_or("").to_string();
            let title = if preview.starts_with("@startuml") {
                text.lines().nth(1).unwrap_or("ダイアグラム")
            } else {
                preview.as_str()
            };

            html! {
                <div class="save-slot" key={slot_num}>
                    <span class="slot-text">{format!("スロット{}: {}", slot_num, title)}</span>
                    <div class="slot-actions">
                        <button class="slot-button reload-button" onclick={on_load_click} title="再読み込み">
                            {"↻"}
                        </button>
                        <button class="slot-button delete-button" onclick={on_delete_click} title="削除">
                            {"×"}
                        </button>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="save-slot empty" key={slot_num}>
                    <span class="slot-text">{format!("スロット{}: (空)", slot_num)}</span>
                </div>
            }
        }
    };

    html! {
        <div class="slot-list">
            { for (1..=10).map(render_slot) }
        </div>
    }
}
