mod app;
mod app_update;
mod app_view;
mod components;
mod header;
mod i18n;
mod storage;
mod types;
mod api;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
