pub mod pool;
mod wikiquote;

pub use pool::{QuotePool, QuotePoolStore};
use serde_json::Value;
pub use wikiquote::{WikiquoteConfig, fetch_wikiquote, fetch_wikiquote_with_config};

fn normalize_deeplx_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim().trim_end_matches('/');
    if trimmed.ends_with("/translate") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/translate")
    }
}

fn deeplx_endpoints() -> Vec<String> {
    if let Ok(configured) = std::env::var("DEEPLX_URL") {
        return vec![normalize_deeplx_endpoint(&configured)];
    }

    vec![
        normalize_deeplx_endpoint("http://127.0.0.1:1188"),
        normalize_deeplx_endpoint("http://localhost:1188"),
    ]
}

fn translated_text_from_value(value: &Value) -> Option<String> {
    value
        .get("data")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .get("data")
                .and_then(Value::as_array)
                .and_then(|items| items.first())
                .and_then(Value::as_str)
        })
        .or_else(|| value.get("translation").and_then(Value::as_str))
        .or_else(|| value.get("text").and_then(Value::as_str))
        .or_else(|| {
            value
                .get("translations")
                .and_then(Value::as_array)
                .and_then(|translations| translations.first())
                .and_then(|translation| translation.get("text"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            value
                .get("data")
                .and_then(|data| data.get("translations"))
                .and_then(Value::as_array)
                .and_then(|translations| translations.first())
                .and_then(|translation| translation.get("text"))
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn translate_with_deeplx(quote: &str, target_language: &str) -> anyhow::Result<String> {
    let mut errors = Vec::new();
    for endpoint in deeplx_endpoints() {
        for source_lang in ["EN", "AUTO"] {
            let body = serde_json::json!({
                "text": quote,
                "source_lang": source_lang,
                "target_lang": target_language,
            });

            let mut response = match ureq::post(&endpoint).send_json(&body) {
                Ok(response) => response,
                Err(err) => {
                    errors.push(format!("{endpoint} ({source_lang}): {err}"));
                    continue;
                }
            };

            let raw = match response.body_mut().read_to_string() {
                Ok(raw) => raw,
                Err(err) => {
                    errors.push(format!(
                        "{endpoint} ({source_lang}): failed to read body: {err}"
                    ));
                    continue;
                }
            };

            let value: Value = match serde_json::from_str(&raw) {
                Ok(value) => value,
                Err(err) => {
                    errors.push(format!(
                        "{endpoint} ({source_lang}): invalid JSON: {err}: {raw}"
                    ));
                    continue;
                }
            };

            if let Some(translated) = translated_text_from_value(&value) {
                return Ok(translated);
            }

            let message = value
                .get("message")
                .and_then(Value::as_str)
                .or_else(|| value.get("msg").and_then(Value::as_str))
                .unwrap_or("response did not contain translated text");
            errors.push(format!(
                "{endpoint} ({source_lang}): DeepLX {message}: {raw}"
            ));
        }
    }

    anyhow::bail!("DeepLX translation failed: {}", errors.join("; "))
}

fn google_language_code(language: &str) -> String {
    language.trim().to_lowercase().replace('_', "-")
}

fn google_translated_text(value: &Value) -> Option<String> {
    let sentences = value.get(0)?.as_array()?;
    let translated = sentences
        .iter()
        .filter_map(|sentence| sentence.get(0).and_then(Value::as_str))
        .collect::<String>();

    let translated = translated.trim();
    if translated.is_empty() {
        None
    } else {
        Some(translated.to_string())
    }
}

fn translate_with_google(quote: &str, target_language: &str) -> anyhow::Result<String> {
    let target_language = google_language_code(target_language);
    let mut response = ureq::get("https://translate.googleapis.com/translate_a/single")
        .query("client", "gtx")
        .query("sl", "auto")
        .query("tl", &target_language)
        .query("dt", "t")
        .query("q", quote)
        .call()
        .map_err(|err| anyhow::anyhow!("Google Translate request failed: {err}"))?;
    let raw = response.body_mut().read_to_string()?;
    let value: Value = serde_json::from_str(&raw)?;

    google_translated_text(&value).ok_or_else(|| {
        anyhow::anyhow!("Google Translate response did not contain translated text: {raw}")
    })
}

pub fn translate_quote(quote: &str, target_language: &str) -> anyhow::Result<String> {
    let target_language = target_language.trim().to_uppercase();
    if target_language == "ORIGINAL" || target_language == "AUTO" {
        return Ok(quote.to_string());
    }

    if std::env::var("DEEPLX_URL").is_ok() {
        match translate_with_deeplx(quote, &target_language) {
            Ok(translated) => return Ok(translated),
            Err(err) => eprintln!("{err}. Falling back to Google Translate."),
        }
    }

    translate_with_google(quote, &target_language)
}

pub fn fetch_pool(
    store: &QuotePoolStore,
    author: &str,
    config: &WikiquoteConfig,
) -> anyhow::Result<QuotePool> {
    let quotes = fetch_wikiquote_with_config(author, config)?;
    let pool = QuotePool {
        key: author.to_string(),
        quotes,
    };
    store.save(&pool)?;
    Ok(pool)
}
