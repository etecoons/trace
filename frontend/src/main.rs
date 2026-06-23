mod header;
mod i18n;
mod storage;
mod types;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use header::Header;
use i18n::{Translations, get_saved_language, get_translations, save_language};
use storage::StorageService;
use types::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WhoisEvent {
    event_action: String,
    event_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WhoisNameserver {
    ldh_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WhoisEntity {
    roles: Vec<String>,
    #[serde(rename = "vcardArray")]
    vcard_array: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct IpAddresses {
    v4: Vec<String>,
    v6: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WhoisData {
    ldh_name: String,
    handle: String,
    status: Vec<String>,
    ip_addresses: IpAddresses,
    events: Vec<WhoisEvent>,
    nameservers: Vec<WhoisNameserver>,
    entities: Vec<WhoisEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IpData {
    ip: String,
    version: String,
    city: Option<String>,
    region: Option<String>,
    region_code: Option<String>,
    country_code: Option<String>,
    country_name: Option<String>,
    postal: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<String>,
    org: Option<String>,
    asn: Option<String>,
    source: String,
    network: Option<String>,
    continent_code: Option<String>,
    languages: Option<String>,
    currency: Option<String>,
    currency_name: Option<String>,
    country_calling_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RirAllocation {
    rir_name: String,
    date_allocated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AsnData {
    asn: u32,
    name: String,
    description_short: String,
    country_code: Option<String>,
    website: String,
    email_contacts: Vec<String>,
    abuse_contacts: Vec<String>,
    owner_address: Vec<String>,
    rir_allocation: RirAllocation,
    traffic_ratio: Option<String>,
    date_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "lowercase")]
enum LookupResponse {
    Whois(WhoisData),
    Ip(IpData),
    Asn(AsnData),
}

#[derive(Clone, Debug, PartialEq)]
struct Toast {
    id: usize,
    message: String,
    is_error: bool,
}

enum Msg {
    UpdateQuery(String),
    PerformLookup,
    LookupIP(String),
    LookupSuccess(Box<LookupResponse>),
    LookupFailure(String),
    LoadConfig(serde_json::Value),
    PinInputChanged(String),
    VerifyPin,
    VerifyPinSuccess,
    VerifyPinFailure(String),
    Logout,
    LogoutSuccess,
    ToggleTheme,
    SwitchLanguage(Language),
    ShowToast(String, bool),
    DismissToast(usize),
    PrintPage,
    Nothing,
}

struct App {
    query: String,
    site_title: String,
    theme: String,
    language: Language,
    loading: bool,
    error: Option<String>,
    response: Option<LookupResponse>,
    toasts: Vec<Toast>,
    next_toast_id: usize,
    status_text: String,
    status_type: String,
    is_authenticated: bool,
    pin_required: bool,
    pin_length: usize,
    pin_input: String,
    error_message: Option<String>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let language = get_saved_language();
        let theme = StorageService::get_item("theme", "dark");

        // Apply theme immediately on startup
        if let Some(window) = web_sys::window()
            && let Some(doc) = window.document()
            && let Some(html) = doc.document_element()
        {
            let _ = html.set_attribute("data-theme", &theme);
            html.set_class_name(&theme);
        }

        // Load site config
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/config").send().await
                && let Ok(json) = resp.json::<serde_json::Value>().await
            {
                link.send_message(Msg::LoadConfig(json));
            }
        });

        Self {
            query: String::new(),
            site_title: "RustWho".to_string(),
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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                        Ok(data) => {
                            link.send_message(Msg::LookupSuccess(Box::new(data)));
                        }
                        Err(err) => {
                            link.send_message(Msg::LookupFailure(err));
                        }
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

                // If query hash is present, scroll to element after rendering
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
                    if let Some(window) = web_sys::window()
                        && let Some(document) = window.document()
                    {
                        document.set_title(&format!("{} - WHOIS, IP, ASN", self.site_title));
                    }
                }
                let pin_req = json
                    .get("pinRequired")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let pin_len = json.get("pinLength").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                self.pin_required = pin_req;
                self.pin_length = pin_len;

                if pin_req {
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(resp) = Request::get("/api/auth-check").send().await {
                            if resp.status() == 200 {
                                link.send_message(Msg::VerifyPinSuccess);
                            } else {
                                link.send_message(Msg::VerifyPinFailure(String::new()));
                            }
                        } else {
                            link.send_message(Msg::VerifyPinFailure(String::new()));
                        }
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
                            link.send_message(Msg::VerifyPinSuccess);
                        }
                        Ok(resp) => {
                            if let Ok(err_json) = resp.json::<serde_json::Value>().await {
                                let msg = err_json
                                    .get("error")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Invalid PIN")
                                    .to_string();
                                link.send_message(Msg::VerifyPinFailure(msg));
                            } else {
                                link.send_message(Msg::VerifyPinFailure("Invalid PIN".to_string()));
                            }
                        }
                        Err(_) => {
                            link.send_message(Msg::VerifyPinFailure(
                                "Connection error".to_string(),
                            ));
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
                    "light" => "dark".to_string(),
                    "dark" => "nord".to_string(),
                    "nord" => "dracula".to_string(),
                    "dracula" => "sepia".to_string(),
                    _ => "light".to_string(),
                };
                StorageService::set_item("theme", &self.theme);
                if let Some(window) = web_sys::window()
                    && let Some(doc) = window.document()
                    && let Some(html) = doc.document_element()
                {
                    let _ = html.set_attribute("data-theme", &self.theme);
                    html.set_class_name(&self.theme);
                }
                true
            }
            Msg::SwitchLanguage(lang) => {
                save_language(lang);
                self.language = lang;

                // Update footer text to localized "Ready"
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

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render && let Some(q) = get_query_param("lookup") {
            ctx.link().send_message(Msg::UpdateQuery(q));
            ctx.link().send_message(Msg::PerformLookup);
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);

        let on_input = ctx.link().callback(|e: InputEvent| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            Msg::UpdateQuery(input.value())
        });

        let on_keydown = ctx.link().callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                Msg::PerformLookup
            } else {
                Msg::Nothing
            }
        });

        let on_submit = ctx.link().callback(|_| Msg::PerformLookup);
        let on_print = ctx.link().callback(|_| Msg::PrintPage);
        let on_toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);
        let on_change_language = ctx.link().callback(Msg::SwitchLanguage);

        html! {
            <>
                /* Header */
                <Header
                    site_title={self.site_title.clone()}
                    theme={self.theme.clone()}
                    language={self.language}
                    toggle_theme={on_toggle_theme}
                    on_language_change={on_change_language}
                    is_authenticated={self.is_authenticated}
                    pin_required={self.pin_required}
                    on_logout={ctx.link().callback(|_| Msg::Logout)}
                    logout_tooltip={tr.logout_tooltip.to_string()}
                    on_print={on_print}
                    print_tooltip={tr.print_tooltip.to_string()}
                    disable_print={self.response.is_none()}
                    theme_toggle_tooltip={tr.toggle_theme.to_string()}
                />

                /* Main Body */
                <div class="container">
                    if !self.is_authenticated {
                        {self.render_pin_entry(ctx)}
                    } else {
                        <div class="app">
                            /* Search controls */
                            <section class="search-controls">
                                <div class="search-input-group">
                                    <input
                                        type="text"
                                        class="search-input"
                                        placeholder={tr.placeholder}
                                        value={self.query.clone()}
                                        oninput={on_input}
                                        onkeydown={on_keydown}
                                    />
                                    <button class="btn btn-primary" onclick={on_submit} title={tr.lookup}>
                                        // Search Icon
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px;">
                                            <circle cx="11" cy="11" r="8"></circle>
                                            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                        </svg>
                                        {tr.lookup}
                                    </button>
                                </div>
                                <p class="examples-text">
                                    {tr.examples}
                                    {": "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("yahoo.com".to_string()))}>{"yahoo.com"}</a>
                                    {", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("8.8.8.8".to_string()))}>{"8.8.8.8"}</a>
                                    {", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("2001:4860:4860::8888".to_string()))}>{"2001:4860:4860::8888"}</a>
                                    {", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("AS13335".to_string()))}>{"AS13335"}</a>
                                </p>
                            </section>

                            /* Loading Spinner */
                            if self.loading {
                                <div class="loading-indicator">
                                    <div class="spinner"></div>
                                    <p>{tr.loading}</p>
                                </div>
                            }

                            /* Results */
                            if let Some(ref data) = self.response {
                                <section>
                                    <h2 class="result-type-header">
                                        {match data {
                                            LookupResponse::Whois(_) => tr.source_whois.to_string(),
                                            LookupResponse::Ip(ip) => tr.source_ip.replace("{}", &ip.source),
                                            LookupResponse::Asn(_) => tr.source_asn.to_string(),
                                        }}
                                    </h2>
                                    <div style="margin-top: 1rem;">
                                        {match data {
                                            LookupResponse::Whois(whois_data) => self.view_whois(whois_data, ctx, &tr),
                                            LookupResponse::Ip(ip_data) => self.view_ip(ip_data, &tr),
                                            LookupResponse::Asn(asn_data) => self.view_asn(asn_data, &tr),
                                        }}
                                    </div>
                                </section>
                            }
                        </div>
                    }
                </div>

                /* Footer */
                <footer class="layout-footer">
                    <div class={classes!("footer-status-text", self.status_type.clone())}>
                        {&self.status_text}
                    </div>
                </footer>

                /* Toast Notifications Container */
                <div class="toast-container">
                    {for self.toasts.iter().map(|t| {
                        let id = t.id;
                        let dismiss = ctx.link().callback(move |_| Msg::DismissToast(id));
                        html! {
                            <div class={classes!("toast", if t.is_error { "error" } else { "success" })} onclick={dismiss}>
                                {&t.message}
                            </div>
                        }
                    })}
                </div>
            </>
        }
    }
}

impl App {
    fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let translations = get_translations(self.language);
        let pin_len = self.pin_length;

        html! {
            <div class="login-container">
                <div class="login-box">
                    <div class="pin-header">
                        <h2 id="pin-description">
                            {translations.enter_pin}
                        </h2>
                    </div>
                    <form id="pin-form" onsubmit={ctx.link().callback(|e: SubmitEvent| { e.prevent_default(); Msg::VerifyPin })}>
                        <div class="pin-wrapper">
                            <input
                                type="password"
                                class="pin-input-field"
                                value={self.pin_input.clone()}
                                oninput={ctx.link().callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::PinInputChanged(input.value())
                                })}
                                placeholder={"• ".repeat(pin_len).trim().to_string()}
                                maxlength={pin_len.to_string()}
                                autofocus=true
                            />
                        </div>
                    </form>
                    <div class="pin-status">
                        if let Some(ref err) = self.error_message {
                            <p id="pin-error" class="pin-error" style="display: block;">
                                {if err == "Invalid PIN" { translations.invalid_pin } else { err.as_str() }}
                            </p>
                        }
                    </div>
                </div>
            </div>
        }
    }

    fn view_whois(&self, data: &WhoisData, ctx: &Context<Self>, tr: &Translations) -> Html {
        let registrar_name = data
            .entities
            .iter()
            .find(|e| e.roles.contains(&"registrar".to_string()))
            .and_then(get_registrar_fn)
            .unwrap_or_else(|| "N/A".to_string());

        let creation = data
            .events
            .iter()
            .find(|e| e.event_action == "registration")
            .map(|e| format_date(&e.event_date))
            .unwrap_or_else(|| "N/A".to_string());
        let expiration = data
            .events
            .iter()
            .find(|e| e.event_action == "expiration")
            .map(|e| format_date(&e.event_date))
            .unwrap_or_else(|| "N/A".to_string());
        let updated = data
            .events
            .iter()
            .find(|e| e.event_action == "lastChanged")
            .map(|e| format_date(&e.event_date))
            .unwrap_or_else(|| "N/A".to_string());

        html! {
            <div class="result-box">
                /* Domain Info */
                <div id="domain-info" class="segment-card segment-blue">
                    <h2>
                        <span>{tr.domain_info}</span>
                        <a href="#domain-info" class="permalink" title="Permalink to this section">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                            </svg>
                        </a>
                    </h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.domain}</span>
                            <span class="data-value">{&data.ldh_name}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.handle}</span>
                            <span class="data-value">{&data.handle}</span>
                        </div>
                    </div>
                </div>

                /* IP Addresses */
                <div id="ip-addresses" class="segment-card segment-red">
                    <h2>
                        <span>{tr.ip_addresses}</span>
                        <a href="#ip-addresses" class="permalink" title="Permalink to this section">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                            </svg>
                        </a>
                    </h2>

                    <div class="data-grid" style="grid-template-columns: 1fr;">
                        <div class="data-row" style="flex-direction: column; gap: 0.5rem; align-items: flex-start;">
                            <span class="data-label">{tr.ipv4}</span>
                            if data.ip_addresses.v4.is_empty() {
                                <span class="data-value">{tr.no_ipv4}</span>
                            } else {
                                <div class="badge-list">
                                    {for data.ip_addresses.v4.iter().map(|ip| {
                                        let ip_clone = ip.clone();
                                        html! {
                                            <button class="ip-button" onclick={ctx.link().callback(move |_| Msg::LookupIP(ip_clone.clone()))}>
                                                {ip}
                                            </button>
                                        }
                                    })}
                                </div>
                            }
                        </div>
                        <div class="data-row" style="flex-direction: column; gap: 0.5rem; align-items: flex-start;">
                            <span class="data-label">{tr.ipv6}</span>
                            if data.ip_addresses.v6.is_empty() {
                                <span class="data-value">{tr.no_ipv6}</span>
                            } else {
                                <div class="badge-list">
                                    {for data.ip_addresses.v6.iter().map(|ip| {
                                        let ip_clone = ip.clone();
                                        html! {
                                            <button class="ip-button" onclick={ctx.link().callback(move |_| Msg::LookupIP(ip_clone.clone()))}>
                                                {ip}
                                            </button>
                                        }
                                    })}
                                </div>
                            }
                        </div>
                    </div>
                </div>

                /* Domain Status */
                if !data.status.is_empty() {
                    <div id="domain-status" class="segment-card segment-green">
                        <h2>
                            <span>{tr.domain_status}</span>
                            <a href="#domain-status" class="permalink" title="Permalink to this section">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                                </svg>
                            </a>
                        </h2>
                        <div class="badge-list">
                            {for data.status.iter().map(|status| html! {
                                <span class="badge">{status}</span>
                            })}
                        </div>
                    </div>
                }

                /* Important Dates */
                <div id="important-dates" class="segment-card segment-purple">
                    <h2>
                        <span>{tr.important_dates}</span>
                        <a href="#important-dates" class="permalink" title="Permalink to this section">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                            </svg>
                        </a>
                    </h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.registration}</span>
                            <span class="data-value">{creation}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.expiration}</span>
                            <span class="data-value">{expiration}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.last_updated}</span>
                            <span class="data-value">{updated}</span>
                        </div>
                    </div>
                </div>

                /* Nameservers */
                if !data.nameservers.is_empty() {
                    <div id="nameservers" class="segment-card segment-yellow">
                        <h2>
                            <span>{tr.nameservers}</span>
                            <a href="#nameservers" class="permalink" title="Permalink to this section">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                                </svg>
                            </a>
                        </h2>
                        <div class="data-grid">
                            {for data.nameservers.iter().map(|ns| html! {
                                <div class="data-row">
                                    <span class="data-value">{&ns.ldh_name}</span>
                                </div>
                            })}
                        </div>
                    </div>
                }

                /* Registrar */
                <div class="segment-card segment-indigo">
                    <h2>{tr.registrar_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.registrar}</span>
                            <span class="data-value">{registrar_name}</span>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn view_ip(&self, data: &IpData, tr: &Translations) -> Html {
        html! {
            <div class="result-box">
                /* IP Info */
                <div class="segment-card segment-blue">
                    <h2>{tr.ip_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.ip_address}</span>
                            <span class="data-value">{&data.ip}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.network}</span>
                            <span class="data-value">{data.network.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.version}</span>
                            <span class="data-value">{&data.version}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.org}</span>
                            <span class="data-value">{data.org.as_deref().unwrap_or("N/A")}</span>
                        </div>
                    </div>
                </div>

                /* Location Info */
                <div class="segment-card segment-green">
                    <h2>{tr.loc_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.city}</span>
                            <span class="data-value">{data.city.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.region}</span>
                            <span class="data-value">
                                {format!("{} ({})",
                                    data.region.as_deref().unwrap_or("N/A"),
                                    data.region_code.as_deref().unwrap_or("N/A")
                                )}
                            </span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.country}</span>
                            <span class="data-value">
                                {format!("{} ({})",
                                    data.country_name.as_deref().unwrap_or("N/A"),
                                    data.country_code.as_deref().unwrap_or("N/A")
                                )}
                            </span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.postal}</span>
                            <span class="data-value">{data.postal.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.continent}</span>
                            <span class="data-value">{data.continent_code.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.timezone}</span>
                            <span class="data-value">{data.timezone.as_deref().unwrap_or("N/A")}</span>
                        </div>
                    </div>
                </div>

                /* Coordinates */
                <div class="segment-card segment-yellow">
                    <h2>{tr.geo_coords}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.lat}</span>
                            <span class="data-value">{data.latitude.map(|l| l.to_string()).unwrap_or_else(|| "N/A".to_string())}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.lon}</span>
                            <span class="data-value">{data.longitude.map(|l| l.to_string()).unwrap_or_else(|| "N/A".to_string())}</span>
                        </div>
                    </div>
                </div>

                /* Additional Info */
                <div class="segment-card segment-purple">
                    <h2>{tr.add_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.asn}</span>
                            <span class="data-value">{data.asn.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.languages}</span>
                            <span class="data-value">{data.languages.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.currency}</span>
                            <span class="data-value">
                                {format!("{} ({})",
                                    data.currency.as_deref().unwrap_or("N/A"),
                                    data.currency_name.as_deref().unwrap_or("N/A")
                                )}
                            </span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.calling_code}</span>
                            <span class="data-value">{data.country_calling_code.as_deref().unwrap_or("N/A")}</span>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    fn view_asn(&self, data: &AsnData, tr: &Translations) -> Html {
        let allocated = data
            .rir_allocation
            .date_allocated
            .as_ref()
            .map(|d| format_date(d))
            .unwrap_or_else(|| "N/A".to_string());
        let updated = data
            .date_updated
            .as_ref()
            .map(|d| format_date(d))
            .unwrap_or_else(|| "N/A".to_string());

        html! {
            <div class="result-box">
                /* ASN Info */
                <div class="segment-card segment-blue">
                    <h2>{tr.asn_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.asn}</span>
                            <span class="data-value">{format!("AS{}", data.asn)}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.name}</span>
                            <span class="data-value">{&data.name}</span>
                        </div>
                        <div class="data-row" style="grid-column: 1 / -1;">
                            <span class="data-label">{tr.description}</span>
                            <span class="data-value">{&data.description_short}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.country}</span>
                            <span class="data-value">{data.country_code.as_deref().unwrap_or("N/A")}</span>
                        </div>
                    </div>
                </div>

                /* Contacts */
                <div class="segment-card segment-green">
                    <h2>{tr.contact_info}</h2>
                    <div class="data-grid" style="grid-template-columns: 1fr;">
                        if !data.website.is_empty() {
                            <div class="data-row">
                                <span class="data-label">{tr.website}</span>
                                <span class="data-value">
                                    <a href={data.website.clone()} target="_blank" rel="noopener noreferrer" style="color: var(--primary);">
                                        {&data.website}
                                    </a>
                                </span>
                            </div>
                        }
                        if !data.email_contacts.is_empty() {
                            <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                                <span class="data-label">{tr.email_contacts}</span>
                                <div style="margin-top: 0.5rem; display: flex; flex-direction: column; gap: 0.25rem;">
                                    {for data.email_contacts.iter().map(|email| html! {
                                        <span class="data-value">{"• "}{email}</span>
                                    })}
                                </div>
                            </div>
                        }
                        if !data.abuse_contacts.is_empty() {
                            <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                                <span class="data-label">{tr.abuse_contacts}</span>
                                <div style="margin-top: 0.5rem; display: flex; flex-direction: column; gap: 0.25rem;">
                                    {for data.abuse_contacts.iter().map(|email| html! {
                                        <span class="data-value">{"• "}{email}</span>
                                    })}
                                </div>
                            </div>
                        }
                    </div>
                </div>

                /* Owner Address */
                if !data.owner_address.is_empty() {
                    <div class="segment-card segment-yellow">
                        <h2>{tr.owner_address}</h2>
                        <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                            {for data.owner_address.iter().map(|line| html! {
                                <span class="data-value">{line}</span>
                            })}
                        </div>
                    </div>
                }

                /* Registry Info */
                <div class="segment-card segment-purple">
                    <h2>{tr.registry_info}</h2>
                    <div class="data-grid">
                        <div class="data-row">
                            <span class="data-label">{tr.rir_name}</span>
                            <span class="data-value">{&data.rir_allocation.rir_name}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.allocated}</span>
                            <span class="data-value">{allocated}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.traffic_ratio}</span>
                            <span class="data-value">{data.traffic_ratio.as_deref().unwrap_or("N/A")}</span>
                        </div>
                        <div class="data-row">
                            <span class="data-label">{tr.last_updated}</span>
                            <span class="data-value">{updated}</span>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

// --- Help utilities ---

fn get_registrar_fn(entity: &WhoisEntity) -> Option<String> {
    let arr = entity.vcard_array.as_array()?;
    let items = arr.get(1)?.as_array()?;
    for item in items {
        let fields = item.as_array()?;
        if fields.first()?.as_str()? == "fn" {
            return fields.get(3)?.as_str().map(|s| s.to_string());
        }
    }
    None
}

fn format_date(date_str: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else if let Ok(dt) = chrono::DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        date_str.replace(['T', 'Z'], " ").trim().to_string()
    }
}

fn get_query_param(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get(name)
}

fn get_hash() -> Option<String> {
    let window = web_sys::window()?;
    let hash = window.location().hash().ok()?;
    if hash.is_empty() { None } else { Some(hash) }
}

fn scroll_to_element(id: &str) {
    let element_id = id.strip_prefix('#').unwrap_or(id);
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(element) = document.get_element_by_id(element_id)
    {
        let html_element = element.dyn_into::<web_sys::HtmlElement>().ok();
        if let Some(el) = html_element {
            el.scroll_into_view();
        }
    }
}

async fn fetch_lookup(query: &str) -> Result<LookupResponse, String> {
    let url = format!("/api/lookup/{}", query);
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        if let Ok(err_json) = resp.json::<serde_json::Value>().await {
            if let Some(err_msg) = err_json.get("message").and_then(|v| v.as_str()) {
                return Err(err_msg.to_string());
            }
            if let Some(err_title) = err_json.get("error").and_then(|v| v.as_str()) {
                return Err(err_title.to_string());
            }
        }
        return Err(format!("Server returned status {}", resp.status()));
    }

    let lookup_data = resp
        .json::<LookupResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(lookup_data)
}

fn main() {
    yew::Renderer::<App>::new().render();
}
