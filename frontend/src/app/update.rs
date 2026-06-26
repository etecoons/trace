use crate::api::fetch_lookup;
use crate::app::App;
use crate::i18n::{get_saved_language, get_translations, save_language};
use crate::storage::StorageService;
use crate::types::*;
use crate::utils::{get_hash, get_query_param, scroll_to_element};
use gloo_net::http::Request;
use yew::prelude::*;

impl App {
    pub fn create_app(ctx: &Context<Self>) -> Self {
        let language = get_saved_language();
        let theme = StorageService::get_item("theme", "crateria");

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

    pub fn update_app(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let tr = get_translations(self.language);
        match msg {
            Msg::UpdateQuery(q) => {
                self.query = q;
                true
            }
            Msg::PerformLookup => {
                let trimmed = self.query.trim().to_string();
                if trimmed.is_empty() {
                    ctx.link()
                        .send_message(Msg::ShowToast(tr.empty_query_toast.to_string(), true));
                    return false;
                }
                self.loading = true;
                self.error = None;
                self.response = None;
                self.status_text = tr.loading.to_string();
                self.status_type = "info".to_string();

                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match fetch_lookup(&trimmed).await {
                        Ok(data) => link.send_message(Msg::LookupSuccess(Box::new(data))),
                        Err(err) => link.send_message(Msg::LookupFailure(err)),
                    }
                });
                true
            }
            Msg::LookupIP(ip) => {
                self.query = ip;
                ctx.link().send_message(Msg::PerformLookup);
                true
            }
            Msg::LookupSuccess(data) => {
                self.loading = false;
                self.response = Some(*data);
                self.status_text = tr.success_toast.to_string();
                self.status_type = "success".to_string();
                ctx.link()
                    .send_message(Msg::ShowToast(tr.success_toast.to_string(), false));

                if let Some(hash) = get_hash() {
                    gloo_timers::callback::Timeout::new(250, move || {
                        scroll_to_element(&hash);
                    })
                    .forget();
                }
                true
            }
            Msg::LookupFailure(err) => {
                self.loading = false;
                self.error = Some(err.clone());
                self.status_text = format!("{}: {}", tr.failed_toast, err);
                self.status_type = "error".to_string();
                if err == "Invalid or missing PIN" || err == "Unauthorized" {
                    self.is_authenticated = false;
                }
                true
            }
            Msg::LoadConfig(json) => {
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
                self.pin_length =
                    json.get("pinLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
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
                    self.theme = "tourian".to_string();
                    if let Some(window) = web_sys::window() {
                        if let Some(doc) = window.document() {
                            if let Some(html) = doc.document_element() {
                                let _ = html.set_attribute("data-theme", "tourian");
                                html.set_class_name("tourian");
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
            Msg::PinInputChanged(val) => {
                self.pin_input = val;
                self.error_message = None;
                true
            }
            Msg::VerifyPin => {
                let pin = self.pin_input.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let body = serde_json::json!({ "pin": pin });
                    match Request::post("/api/verify-pin")
                        .json(&body)
                        .unwrap()
                        .send()
                        .await
                    {
                        Ok(resp) if resp.status() == 200 => {
                            link.send_message(Msg::VerifyPinSuccess)
                        }
                        Ok(resp) => {
                            let msg = resp
                                .json::<serde_json::Value>()
                                .await
                                .ok()
                                .and_then(|j| {
                                    j.get("error")
                                        .and_then(|v| v.as_str())
                                        .map(|s| s.to_string())
                                })
                                .unwrap_or_else(|| "Invalid PIN".to_string());
                            link.send_message(Msg::VerifyPinFailure(msg));
                        }
                        Err(_) => {
                            link.send_message(Msg::VerifyPinFailure("Connection error".to_string()))
                        }
                    }
                });
                false
            }
            Msg::VerifyPinSuccess => {
                self.is_authenticated = true;
                self.pin_input = String::new();
                self.error_message = None;
                true
            }
            Msg::VerifyPinFailure(err) => {
                self.is_authenticated = false;
                if !err.is_empty() {
                    self.error_message = Some(err);
                }
                true
            }
            Msg::Logout => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("/api/logout").send().await;
                    link.send_message(Msg::LogoutSuccess);
                });
                false
            }
            Msg::LogoutSuccess => {
                self.is_authenticated = false;
                self.pin_input = String::new();
                self.error_message = None;
                self.response = None;
                self.query = String::new();
                true
            }
            Msg::ToggleTheme => {
                self.theme = match self.theme.as_str() {
                    "crateria" => "brinstar".to_string(),
                    "brinstar" => "norfair".to_string(),
                    "norfair" => "wrecked_ship".to_string(),
                    "wrecked_ship" => "maridia".to_string(),
                    "maridia" => "tourian".to_string(),
                    _ => "crateria".to_string(),
                };
                StorageService::set_item("theme", &self.theme);
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Some(html) = doc.document_element() {
                            let _ = html.set_attribute("data-theme", &self.theme);
                            html.set_class_name(&self.theme);
                        }
                    }
                }
                true
            }
            Msg::SwitchLanguage(lang) => {
                save_language(lang);
                self.language = lang;
                self.status_text = "Ready".to_string();
                self.status_type = "info".to_string();
                true
            }
            Msg::ShowToast(message, is_error) => {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                self.toasts.push(Toast {
                    id,
                    message,
                    is_error,
                });
                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(2000, move || {
                    link.send_message(Msg::DismissToast(id));
                })
                .forget();
                true
            }
            Msg::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
                true
            }
            Msg::PrintPage => {
                if let Some(window) = web_sys::window() {
                    let title = self.query.trim();
                    let original_title = window.document().map(|d| d.title()).unwrap_or_default();
                    if let Some(doc) = window.document() {
                        doc.set_title(&format!("{} - {}", self.site_title, title));
                    }
                    let _ = window.print();
                    if let Some(doc) = window.document() {
                        doc.set_title(&original_title);
                    }
                }
                false
            }
            Msg::Nothing => false,
        }
    }

    pub fn rendered_app(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(q) = get_query_param("lookup") {
                ctx.link().send_message(Msg::UpdateQuery(q));
                ctx.link().send_message(Msg::PerformLookup);
            }
        }
    }
}
