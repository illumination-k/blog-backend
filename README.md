# smark

[![codecov](https://codecov.io/gh/illumination-k/smark/branch/main/graph/badge.svg?token=3I8IEVXO2Q)](https://codecov.io/gh/illumination-k/smark)

[![API Documetation](https://github.com/illumination-k/smark/actions/workflows/redoc.yml/badge.svg)]

`smark` is the tool to serve markdown as API server. 

This tool provides API to get markdown posts based on tanitivy (full-text search engine implemented by Rust).

Please see [API Documetation](https://illumination-k.github.io/smark/) for more details.

## Install

You can use release binary from release page.

### Prepare Post

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

### Preparation index

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

### From source

You can use cross to build.

```bash
git clone https://github.com/illumination-k/smark.git
cross build --target x86_64-unknown-linux-musl --release
chmod 777 target/x86_64-unknown-linux-musl/release/smark
```
