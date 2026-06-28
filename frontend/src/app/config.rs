use crate::app::App;
use crate::types::Msg;
use gloo_net::http::Request;
use shared_frontend::theme::Theme;
use yew::prelude::*;

impl App {
    pub(crate) fn handle_load_config(
        &mut self,
        ctx: &Context<Self>,
        json: serde_json::Value,
    ) -> bool {
        if let Some(title) = json.get("siteTitle").and_then(|v| v.as_str()) {
            self.site_title = title.to_string();
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    document.set_title(&self.site_title);
                }
            }
        }
        self.pin_required = json
            .get("pinRequired")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.pin_length = json.get("pinLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        self.enable_translation = json
            .get("enableTranslation")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("enable_translation").and_then(|v| v.as_bool()))
            .unwrap_or(false);
        self.enable_themes = json
            .get("enableThemes")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("enable_themes").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        self.enable_print = json
            .get("enablePrint")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("enable_print").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        self.show_version = json
            .get("showVersion")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("show_version").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        self.show_github = json
            .get("showGithub")
            .and_then(|v| v.as_bool())
            .or_else(|| json.get("show_github").and_then(|v| v.as_bool()))
            .unwrap_or(true);

        if !self.enable_themes {
            self.theme = Theme::Tourian.name().to_string();
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Some(html) = doc.document_element() {
                        let _ = html.set_attribute("data-theme", Theme::Tourian.name());
                        html.set_class_name(Theme::Tourian.name());
                    }
                }
            }
        }

        if self.pin_required {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("/api/auth-check").send().await {
                    if resp.status() == 200 {
                        link.send_message(Msg::VerifyPinSuccess);
                        return;
                    }
                }
                link.send_message(Msg::VerifyPinFailure(String::new()));
            });
        } else {
            self.is_authenticated = true;
        }
        true
    }
}
