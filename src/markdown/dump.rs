use crate::posts::Post;
use anyhow::Result;
use tantivy::{schema::Schema, Document};
use yaml_rust::YamlEmitter;

use super::frontmatter::FrontMatter;

pub fn dump_matter(matter: &FrontMatter) -> Result<String> {
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&matter.to_yaml())?;
    out_str.push_str("\n---\n");

    Ok(out_str)
}

pub fn dump_doc(doc: &Document, schema: &Schema) -> Result<(String, String)> {
    let post = Post::from_doc(doc, schema)?;

    let mut out_str = dump_matter(&post.matter())?;
    out_str.push_str(&post.body());

    let mut filename = post.lang().as_str().to_string();
    filename.push('/');
    filename.push_str(&post.slug());
    filename.push_str(".md");
    Ok((filename, out_str))
}
