use crate::i18n::Translations;
use crate::types::WhoisData;
use crate::utils::{format_date, get_registrar_fn};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WhoisProps {
    pub data: WhoisData,
    pub tr: Translations,
    pub on_lookup_ip: Callback<String>,
}

#[function_component(ResultWhois)]
pub fn result_whois(props: &WhoisProps) -> Html {
    let data = &props.data;
    let tr = &props.tr;
    let on_lookup_ip = &props.on_lookup_ip;

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
                    <span>{&tr.domain_info}</span>
                    <a href="#domain-info" class="permalink" title="Permalink to this section">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                        </svg>
                    </a>
                </h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.domain}</span>
                        <span class="data-value">{&data.ldh_name}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.handle}</span>
                        <span class="data-value">{&data.handle}</span>
                    </div>
                </div>
            </div>

            /* IP Addresses */
            <div id="ip-addresses" class="segment-card segment-red">
                <h2>
                    <span>{&tr.ip_addresses}</span>
                    <a href="#ip-addresses" class="permalink" title="Permalink to this section">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                        </svg>
                    </a>
                </h2>

                <div class="data-grid" style="grid-template-columns: 1fr;">
                    <div class="data-row" style="flex-direction: column; gap: 0.5rem; align-items: flex-start;">
                        <span class="data-label">{&tr.ipv4}</span>
                        if data.ip_addresses.v4.is_empty() {
                            <span class="data-value">{&tr.no_ipv4}</span>
                        } else {
                            <div class="badge-list">
                                {for data.ip_addresses.v4.iter().map(|ip| {
                                    let ip_clone = ip.clone();
                                    let cb = on_lookup_ip.clone();
                                    html! {
                                        <button class="ip-button" onclick={Callback::from(move |_| cb.emit(ip_clone.clone()))}>
                                            {ip}
                                        </button>
                                    }
                                })}
                            </div>
                        }
                    </div>
                    <div class="data-row" style="flex-direction: column; gap: 0.5rem; align-items: flex-start;">
                        <span class="data-label">{&tr.ipv6}</span>
                        if data.ip_addresses.v6.is_empty() {
                            <span class="data-value">{&tr.no_ipv6}</span>
                        } else {
                            <div class="badge-list">
                                {for data.ip_addresses.v6.iter().map(|ip| {
                                    let ip_clone = ip.clone();
                                    let cb = on_lookup_ip.clone();
                                    html! {
                                        <button class="ip-button" onclick={Callback::from(move |_| cb.emit(ip_clone.clone()))}>
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
                        <span>{&tr.domain_status}</span>
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
                    <span>{&tr.important_dates}</span>
                    <a href="#important-dates" class="permalink" title="Permalink to this section">
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="theme-icon">
                            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
                        </svg>
                    </a>
                </h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.registration}</span>
                        <span class="data-value">{creation}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.expiration}</span>
                        <span class="data-value">{expiration}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.last_updated}</span>
                        <span class="data-value">{updated}</span>
                    </div>
                </div>
            </div>

            /* Nameservers */
            if !data.nameservers.is_empty() {
                <div id="nameservers" class="segment-card segment-yellow">
                    <h2>
                        <span>{&tr.nameservers}</span>
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
                <h2>{&tr.registrar_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.registrar}</span>
                        <span class="data-value">{registrar_name}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
