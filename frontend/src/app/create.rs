use crate::app::App;
use crate::i18n::get_saved_language;
use crate::storage::StorageService;
use crate::types::Msg;
use gloo_net::http::Request;
use shared_frontend::theme::{Theme, mapping::Scheme};
use yew::prelude::*;

impl App {
    pub fn create_app(ctx: &Context<Self>) -> Self {
        let language = get_saved_language();
        let raw = StorageService::get_item("theme", Theme::default().name());
        let theme = if let Some(scheme) = Scheme::from_id(&raw) {
            scheme.to_theme().name().to_string()
        } else {
            Theme::from_name(&raw)
                .unwrap_or_default()
                .name()
                .to_string()
        };
        if theme != raw {
            StorageService::set_item("theme", &theme);
        }

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/config").send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    link.send_message(Msg::LoadConfig(json));
                }
            }
        });

        Self {
            query: String::new(),
            site_title: "Trace".to_string(),
            theme,
            language,
            loading: false,
            error: None,
            response: None,
            toasts: Vec::new(),
            next_toast_id: 0,
            status_text: "Ready".to_string(),
            status_type: "info".to_string(),
            is_authenticated: true,
            pin_required: false,
            pin_length: 0,
            pin_input: String::new(),
            error_message: None,
            enable_translation: false,
            enable_themes: true,
            enable_print: false,
            show_version: true,
            show_github: true,
        }
    }
}
