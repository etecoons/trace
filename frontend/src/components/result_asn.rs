use crate::i18n::Translations;
use crate::types::AsnData;
use crate::utils::format_date;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AsnProps {
    pub data: AsnData,
    pub tr: Translations,
}

#[function_component(ResultAsn)]
pub fn result_asn(props: &AsnProps) -> Html {
    let data = &props.data;
    let tr = &props.tr;

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
                <h2>{&tr.asn_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.asn}</span>
                        <span class="data-value">{format!("AS{}", data.asn)}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.name}</span>
                        <span class="data-value">{&data.name}</span>
                    </div>
                    <div class="data-row" style="grid-column: 1 / -1;">
                        <span class="data-label">{&tr.description}</span>
                        <span class="data-value">{&data.description_short}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.country}</span>
                        <span class="data-value">{data.country_code.as_deref().unwrap_or("N/A")}</span>
                    </div>
                </div>
            </div>

            /* Contacts */
            <div class="segment-card segment-green">
                <h2>{&tr.contact_info}</h2>
                <div class="data-grid" style="grid-template-columns: 1fr;">
                    if !data.website.is_empty() {
                        <div class="data-row">
                            <span class="data-label">{&tr.website}</span>
                            <span class="data-value">
                                <a href={data.website.clone()} target="_blank" rel="noopener noreferrer" style="color: var(--primary);">
                                    {&data.website}
                                </a>
                            </span>
                        </div>
                    }
                    if !data.email_contacts.is_empty() {
                        <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                            <span class="data-label">{&tr.email_contacts}</span>
                            <div style="margin-top: 0.5rem; display: flex; flex-direction: column; gap: 0.25rem;">
                                {for data.email_contacts.iter().map(|email| html! {
                                    <span class="data-value">{"• "}{email}</span>
                                })}
                            </div>
                        </div>
                    }
                    if !data.abuse_contacts.is_empty() {
                        <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                            <span class="data-label">{&tr.abuse_contacts}</span>
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
                    <h2>{&tr.owner_address}</h2>
                    <div class="data-row" style="flex-direction: column; align-items: flex-start;">
                        {for data.owner_address.iter().map(|line| html! {
                            <span class="data-value">{line}</span>
                        })}
                    </div>
                </div>
            }

            /* Registry Info */
            <div class="segment-card segment-purple">
                <h2>{&tr.registry_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.rir_name}</span>
                        <span class="data-value">{&data.rir_allocation.rir_name}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.allocated}</span>
                        <span class="data-value">{allocated}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.traffic_ratio}</span>
                        <span class="data-value">{data.traffic_ratio.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.last_updated}</span>
                        <span class="data-value">{updated}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
