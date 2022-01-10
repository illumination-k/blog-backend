use crate::posts::Post;
use anyhow::Result;
use tantivy::{schema::Schema, Document};
use yaml_rust::YamlEmitter;

pub fn dump_doc(doc: &Document, schema: &Schema) -> Result<(String, String)> {
    let post = Post::from_doc(doc, schema);

    let matter = post.matter().to_yaml();
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&matter)?;
    out_str.push_str("\n---\n");
    out_str.push_str(&post.body());

    let mut filename = post.lang().as_str().to_string();
    filename.push('/');
    filename.push_str(&post.slug());
    filename.push_str(".md");
    Ok((filename, out_str))
}
