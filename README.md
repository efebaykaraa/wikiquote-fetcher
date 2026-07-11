# wikiquote-fetcher

Reusable Rust library and CLI for fetching quotes from Wikiquote, translating text, and optionally storing quote pools in an application-provided directory.

The library is application-agnostic: callers provide the author/page name, optional `WikiquoteConfig`, and optional `QuotePoolStore` path.

## AUR

> [!TIP]
> **wikiquote-fetcher** is available on the Arch User Repository: [`wikiquote-fetcher`](https://aur.archlinux.org/packages/wikiquote-fetcher)
>
> ```sh
> yay -S wikiquote-fetcher
> ```

## CLI

```sh
wikiquote-fetcher fetch "Rosa Luxemburg"
wikiquote-fetcher translate TR "Workers of the world, unite!"
wikiquote-fetcher pool --dir ~/.cache/my-quote-app/pools fetch "Rosa Luxemburg"
wikiquote-fetcher pool --dir ~/.cache/my-quote-app/pools show "Rosa Luxemburg"
```
