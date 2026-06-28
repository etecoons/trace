use crate::app::App;
use crate::components::header::Header;
use crate::components::pin::PinEntry;
use crate::components::result_asn::ResultAsn;
use crate::components::result_ip::ResultIp;
use crate::components::result_whois::ResultWhois;
use crate::i18n::get_translations;
use crate::types::*;
use yew::prelude::*;

impl App {
    pub fn view_app(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let show_version = self.show_version;
        let show_github = self.show_github;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let version_url = format!(
            "https://github.com/UberMetroid/trace/releases/tag/v{}",
            version
        );

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

        html! {
            <>
                <Header
                    site_title={self.site_title.clone()}
                    theme={self.theme.clone()}
                    language={self.language}
                    toggle_theme={ctx.link().callback(|_| Msg::ToggleTheme)}
                    on_language_change={ctx.link().callback(Msg::SwitchLanguage)}
                    is_authenticated={self.is_authenticated}
                    pin_required={self.pin_required}
                    on_logout={ctx.link().callback(|_| Msg::Logout)}
                    logout_tooltip={tr.logout_tooltip.to_string()}
                    on_print={ctx.link().callback(|_| Msg::PrintPage)}
                    print_tooltip={tr.print_tooltip.to_string()}
                    print_disabled={!self.enable_print || self.response.is_none()}
                    theme_toggle_tooltip={tr.toggle_theme.to_string()}
                    enable_translation={self.enable_translation}
                    enable_themes={self.enable_themes}
                    
                />
                <div class="container">
                    if !self.is_authenticated {
                        <PinEntry
                            translations={tr.clone()}
                            pin_length={self.pin_length}
                            pin_input={self.pin_input.clone()}
                            error_message={self.error_message.clone()}
                            on_input_change={ctx.link().callback(Msg::PinInputChanged)}
                            on_submit={ctx.link().callback(|_| Msg::VerifyPin)}
                        />
                    } else {
                        <div class="app">
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
                                    <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::PerformLookup)} title={tr.lookup}>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px;">
                                            <circle cx="11" cy="11" r="8"></circle>
                                            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                                        </svg>
                                        {tr.lookup}
                                    </button>
                                </div>
                                <p class="examples-text">
                                    {tr.examples}{": "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("yahoo.com".to_string()))}>{"yahoo.com"}</a>{", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("8.8.8.8".to_string()))}>{"8.8.8.8"}</a>{", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("2001:4860:4860::8888".to_string()))}>{"2001:4860:4860::8888"}</a>{", "}
                                    <a class="example-link" onclick={ctx.link().callback(|_| Msg::LookupIP("AS13335".to_string()))}>{"AS13335"}</a>
                                </p>
                            </section>
                            if self.loading {
                                <div class="loading-indicator">
                                    <div class="spinner"></div>
                                    <p>{tr.loading}</p>
                                </div>
                            }
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
                                            LookupResponse::Whois(whois_data) => html! {
                                                <ResultWhois data={whois_data.clone()} tr={tr.clone()} on_lookup_ip={ctx.link().callback(Msg::LookupIP)} />
                                            },
                                            LookupResponse::Ip(ip_data) => html! {
                                                <ResultIp data={ip_data.clone()} tr={tr.clone()} />
                                            },
                                            LookupResponse::Asn(asn_data) => html! {
                                                <ResultAsn data={asn_data.clone()} tr={tr.clone()} />
                                            },
                                        }}
                                    </div>
                                </section>
                            }
                        </div>
                    }
                </div>
                <crate::components::footer::Footer {show_version} {version} {show_github} {version_url}>
                    <div class={classes!("footer-status-text", self.status_type.clone())}>
                        {&self.status_text}
                    </div>
                </crate::components::footer::Footer>
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
