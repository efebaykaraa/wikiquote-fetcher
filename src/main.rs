use anyhow::{Context, bail};
use std::path::PathBuf;
use wikiquote_fetcher::{QuotePoolStore, WikiquoteConfig};

fn usage() {
    eprintln!(
        "Usage:
  wikiquote-fetcher fetch <author>
  wikiquote-fetcher translate <language> <text>
  wikiquote-fetcher pool --dir <path> path <author>
  wikiquote-fetcher pool --dir <path> show <author>
  wikiquote-fetcher pool --dir <path> fetch <author>
  wikiquote-fetcher pool --dir <path> clear <author>"
    );
}

fn pool_command(args: &[String]) -> anyhow::Result<()> {
    if args.first().map(String::as_str) != Some("--dir") {
        bail!("pool commands require --dir <path>");
    }
    let dir = PathBuf::from(args.get(1).context("missing pool dir")?);
    let command = args.get(2).map(String::as_str).context("missing pool command")?;
    let author = args.get(3).context("missing author")?;
    let store = QuotePoolStore::new(dir);

    match command {
        "path" => println!("{}", store.pool_path(author).display()),
        "show" => {
            if let Some(pool) = store.load(author) {
                println!("{}", serde_json::to_string_pretty(&pool)?);
            } else {
                bail!("pool not found for {author}");
            }
        }
        "fetch" => {
            let pool = wikiquote_fetcher::fetch_pool(&store, author, &WikiquoteConfig::default())?;
            println!("saved {} quotes for {}", pool.quotes.len(), author);
        }
        "clear" => {
            store.clear(author)?;
            println!("cleared pool for {author}");
        }
        _ => bail!("unknown pool command: {command}"),
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("fetch") => {
            let author = args.get(1).context("missing author")?;
            let quotes = wikiquote_fetcher::fetch_wikiquote(author)?;
            println!("{}", serde_json::to_string_pretty(&quotes)?);
        }
        Some("translate") => {
            let language = args.get(1).context("missing language")?;
            let text = args.get(2..).context("missing text")?.join(" ");
            println!("{}", wikiquote_fetcher::translate_quote(&text, language)?);
        }
        Some("pool") => pool_command(&args[1..])?,
        Some("help") | Some("--help") | Some("-h") => usage(),
        _ => {
            usage();
            bail!("unknown command");
        }
    }
    Ok(())
}
