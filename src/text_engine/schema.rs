use crate::posts::Lang;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use tantivy::schema::*;

#[cfg(test)]
use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(test, derive(EnumIter, EnumCount))]
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
    CreatedAtFormat,
    UpdatedAtFormat,
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
            PostField::CreatedAtFormat => "created_at_format",
            PostField::UpdatedAtFormat => "updated_at_format",
        }
    }

    pub fn text_fields() -> [Self; 10] {
        [
            PostField::Uuid,
            PostField::Slug,
            PostField::Title,
            PostField::Lang,
            PostField::Description,
            PostField::Category,
            PostField::Tags,
            PostField::Body,
            PostField::CreatedAtFormat,
            PostField::UpdatedAtFormat,
        ]
    }

    pub fn date_fields() -> [Self; 2] {
        [PostField::CreatedAt, PostField::UpdatedAt]
    }

    pub fn not_stored_fileds() -> [Self; 1] {
        [PostField::RawText]
    }
}

pub struct FieldGetter<'a> {
    schema: &'a Schema,
}

impl<'a> FieldGetter<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        Self { schema }
    }

    pub fn get_field(&self, field: PostField) -> Field {
        let field_name = field.as_str();

        self.schema
            .get_field(field_name)
            .unwrap_or_else(|| panic!("Error in PostField: {}", field_name))
    }

    #[allow(dead_code)]
    pub fn get_fields(&self, fields: &[PostField]) -> Vec<Field> {
        fields.iter().map(|&pf| self.get_field(pf)).collect()
    }

    pub fn get_text(&self, doc: &Document, field: PostField) -> Result<String> {
        if PostField::not_stored_fileds().contains(&field) {
            return Err(anyhow!(format!("{} is not stored field", field.as_str())));
        }
        if PostField::text_fields().contains(&field) {
            Ok(doc
                .get_first(self.get_field(field))
                .unwrap_or_else(|| panic!("Error in get text with {}", field.as_str()))
                .text()
                .unwrap_or_else(|| panic!("Error in get text with {}", field.as_str()))
                .to_string())
        } else {
            Err(anyhow!(format!("{} is not text field", field.as_str())))
        }
    }

    pub fn get_date(&self, doc: &Document, field: PostField) -> Result<DateTime<Utc>> {
        if PostField::not_stored_fileds().contains(&field) {
            return Err(anyhow!(format!("{} is not stored field", field.as_str())));
        }
        if PostField::date_fields().contains(&field) {
            Ok(doc
                .get_first(self.get_field(field))
                .unwrap_or_else(|| panic!("Error in get date with {}", field.as_str()))
                .date_value()
                .unwrap_or_else(|| panic!("Error in get date with {}", field.as_str()))
                .to_owned())
        } else {
            Err(anyhow!(format!("{} is not date field", field.as_str())))
        }
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
            self.schema_builder
                .add_date_field(field.as_str(), FAST | STORED);
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

    constructor.build_simple_text_fields(&[
        PostField::Body,
        PostField::Tags,
        PostField::CreatedAtFormat,
        PostField::UpdatedAtFormat,
    ]);
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

#[cfg(test)]
mod test_textengine_schmea {
    use strum::{EnumCount, IntoEnumIterator};
    use super::*;

    #[test]
    fn test_get_field() {
        let schema = build_schema();
        let fg = FieldGetter::new(&schema);

        for pf in PostField::iter() {
            fg.get_field(pf);
        }
    }

    #[test]
    fn test_postfields_beloging_some_fields_getter() {
        assert_eq!(
            PostField::COUNT,
            PostField::text_fields().len()
                + PostField::date_fields().len()
                + PostField::not_stored_fileds().len()
        )
    }

    #[test]
    fn test_get_text_and_date() {
        let schema = build_schema();
        let fg = FieldGetter::new(&schema);

        let mut doc = Document::new();
        let datetime = Utc::now();

        fg.get_fields(&PostField::text_fields())
            .iter()
            .for_each(|&x| doc.add_text(x, ""));

        fg.get_fields(&PostField::date_fields())
            .iter()
            .for_each(|&x| doc.add_date(x, &datetime));

        PostField::text_fields().iter().for_each(|&x| assert!(
            fg.get_text(&doc, x).is_ok()
        ));

        PostField::date_fields().iter().for_each(|&x| assert!(
            fg.get_date(&doc, x).is_ok()
        ));
    }
}
