# wikiquote-fetcher

Reusable Rust library and CLI for fetching quotes from Wikiquote, translating text, and optionally storing quote pools in an application-provided directory.

The library is application-agnostic: callers provide the author/page name, optional `WikiquoteConfig`, and optional `QuotePoolStore` path.

## Installation

Install the command-line application from crates.io:

```sh
cargo install wikiquote-fetcher
```

On Arch Linux and derivatives, install the native AUR package instead:

```sh
yay -S wikiquote-fetcher
# or
paru -S wikiquote-fetcher
```

Add the library to a Rust project:

```sh
cargo add wikiquote-fetcher
```

## CLI

```sh
wikiquote-fetcher fetch "Rosa Luxemburg"
wikiquote-fetcher translate TR "Workers of the world, unite!"
wikiquote-fetcher pool --dir ~/.cache/my-quote-app/pools fetch "Rosa Luxemburg"
wikiquote-fetcher pool --dir ~/.cache/my-quote-app/pools show "Rosa Luxemburg"
```

## Library

```rust,no_run
use wikiquote_fetcher::{QuotePoolStore, WikiquoteConfig, fetch_wikiquote_with_config};

fn main() -> anyhow::Result<()> {
    let config = WikiquoteConfig::default();
    let quotes = fetch_wikiquote_with_config("Rosa Luxemburg", &config)?;

    let store = QuotePoolStore::new("./quote-pools");
    if let Some(quote) = quotes.first() {
        println!("{quote}");
    }
    println!("Pools are stored in {}", store.dir().display());
    Ok(())
}
```

Translation uses Google Translate. Wikiquote requests use the English
Wikiquote API.

## License

GPL-3.0-or-later
