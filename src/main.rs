use anyhow::{Context, bail};
use wikiquote_fetcher::{QuotePool, cache_file_path};

fn usage() {
    eprintln!(
        "Usage:
  wikiquote-fetcher
  wikiquote-fetcher fetch
  wikiquote-fetcher current
  wikiquote-fetcher cache-path
  wikiquote-fetcher clear-cache
  wikiquote-fetcher translate <language> <text>
  wikiquote-fetcher fits <max-chars>
  wikiquote-fetcher wikiquote <author>
  wikiquote-fetcher pool path <author>
  wikiquote-fetcher pool show <author>
  wikiquote-fetcher pool fetch <author>
  wikiquote-fetcher pool clear <author>"
    );
}

fn pool_command(args: &[String]) -> anyhow::Result<()> {
    let command = args.first().map(String::as_str).context("missing pool command")?;
    let author = args.get(1).context("missing author")?;
    match command {
        "path" => println!("{}", QuotePool::pool_path(author).display()),
        "show" => {
            if let Some(pool) = QuotePool::load(author) {
                println!("{}", serde_json::to_string_pretty(&pool)?);
            } else {
                bail!("pool not found for {author}");
            }
        }
        "fetch" => {
            let pool = wikiquote_fetcher::fetch_pool(author)?;
            println!("saved {} quotes for {}", pool.quotes.len(), author);
        }
        "clear" => {
            wikiquote_fetcher::clear_pool(author)?;
            println!("cleared pool for {author}");
        }
        _ => bail!("unknown pool command: {command}"),
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("fetch") => wikiquote_fetcher::fetch_quote()?,
        Some("current") => {
            if let Some(quote) = wikiquote_fetcher::read_cached_quote()? {
                println!("{quote}");
            } else {
                bail!("no cached quote at {}", cache_file_path().display());
            }
        }
        Some("cache-path") => println!("{}", cache_file_path().display()),
        Some("clear-cache") => {
            wikiquote_fetcher::clear_cached_quote()?;
            println!("cleared {}", cache_file_path().display());
        }
        Some("translate") => {
            let language = args.get(1).context("missing language")?;
            let text = args.get(2..).context("missing text")?.join(" ");
            println!("{}", wikiquote_fetcher::translate_quote(&text, language)?);
        }
        Some("fits") => {
            let max_chars: usize = args.get(1).context("missing max-chars")?.parse()?;
            println!("{}", wikiquote_fetcher::any_quote_fits_all_authors(max_chars)?);
        }
        Some("wikiquote") => {
            let author = args.get(1).context("missing author")?;
            let quotes = wikiquote_fetcher::fetch_wikiquote(author)?;
            println!("{}", serde_json::to_string_pretty(&quotes)?);
        }
        Some("pool") => pool_command(&args[1..])?,
        Some("help") | Some("--help") | Some("-h") => usage(),
        Some(command) => {
            usage();
            bail!("unknown command: {command}");
        }
    }
    Ok(())
}
