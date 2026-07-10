# wikiquote-fetcher

Fetches and prepares quotes for Marxist Quote.

This repository contains the standalone fetcher binary and library crate. It
depends on the sibling `engyls` repository for shared configuration types.

Licensed under GPL-3.0-or-later.

## CLI

```bash
wikiquote-fetcher fetch
wikiquote-fetcher current
wikiquote-fetcher cache-path
wikiquote-fetcher translate TR "Workers of the world, unite!"
wikiquote-fetcher pool fetch "Karl Marx"
wikiquote-fetcher pool show "Karl Marx"
```
