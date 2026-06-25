#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub site_title: String,
    pub pin: Option<String>,
    pub max_attempts: usize,
    pub lockout_time_minutes: u64,
    pub cookie_max_age_hours: i64,
    pub trust_proxy: bool,
    pub trusted_proxies: Vec<ipnet::IpNet>,
    pub allowed_origins: String,
    pub base_url: String,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::from_path("/app/data/.env").ok();
        dotenvy::dotenv().ok();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4404);

        let site_title = std::env::var("RUSTWHO_TITLE")
            .or_else(|_| std::env::var("RUSTWHO_SITE_TITLE"))
            .or_else(|_| std::env::var("SITE_TITLE"))
            .unwrap_or_else(|_| "RustWho".to_string());

        let pin = std::env::var("RUSTWHO_PIN")
            .or_else(|_| std::env::var("PIN"))
            .ok()
            .filter(|p| {
                !p.is_empty()
                    && p.chars().all(|c| c.is_ascii_digit())
                    && p.len() >= 4
                    && p.len() <= 10
            });

        let max_attempts = std::env::var("MAX_ATTEMPTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);

        let lockout_time_minutes = std::env::var("LOCKOUT_TIME")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15);

        let cookie_max_age_hours = std::env::var("COOKIE_MAX_AGE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        let trust_proxy = std::env::var("TRUST_PROXY")
            .map(|s| s == "true")
            .unwrap_or(false);

        let trusted_proxy_ips_raw = std::env::var("TRUSTED_PROXY_IPS").unwrap_or_default();
        let mut trusted_proxies = Vec::new();
        for s in trusted_proxy_ips_raw.split(',') {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                if let Ok(net) = trimmed.parse::<ipnet::IpNet>() {
                    trusted_proxies.push(net);
                } else if let Ok(ip) = trimmed.parse::<std::net::IpAddr>() {
                    let bits = match ip {
                        std::net::IpAddr::V4(_) => 32,
                        std::net::IpAddr::V6(_) => 128,
                    };
                    if let Ok(net) = ipnet::IpNet::new(ip, bits) {
                        trusted_proxies.push(net);
                    }
                }
            }
        }

        let allowed_origins = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());

        let base_url =
            std::env::var("BASE_URL").unwrap_or_else(|_| format!("http://localhost:{}", port));

        let enable_translation = std::env::var("ENABLE_TRANSLATION")
            .map(|v| v == "true" || v == "on")
            .unwrap_or(false);

        let enable_themes = std::env::var("ENABLE_THEMES")
            .map(|v| v != "false" && v != "off")
            .unwrap_or(true);

        let enable_print = std::env::var("ENABLE_PRINT")
            .map(|v| v != "false" && v != "off")
            .unwrap_or(true);

        Self {
            port,
            site_title,
            pin,
            max_attempts,
            lockout_time_minutes,
            cookie_max_age_hours,
            trust_proxy,
            trusted_proxies,
            allowed_origins,
            base_url,
            enable_translation,
            enable_themes,
            enable_print,
        }
    }
}
