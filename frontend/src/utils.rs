use crate::types::WhoisEntity;
use wasm_bindgen::JsCast;

pub fn get_registrar_fn(entity: &WhoisEntity) -> Option<String> {
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

pub fn format_date(date_str: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else if let Ok(dt) = chrono::DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        date_str.replace(['T', 'Z'], " ").trim().to_string()
    }
}

pub fn get_query_param(name: &str) -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get(name)
}

pub fn get_hash() -> Option<String> {
    let window = web_sys::window()?;
    let hash = window.location().hash().ok()?;
    if hash.is_empty() { None } else { Some(hash) }
}

pub fn scroll_to_element(id: &str) {
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
