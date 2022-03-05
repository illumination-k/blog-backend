# smark

`smark` is the tool to serve markdown as API server. 

This tool provides API to get blog posts based on tanitivy (full-text search engine implemented by Rust).

You can use following APIs. The method of all APIs is `GET`.

|endpoint|query parameters|description|
|---|---|---|
|post/uuid/$(uuid)|None|Get a blog post by uuid|
|posts|-|get all posts|
|categories|-|all categories|
|tags|-|all tags|
|search|query|Search Blog Post by tanitivy query language|

## Install

You can use release binary.

### From source

You can use cross to build.

```bash
git clone https://github.com/illumination-k/smark.git
cross build --target x86_64-unknown-linux-musl --release
chmod 777 target/x86_64-unknown-linux-musl/release/smark
```
