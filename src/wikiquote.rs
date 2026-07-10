use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct WikiquoteConfig {
    pub user_agent: String,
    pub max_quotes: usize,
}

impl Default for WikiquoteConfig {
    fn default() -> Self {
        Self {
            user_agent: "wikiquote-fetcher/1.0".into(),
            max_quotes: 200,
        }
    }
}

pub fn fetch_wikiquote(author: &str) -> anyhow::Result<Vec<String>> {
    fetch_wikiquote_with_config(author, &WikiquoteConfig::default())
}

pub fn fetch_wikiquote_with_config(
    author: &str,
    config: &WikiquoteConfig,
) -> anyhow::Result<Vec<String>> {
    let page_title = author.replace(' ', "_");

    let sections_url = format!(
        "https://en.wikiquote.org/w/api.php?action=parse&page={}&format=json&prop=sections",
        urlencoded(&page_title)
    );

    let sections_body: String = ureq::get(&sections_url)
        .header("User-Agent", &config.user_agent)
        .call()?
        .body_mut()
        .read_to_string()?;

    let sections_json: serde_json::Value = serde_json::from_str(&sections_body)?;

    let sections = sections_json["parse"]["sections"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse sections for {}", author))?;

    let mut quote_section_indices: Vec<String> = Vec::new();
    let mut in_quotes_section = false;

    for section in sections {
        let toclevel = section["toclevel"].as_u64().unwrap_or(0);
        let line = section["line"].as_str().unwrap_or("");
        let index = section["index"].as_str().unwrap_or("");

        if toclevel == 1 {
            in_quotes_section = line == "Quotes";
        }

        if in_quotes_section && toclevel == 2 && !index.is_empty() {
            quote_section_indices.push(index.to_string());
        }
    }

    if quote_section_indices.is_empty() {
        if let Some(quotes_section) = sections
            .iter()
            .find(|s| s["line"].as_str() == Some("Quotes"))
        {
            if let Some(idx) = quotes_section["index"].as_str() {
                quote_section_indices.push(idx.to_string());
            }
        }
    }

    if quote_section_indices.is_empty() {
        for i in 1..=5 {
            quote_section_indices.push(i.to_string());
        }
    }

    let mut all_quotes: Vec<String> = Vec::new();
    for section_idx in &quote_section_indices {
        let section_url = format!(
            "https://en.wikiquote.org/w/api.php?action=parse&page={}&format=json&prop=text&section={}",
            urlencoded(&page_title),
            section_idx
        );

        let body: String = match ureq::get(&section_url)
            .header("User-Agent", &config.user_agent)
            .call()
        {
            Ok(mut resp) => resp.body_mut().read_to_string()?,
            Err(_) => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let html_str = match json["parse"]["text"]["*"].as_str() {
            Some(s) => s,
            None => continue,
        };

        let extracted = extract_quotes_from_html(html_str);
        all_quotes.extend(extracted);

        if all_quotes.len() >= config.max_quotes {
            break;
        }
    }

    Ok(all_quotes)
}

fn extract_quotes_from_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let ul_sel = Selector::parse("div.mw-parser-output > ul").unwrap();
    let li_sel = Selector::parse(":scope > li").unwrap();

    let mut quotes = Vec::new();
    for ul in document.select(&ul_sel) {
        for li in ul.select(&li_sel) {
            let quote_text = extract_direct_text(&li);
            let cleaned = clean_quote(&quote_text);
            if cleaned.len() >= 20 && !is_attribution(&cleaned) {
                quotes.push(cleaned);
            }
        }
    }
    quotes
}

fn extract_direct_text(element: &scraper::ElementRef) -> String {
    let mut text = String::new();
    for child in element.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                text.push_str(t);
            }
            scraper::node::Node::Element(el) => {
                let tag = el.name();
                if tag == "ul"
                    || tag == "dl"
                    || tag == "span"
                        && el
                            .attr("class")
                            .map_or(false, |c| c.contains("editsection"))
                {
                    continue;
                }
                if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                    if tag == "sup" {
                        continue;
                    }
                    text.push_str(&extract_direct_text(&child_ref));
                }
            }
            _ => {}
        }
    }
    text
}

fn clean_quote(text: &str) -> String {
    let mut s = text.trim().to_string();
    for mark in &['"', '“', '”', '«', '»', '\u{201C}', '\u{201D}'] {
        if s.starts_with(*mark) {
            s = s.trim_start_matches(*mark).to_string();
        }
        if s.ends_with(*mark) {
            s = s.trim_end_matches(*mark).to_string();
        }
    }
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn is_attribution(text: &str) -> bool {
    let lower = text.to_lowercase();
    let prefixes = [
        "as quoted in",
        "letter from",
        "letter to",
        "quoted in",
        "source:",
        "variant:",
        "see also",
        "compare:",
        "attributed",
        "paraphrase",
        "often misquoted",
        "sometimes attributed",
        "this is often",
    ];
    prefixes.iter().any(|p| lower.starts_with(p))
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('#', "%23")
}
