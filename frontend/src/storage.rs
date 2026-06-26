pub struct StorageService;

impl StorageService {
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn get_cookie_str() -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let val = js_sys::Reflect::get(&document, &wasm_bindgen::JsValue::from_str("cookie")).ok()?;
        val.as_string()
    }

    fn set_cookie_str(cookie_value: &str) -> Option<()> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let _ = js_sys::Reflect::set(
            &document,
            &wasm_bindgen::JsValue::from_str("cookie"),
            &wasm_bindgen::JsValue::from_str(cookie_value)
        );
        Some(())
    }

    pub fn get_item(key: &str, default: &str) -> String {
        if key == "theme" {
            if let Some(cookie_str) = Self::get_cookie_str() {
                for cookie in cookie_str.split(';') {
                    let parts: Vec<&str> = cookie.split('=').map(|s| s.trim()).collect();
                    if parts.len() >= 2 && parts[0] == "super_metroid_theme" {
                        let val = parts[1].to_string();
                        let clean = if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            val[1..val.len() - 1].to_string()
                        } else {
                            val
                        };
                        let _ = Self::local_storage().map(|s| s.set_item(key, &clean));
                        return clean;
                    }
                }
            }
        }
        let val = Self::local_storage().and_then(|s| s.get_item(key).ok().flatten());
        match val {
            Some(v) => {
                if v.starts_with('"') && v.ends_with('"') && v.len() >= 2 {
                    let clean = v[1..v.len() - 1].to_string();
                    Self::set_item(key, &clean);
                    clean
                } else {
                    v
                }
            }
            None => default.to_string(),
        }
    }

    pub fn set_item(key: &str, value: &str) {
        if let Some(s) = Self::local_storage() {
            let _ = s.set_item(key, value);
        }
        if key == "theme" {
            let cookie_value = format!(
                "super_metroid_theme={}; Path=/; Max-Age=31536000; SameSite=Lax",
                value
            );
            Self::set_cookie_str(&cookie_value);
        }
    }
}
