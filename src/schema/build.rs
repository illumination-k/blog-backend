use tantivy::schema::*;

use crate::utils::Lang;

pub fn add_need_field(schema_builder: &mut SchemaBuilder) {
    let text_fields = ["id", "slug", "body", "lang", "category", "tags"];

    text_fields.into_iter().for_each(|field| {
        schema_builder.add_text_field(field, TEXT | STORED);
    });
    schema_builder.add_date_field("created_at", STORED);
    schema_builder.add_date_field("updated_at", STORED);
}

pub fn build_schema() -> Schema {
    let tokenizer_name = Lang::Ja.tokenizer_name();
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field(
        "title",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(&tokenizer_name)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    schema_builder.add_text_field(
        "description",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(&tokenizer_name)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    schema_builder.add_text_field(
        "raw_text",
        TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(&tokenizer_name)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored(),
    );

    add_need_field(&mut schema_builder);
    schema_builder.build()
}
