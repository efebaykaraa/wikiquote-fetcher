mod pool;
mod wikiquote;

use engyls::config::ConfigManager;
pub use pool::QuotePool;
use rand::Rng;
use serde_json::Value;
use std::path::PathBuf;
pub use wikiquote::fetch_wikiquote;

pub fn cache_file_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("marxist_quote")
        .join("current_quote.txt")
}

pub fn current_quote_exists() -> bool {
    std::fs::read_to_string(cache_file_path())
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false)
}

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
        .query("sl", "en")
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

fn translate_quote(quote: &str, target_language: &str) -> anyhow::Result<String> {
    let target_language = target_language.trim().to_uppercase();
    if target_language == "ORIGINAL" || target_language == "EN" {
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

fn parse_cached_quote(raw_text: &str) -> Option<(String, String)> {
    let (quote, author) = raw_text.rsplit_once(" — ")?;
    Some((
        quote.trim().trim_matches('"').to_string(),
        author.trim().to_string(),
    ))
}

fn remove_cached_quote_from_pool() -> anyhow::Result<()> {
    let cache_file = cache_file_path();
    let raw_text = match std::fs::read_to_string(&cache_file) {
        Ok(text) => text,
        Err(_) => return Ok(()),
    };

    if let Some((quote, author)) = parse_cached_quote(&raw_text) {
        if let Some(mut pool) = QuotePool::load(&author) {
            pool.quotes.retain(|candidate| candidate != &quote);
            let _ = pool.save(&author);
        }
    }

    let _ = std::fs::remove_file(cache_file);
    Ok(())
}

/// Fetch a random quote from WikiQuote for a weighted-random author and save it to the cache.
pub fn fetch_quote() -> anyhow::Result<()> {
    remove_cached_quote_from_pool()?;

    let (authors_cfg, _) = ConfigManager::load_authors();
    let (mut settings_cfg, _) = ConfigManager::load_settings();

    let authors: Vec<_> = authors_cfg
        .authors
        .into_iter()
        .filter(|author| !author.name.trim().is_empty())
        .collect();
    let total_weight: u32 = authors.iter().map(|a| a.weight).sum();
    if total_weight == 0 {
        anyhow::bail!("Total weight of authors is zero");
    }

    let mut rng = rand::rng();
    let mut chosen_weight = rng.random_range(0..total_weight);
    let mut selected_author = authors
        .first()
        .map(|a| a.name.as_str())
        .unwrap_or("Karl Marx");

    for author in &authors {
        if chosen_weight < author.weight {
            selected_author = &author.name;
            break;
        }
        chosen_weight -= author.weight;
    }

    let current_hash = settings_cfg.calculate_position_hash();
    let max_chars = settings_cfg.appearance.max_quote_chars;

    println!(
        "Picking quote for {} (max chars: {}, hash: {})",
        selected_author, max_chars, current_hash
    );

    let mut pool = QuotePool::load(selected_author).unwrap_or_else(|| QuotePool {
        position_hash: String::new(),
        quotes: Vec::new(),
    });

    if pool.position_hash != current_hash || pool.quotes.is_empty() {
        println!("Hash mismatch or empty pool. Refetching from WikiQuote...");
        let new_quotes = fetch_wikiquote(selected_author)?;
        pool.quotes = new_quotes;
        pool.position_hash = current_hash.clone();
        settings_cfg.appearance.position_hash = current_hash;
        let _ = ConfigManager::save_settings(&settings_cfg);
    }

    let mut chosen_quote = String::new();
    while !pool.quotes.is_empty() {
        let idx = rng.random_range(0..pool.quotes.len());
        let q = pool.quotes.remove(idx);

        if q.chars().count() <= max_chars {
            chosen_quote = q;
            break;
        } else {
            println!(
                "Quote too long ({} chars), removing from pool.",
                q.chars().count()
            );
        }
    }

    if chosen_quote.is_empty() {
        anyhow::bail!(
            "No fitting quotes found for {} in current pool. Try resizing or wait for next fetch.",
            selected_author
        );
    }

    let _ = pool.save(selected_author);

    let display_quote = match translate_quote(&chosen_quote, &settings_cfg.appearance.language) {
        Ok(translated) => translated,
        Err(err) => {
            eprintln!(
                "Translation failed for language {}: {}",
                settings_cfg.appearance.language, err
            );
            chosen_quote
        }
    };

    let formatted = format!("\"{}\" — {}", display_quote, selected_author);
    let cache_dir = cache_file_path()
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("~/.cache/marxist_quote"));
    let _ = std::fs::create_dir_all(&cache_dir);
    let _ = std::fs::write(cache_dir.join("current_quote.txt"), &formatted);

    println!("Selected: {}", formatted);
    Ok(())
}

/// Check if at least one quote for ANY configured author fits the character limit.
pub fn any_quote_fits_all_authors(max_chars: usize) -> anyhow::Result<bool> {
    let (authors_cfg, _) = ConfigManager::load_authors();
    for author in &authors_cfg.authors {
        let quotes = fetch_wikiquote(&author.name)?;
        if quotes.iter().any(|q| q.chars().count() <= max_chars) {
            return Ok(true);
        }
    }
    Ok(false)
}
