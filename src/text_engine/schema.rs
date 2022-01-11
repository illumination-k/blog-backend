use crate::posts::Lang;
use tantivy::schema::*;

pub enum PostField {
    Uuid,
    Slug,
    Title,
    Description,
    Lang,
    Category,
    Tags,
    Body,
    RawText,
    CreatedAt,
    UpdatedAt,
}

impl PostField {
    pub fn as_str(&self) -> &str {
        match self {
            PostField::Uuid => "uuid",
            PostField::Slug => "slug",
            PostField::Title => "title",
            PostField::Description => "description",
            PostField::Lang => "lang",
            PostField::Category => "category",
            PostField::Tags => "tags",
            PostField::Body => "body",
            PostField::RawText => "raw_text",
            PostField::CreatedAt => "created_at",
            PostField::UpdatedAt => "updated_at",
        }
    }
}

pub struct FieldGetter {
    schema: Schema,
}

impl FieldGetter {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub fn get_field(&self, field: PostField) -> Field {
        let field_name = field.as_str();

        self.schema
            .get_field(field_name)
            .unwrap_or_else(|| panic!("Error in PostField: {}", field_name))
    }
}

pub struct SchemaConstructor {
    schema_builder: SchemaBuilder,
}

impl SchemaConstructor {
    pub fn new() -> Self {
        Self {
            schema_builder: Schema::builder(),
        }
    }

    pub fn build_simple_text_fields(&mut self, fields: &[PostField]) {
        fields.iter().for_each(|field| {
            self.schema_builder
                .add_text_field(field.as_str(), TEXT | STORED);
        })
    }

    pub fn build_date_fields(&mut self, fields: &[PostField]) {
        fields.iter().for_each(|field| {
            self.schema_builder.add_date_field(field.as_str(), STORED);
        })
    }

    pub fn build_custom_tokenizer_text_field(
        &mut self,
        tokenizer_name: &str,
        fields: &[PostField],
    ) {
        fields.iter().for_each(|field| {
            self.schema_builder.add_text_field(
                field.as_str(),
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

    pub fn build_custom_tokenizer_text_field_no_stored(
        &mut self,
        tokenizer_name: &str,
        fields: &[PostField],
    ) {
        fields.iter().for_each(|field| {
            self.schema_builder.add_text_field(
                field.as_str(),
                TextOptions::default().set_indexing_options(
                    TextFieldIndexing::default()
                        .set_tokenizer(tokenizer_name)
                        .set_index_option(IndexRecordOption::WithFreqsAndPositions),
                ),
            );
        })
    }
}

pub fn build_schema() -> Schema {
    let mut constructor = SchemaConstructor::new();

    constructor.build_simple_text_fields(&[PostField::Body, PostField::Tags]);
    constructor.build_custom_tokenizer_text_field(
        "raw_tokenizer",
        &[
            PostField::Uuid,
            PostField::Slug,
            PostField::Category,
            PostField::Lang,
        ],
    );
    constructor.build_custom_tokenizer_text_field(
        &Lang::Ja.tokenizer_name(),
        &[PostField::Title, PostField::Description],
    );
    constructor.build_custom_tokenizer_text_field_no_stored(
        &Lang::Ja.tokenizer_name(),
        &[PostField::RawText],
    );
    constructor.build_date_fields(&[PostField::CreatedAt, PostField::UpdatedAt]);

    constructor.schema_builder.build()
}
