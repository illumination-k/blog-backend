use tantivy::schema::*;

use crate::utils::Lang;

pub struct SchemaConstructor {
    schema_builder: SchemaBuilder,
}

impl SchemaConstructor {
    pub fn new() -> Self {
        Self {
            schema_builder: Schema::builder(),
        }
    }

    pub fn build_simple_text_fields(&mut self, fields: &[&str]) {
        fields.into_iter().for_each(|field_name_str| {
            self.schema_builder
                .add_text_field(field_name_str, TEXT | STORED);
        })
    }

    pub fn build_date_fields(&mut self, fields: &[&str]) {
        fields.into_iter().for_each(|field_name_str| {
            self.schema_builder.add_date_field(field_name_str, STORED);
        })
    }

    pub fn build_custom_tokenizer_text_field(&mut self, tokenizer_name: &str, fields: &[&str]) {
        fields.into_iter().for_each(|field_name_str| {
            self.schema_builder.add_text_field(
                field_name_str,
                TextOptions::default()
                    .set_indexing_options(
                        TextFieldIndexing::default()
                            .set_tokenizer(tokenizer_name)
                            .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                    )
                    .set_stored(),
            );
        })
    }
}

pub fn build_schema() -> Schema {
    let mut constructor = SchemaConstructor::new();

    constructor.build_simple_text_fields(&["slug", "body", "tags"]);
    constructor.build_custom_tokenizer_text_field("raw_tokenizer", &["uuid", "category", "lang"]);
    constructor.build_custom_tokenizer_text_field(&Lang::Ja.tokenizer_name(), &["title", "description", "raw_text"]);
    constructor.build_date_fields(&["created_at", "updated_at"]);

    constructor.schema_builder.build()
}
