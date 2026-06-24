use yew::prelude::*;
use crate::types::IpData;
use crate::i18n::Translations;

#[derive(Properties, PartialEq)]
pub struct IpProps {
    pub data: IpData,
    pub tr: Translations,
}

#[function_component(ResultIp)]
pub fn result_ip(props: &IpProps) -> Html {
    let data = &props.data;
    let tr = &props.tr;

    html! {
        <div class="result-box">
            /* IP Info */
            <div class="segment-card segment-blue">
                <h2>{&tr.ip_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.ip_address}</span>
                        <span class="data-value">{&data.ip}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.network}</span>
                        <span class="data-value">{data.network.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.version}</span>
                        <span class="data-value">{&data.version}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.org}</span>
                        <span class="data-value">{data.org.as_deref().unwrap_or("N/A")}</span>
                    </div>
                </div>
            </div>

            /* Location Info */
            <div class="segment-card segment-green">
                <h2>{&tr.loc_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.city}</span>
                        <span class="data-value">{data.city.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.region}</span>
                        <span class="data-value">
                            {format!("{} ({})",
                                data.region.as_deref().unwrap_or("N/A"),
                                data.region_code.as_deref().unwrap_or("N/A")
                            )}
                        </span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.country}</span>
                        <span class="data-value">
                            {format!("{} ({})",
                                data.country_name.as_deref().unwrap_or("N/A"),
                                data.country_code.as_deref().unwrap_or("N/A")
                            )}
                        </span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.postal}</span>
                        <span class="data-value">{data.postal.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.continent}</span>
                        <span class="data-value">{data.continent_code.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.timezone}</span>
                        <span class="data-value">{data.timezone.as_deref().unwrap_or("N/A")}</span>
                    </div>
                </div>
            </div>

            /* Coordinates */
            <div class="segment-card segment-yellow">
                <h2>{&tr.geo_coords}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.lat}</span>
                        <span class="data-value">{data.latitude.map(|l| l.to_string()).unwrap_or_else(|| "N/A".to_string())}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.lon}</span>
                        <span class="data-value">{data.longitude.map(|l| l.to_string()).unwrap_or_else(|| "N/A".to_string())}</span>
                    </div>
                </div>
            </div>

            /* Additional Info */
            <div class="segment-card segment-purple">
                <h2>{&tr.add_info}</h2>
                <div class="data-grid">
                    <div class="data-row">
                        <span class="data-label">{&tr.asn}</span>
                        <span class="data-value">{data.asn.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.languages}</span>
                        <span class="data-value">{data.languages.as_deref().unwrap_or("N/A")}</span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.currency}</span>
                        <span class="data-value">
                            {format!("{} ({})",
                                data.currency.as_deref().unwrap_or("N/A"),
                                data.currency_name.as_deref().unwrap_or("N/A")
                            )}
                        </span>
                    </div>
                    <div class="data-row">
                        <span class="data-label">{&tr.calling_code}</span>
                        <span class="data-value">{data.country_calling_code.as_deref().unwrap_or("N/A")}</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
