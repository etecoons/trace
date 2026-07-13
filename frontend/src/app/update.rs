use crate::api::fetch_lookup;
use crate::app::App;
use crate::i18n::{get_translations, save_language};
use crate::types::*;
use crate::utils::{get_hash, scroll_to_element};

use gloo_net::http::Request;
use shared_frontend::i18n::strings::{StringKey, lookup};
use shared_frontend::storage::StorageService;
use shared_frontend::theme::Theme;
use yew::prelude::*;

impl App {
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
                    ctx.link().send_message(Msg::ShowToast(
                        lookup(StringKey::StatusValidationError, self.language).to_string(),
                        true,
                    ));
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
            Msg::LoadConfig(json) => self.handle_load_config(ctx, json),
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
                ctx.link().send_message(Msg::ShowToast(
                    lookup(StringKey::StatusPinSuccess, self.language).to_string(),
                    false,
                ));
                true
            }
            Msg::VerifyPinFailure(err) => {
                self.is_authenticated = false;
                if !err.is_empty() {
                    self.error_message = Some(err);
                }
                ctx.link().send_message(Msg::ShowToast(
                    lookup(StringKey::StatusPinFailure, self.language).to_string(),
                    true,
                ));
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
                ctx.link().send_message(Msg::ShowToast(
                    lookup(StringKey::StatusLogout, self.language).to_string(),
                    false,
                ));
                true
            }
            Msg::ToggleTheme => {
                let current = Theme::from_name(&self.theme).unwrap_or_default();
                let next = match current {
                    Theme::Brinstar => Theme::Norfair,
                    Theme::Norfair => Theme::WreckedShip,
                    Theme::WreckedShip => Theme::Maridia,
                    Theme::Maridia => Theme::Tourian,
                    Theme::Tourian => Theme::Crateria,
                    Theme::Crateria => Theme::Brinstar,
                };
                self.theme = next.name().to_string();
                StorageService::new().set_item("theme", &self.theme);
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Some(html) = doc.document_element() {
                            let _ = html.set_attribute("data-theme", &self.theme);
                            html.set_class_name(&self.theme);
                        }
                    }
                }
                ctx.link().send_message(Msg::ShowToast(
                    lookup(StringKey::StatusThemeChanged, self.language).to_string(),
                    false,
                ));
                true
            }
            Msg::SwitchLanguage(lang) => {
                save_language(lang);
                self.language = lang;
                self.status_text = lookup(StringKey::StatusReady, self.language).to_string();
                self.status_type = "success".to_string();
                true
            }
            Msg::ShowToast(message, is_error) => {
                self.status_text = message.clone();
                self.status_type = if is_error { "error" } else { "success" }.to_string();
                let default_ready = lookup(StringKey::StatusReady, self.language).to_string();
                if message != default_ready {
                    let link = ctx.link().clone();
                    let ready_str = default_ready.clone();
                    gloo_timers::callback::Timeout::new(3000, move || {
                        link.send_message(Msg::ShowToast(ready_str, false));
                    })
                    .forget();
                }
                true
            }
            Msg::DismissToast(_) => true,
            Msg::PrintPage => {
                if let Some(window) = web_sys::window() {
                    let title = self.query.trim();
                    let original_title = window.document().map(|d| d.title()).unwrap_or_default();
                    if let Some(doc) = window.document() {
                        doc.set_title(&format!("{} - {}", self.site_title, title));
                    }
                    let print_res = window.print();
                    if let Some(doc) = window.document() {
                        doc.set_title(&original_title);
                    }
                    if print_res.is_ok() {
                        ctx.link().send_message(Msg::ShowToast(
                            lookup(StringKey::StatusPrintSuccess, self.language).to_string(),
                            false,
                        ));
                    } else {
                        ctx.link().send_message(Msg::ShowToast(
                            lookup(StringKey::StatusPrintFailure, self.language).to_string(),
                            true,
                        ));
                    }
                }
                false
            }
            Msg::OnlineStatusChanged(online) => {
                let (msg_key, is_error) = if online {
                    (StringKey::StatusOnline, false)
                } else {
                    (StringKey::StatusOffline, true)
                };
                ctx.link().send_message(Msg::ShowToast(
                    lookup(msg_key, self.language).to_string(),
                    is_error,
                ));
                true
            }
            Msg::Nothing => false,
        }
    }
}
