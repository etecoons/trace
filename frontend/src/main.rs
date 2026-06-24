#![allow(clippy::collapsible_if, clippy::unnecessary_map_or)]

mod api;
mod app;
mod app_update;
mod app_view;
mod components;
mod header;
mod i18n;
mod storage;
mod types;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
