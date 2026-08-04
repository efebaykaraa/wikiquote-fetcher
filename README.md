# wikiquote-fetcher

Reusable Rust library and CLI for fetching quotes from Wikiquote, translating text, and optionally storing quote pools in an application-provided directory.

The library is application-agnostic: callers provide the author/page name, optional `WikiquoteConfig`, and optional `QuotePoolStore` path.

## Add to a Rust project

Add the library to a Rust project:

```sh
cargo add wikiquote-fetcher
```

## Install the CLI tool and shared library

Install the command-line application from crates.io:

```sh
cargo install wikiquote-fetcher
```

Install or update the matching shared library for the CLI version:

```sh
wikiquote-fetcher --install-so
```

The command does nothing when the same or a newer library is already installed.
By default it installs updates in `~/.local/lib`; pass a directory to override it.

On Arch Linux and derivatives, it automatically installs both the shared library and the CLI tool from the AUR package [wikiquote-fetcher](https://aur.archlinux.org/packages/wikiquote-fetcher/):

<details open>
<summary><strong>paru</strong></summary>

```sh
paru -S muote
```
</details>

<details closed>
<summary><strong>yay</strong></summary>

```sh
yay -S muote
```
</details>

To build the native package from this repository:

```sh
makepkg -si
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
