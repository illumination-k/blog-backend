# smark

[![codecov](https://codecov.io/gh/illumination-k/smark/branch/main/graph/badge.svg?token=3I8IEVXO2Q)](https://codecov.io/gh/illumination-k/smark)
[![API Documetation](https://github.com/illumination-k/smark/actions/workflows/redoc.yml/badge.svg)](https://illumination-k.github.io/smark/)

`smark` is the tool to serve markdown as the API server.

This tool provides API to get markdown documents from files based on [tantivy](https://github.com/quickwit-oss/tantivy) (A full-text search engine in Rust) and [lindera](https://github.com/lindera-morphology/lindera) (A morphological analysis library in Rust).

Please see [API Documetation](https://illumination-k.github.io/smark/) for more details. `smark` also provides [openapi schema](./openapi.yml).

## Install

You can use release binary from release page.

### Prepare Posts

You can make template markdown with required frontmatters by `template` subcommand.

```bash
smark template

# ---
# uuid: db71b71a-c7f2-47c4-ab87-81bd1bb6d58a
# title: ""
# description: ""
# lang: ja
# category: ""
# ---
```

### Prepare index

You need to prepare index to register your markdown posts.
Please specify input markdown direcotry and output index direcotry.

`smark` detect `posts/**/*.md` files.

```bash
smark prep --index-dir index --input posts
```

### Run server

You completed all steps! Let's run server!

```bash
smark run --index-dir index --static-dir images
```

## From source

You can use cross to build.

```bash
git clone https://github.com/illumination-k/smark.git
cross build --target x86_64-unknown-linux-musl --release
chmod 777 target/x86_64-unknown-linux-musl/release/smark
```
