use crate::schema::schema_parser;
use clap::Parser;
use std::io::BufReader;
use std::{collections::HashMap, fs::File};
use xml::reader::{EventReader, XmlEvent};

mod schema;
#[derive(Parser, Debug)]
#[command(name = "schemeless")]
#[command(bin_name = "schemeless")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
#[group(required = true)]
struct SchemaArgs {
    #[arg(id="file", short='f', value_parser = clap::value_parser!(String))]
    file: String,
}

fn main() -> std::io::Result<()> {
    let args = SchemaArgs::parse();
    let file = File::open(args.file)?;
    let file: BufReader<&File> = BufReader::new(&file);
    schema_operations(file);
    Ok(())
}

fn schema_operations<R: std::io::Read>(reader: R) {
    let buf_reader = BufReader::new(reader);
    let parser = EventReader::new(buf_reader);

    let mut names: Vec<String> = Vec::new();
    let mut copy_fields: Vec<String> = Vec::new();
    let mut field_types: HashMap<String, String> = HashMap::new();
    let mut unique_key_exists = false;
    let mut id_field = String::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                schema_parser(
                    &mut names,
                    &mut copy_fields,
                    &mut field_types,
                    &name,
                    attributes,
                );
                let local_name = name.local_name.as_str();
                if local_name == "uniqueKey" {
                    unique_key_exists = true;
                }
            }
            Ok(XmlEvent::Characters(ref data)) => {
                if unique_key_exists {
                    id_field = data.to_owned();
                    unique_key_exists = false;
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }
    if !id_field.is_empty() && !names.contains(&format!("field:{}", &id_field)) {
        panic!(
            "Could not found the field '{}' among the field types.",
            id_field
        )
    }
    for (key_field, value_field_type) in field_types.iter() {
        if !names.contains(&format!("fieldType:{}", value_field_type)) {
            panic!(
                "Could not find the type '{}' defined in '{}'",
                value_field_type, key_field
            )
        }
    }
    for key in &copy_fields {
        let field_name = &format!("field:{}", key);
        if names.contains(&field_name) {
            continue;
        }
        panic!("Could not find the field type '{}' in one copyField.", key)
    }
}

#[cfg(test)]
mod tests {
    use crate::schema_operations;
    use std::io::Cursor;

    #[test]
    #[should_panic(expected = "Found unsupported schema field: fiedTtype")]
    fn test_schema_with_incorrect_definition() {
        let example = r#"
        <schema version="1.6">
            <similarity class="solr.BM25SimilarityFactory" />
            <fiedTtype name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Could not found the field 'id' among the field types")]
    fn test_schema_with_missing_uniquekey() {
        let example = r#"
        <schema version="1.6">
        <uniqueKey>id</uniqueKey>
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    fn test_schema_with_correct_attributes() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <fieldType name="id_unique" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Could not find the type 'id_unique' defined in 'id'")]
    fn test_schema_with_incorrect_type_reference() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(
        expected = "Found unsupported field key or property for 'field': [\"name\", \"type\"]"
    )]
    fn test_schema_with_missing_type() {
        let example = r#"
        <schema version="1.6">
        <field name="id" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found unsupported value 'TruE' for stored type in field=id")]
    fn test_schema_with_incorrect_bool_value() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="TruE" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(
        expected = "Found some optional fields are incorrectly defined for 'field': equired."
    )]
    fn test_schema_with_incorrect_attribute() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" equired="true" stored="TruE" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "copyField must have the source attribute.")]
    fn test_copyfied_source() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField dest="doi_string" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "copyField must have the dest attribute.")]
    fn test_copyfied_dest() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi"  />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(
        expected = "dest: 'doi' and source: 'doi' cannot share the same value in copyField"
    )]
    fn test_copyfied_source_dest() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi" dest="doi" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found an undefined class type in the fieldType declaration")]
    fn test_undefined_solr_class() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi" dest="doid" />
        <fieldType name="pdates" class="solr.datePointField" docValues="true" multiValued="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found deprecated class in the fieldType declaration")]
    fn test_deprecated_type() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi" dest="doid" />
        <fieldType name="int" class="solr.TrieDoubleField" positionIncrementGap="0" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Could not find any attributes of the fieldType")]
    fn test_general_attributes() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi" dest="doid" />
        <fieldType class="solr.DoublePointField"  />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Could not find the field type 'doid' in one copyField")]
    fn test_field_not_found_in_copy_field() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="id_unique" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <copyField source="doi" dest="doid" />
        <fieldType name="doi" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found duplicate field names 'string'")]
    fn test_duplicate_value() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found the reserved keyword")]
    fn test_dreserved_keyword() {
        let example = r#"
        <schema version="1.6">
        <field name="id" type="id_unique" required="true" stored="true" />
        <fieldType name="add" class="solr.StrField" sortMissingLast="true" docValues="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    #[should_panic(expected = "Found duplicate types with the same name")]
    fn test_duplicate_field_name_keyword() {
        let example = r#"
        <schema version="1.6">
            <field name="id" type="id_unique" required="true" stored="true" />
            <fieldType name="string" class="solr.StrField" sortMissingLast="true" docValues="true" />
            <field name="id" type="id_unique" required="true" stored="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }

    #[test]
    fn test_correct_schema() {
        let example = r#"
        <schema version="1.6">
            <field name="id" type="id_unique" required="true" stored="true" />
            <fieldType name="id_unique" class="solr.StrField" sortMissingLast="true" docValues="true" />
            <field name="ids" type="id_unique" required="true" stored="true" />
        </schema>
        "#;
        let cursor = Cursor::new(example);
        schema_operations(cursor)
    }
}
