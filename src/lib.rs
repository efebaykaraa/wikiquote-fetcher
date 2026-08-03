pub mod pool;
mod wikiquote;

pub use pool::{QuotePool, QuotePoolStore};
use serde_json::Value;
pub use wikiquote::{WikiquoteConfig, fetch_wikiquote, fetch_wikiquote_with_config};

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
