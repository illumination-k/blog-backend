use crate::posts::Post;
use anyhow::Result;
use tantivy::{schema::Schema, Document};
use yaml_rust::YamlEmitter;

pub fn dump_doc(doc: &Document, schema: &Schema) -> Result<(String, String)> {
    let post = Post::from_doc(doc, schema);

    let updated_at_field = schema.get_field("updated_at").unwrap();
    let created_at_field = schema.get_field("created_at").unwrap();

    let updated_at = doc
        .get_first(updated_at_field)
        .unwrap()
        .date_value()
        .unwrap()
        .to_rfc3339();
    let created_at = doc
        .get_first(created_at_field)
        .unwrap()
        .date_value()
        .unwrap()
        .to_rfc3339();

    let matter = post.matter().to_yaml_with_date(created_at, updated_at);
    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(&matter)?;
    out_str.push_str("\n---\n");
    out_str.push_str(&post.body());

    let mut filename = post.lang().as_str().to_string();
    filename.push('/');
    filename.push_str(&post.slug());
    filename.push_str(".md");
    Ok((filename,  out_str))
}
