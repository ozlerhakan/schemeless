use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;

const SCHEME_FIELDS: [&'static str; 10] = [
    "field",
    "fieldType",
    "dynamicField",
    "uniqueKey",
    "analyzer",
    "tokenizer",
    "filter",
    "schema",
    "charFilter",
    "copyField",
];

const FIELD_PROPERTIES: [&str; 17] = [
    "indexed",
    "stored",
    "docValues",
    "sortMissingFirst",
    "sortMissingLast",
    "multiValued",
    "uninvertible",
    "omitNorms",
    "omitTermFreqAndPosition",
    "omitPositions",
    "termVectors",
    "termPosition",
    "termOffsets",
    "termPayloads",
    "required",
    "unseDocValuesAsStored",
    "large",
];

const FIELD_KEYS: [&str; 2] = ["name", "type"];

pub fn solr_parser(name: OwnedName, attributes: Vec<OwnedAttribute>) {
    let local_name = name.local_name.as_str();
    if !SCHEME_FIELDS.contains(&local_name) {
        panic!("Found unsupported schema field: {}.", &local_name)
    }
    if &local_name == &"field" {
        for attribute in &attributes {
            let field_property = attribute.name.local_name.as_str();
            if !FIELD_KEYS.contains(&field_property) && !FIELD_PROPERTIES.contains(&field_property)
            {
                panic!(
                    "Found unsupported field key or property for 'field': {}.",
                    &field_property
                )
            }
            if FIELD_PROPERTIES.contains(&field_property) {
                if attribute.value != "true" && attribute.value != "false" {
                    panic!(
                        "Found unsupported value '{}' for {} type in {}={}.",
                        attribute.value,
                        field_property,
                        local_name,
                        &attributes
                            .iter()
                            .find(|n| n.name.local_name == "name")
                            .unwrap()
                            .value
                    )
                }
            }
        }
    } else if &local_name == &"copyField" {
        let dest = &attributes
            .iter()
            .find(|n| n.name.local_name == "dest")
            .expect("copyField must have the dest attribute.")
            .value;
        let source = &attributes
            .iter()
            .find(|n| n.name.local_name == "source")
            .expect("copyField must have the source attribute.")
            .value;
        if dest == source {
            panic!(
                "dest: '{}' and source: '{}' cannot share the same value in copyField.",
                dest, source
            )
        }
    }
}
