use std::collections::HashMap;
use std::str::FromStr;

use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;

const SCHEME_FIELDS: [&'static str; 11] = [
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
    "similarity",
];

const OPTIONAL_FIELD_PROPERTIES: [&str; 18] = [
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
    "default",
];

const FIELD_TYPE_CLASSES: [&'static str; 27] = [
    "BBoxField",
    "BinaryField",
    "BoolField",
    "CollationField",
    "CurrencyFieldType",
    "DateRangeField",
    "DenseVectorField",
    "DatePointField",
    "DoublePointField",
    "ExternalFileField",
    "EnumFieldType",
    "FloatPointField",
    "ICUCollationField",
    "IntPointField",
    "LatLonPointSpatialField",
    "LongPointField",
    "NestPathField",
    "PointType",
    "PreAnalyzedField",
    "RandomSortField",
    "RankField",
    "RptWithGeometrySpatialField",
    "SortableTextField",
    "SpatialRecursivePrefixTreeFieldType",
    "StrField",
    "TextField",
    "UUIDField",
];

const DEPRECATED_FIELD_TYPES: [&'static str; 8] = [
    "CurrencyField",
    "EnumField",
    "TrieDateField",
    "TrieDoubleField",
    "TrieFloatField",
    "TrieIntField",
    "TrieLongField",
    "TrieField",
];

const FIELD_TYPE_GENERAL_PROPERTIES: [&'static str; 7] = [
    "name",
    "positionIncrementGap",
    "autoGeneratePhraseQueries",
    "synonymQueryStyle",
    "enableGraphQueries",
    "docValuesFormat",
    "postingsFormat",
];

// SOLR-17274: https://issues.apache.org/jira/browse/SOLR-17274
const PRESERVED_SOLR_NAMES: [&'static str; 3] = ["set", "add", "remove"];

const SOLR_CONSTANT_TYPE_NAMES: [&'static str; 4] =
    ["_root_", "_version_", "_nest_path_", "_text_"];

const FIELD_TYPE_CLASSES_NAMES: [&'static str; 2] = ["solr.", "org.apache.solr.schema."];

const FIELD_DEFINITIONS: [&str; 2] = ["name", "type"];

#[derive(Debug, PartialEq)]
enum SolrFields {
    Field,
    CopyField,
    FieldType,
    DynamicField,
    Unknown(String),
}

impl FromStr for SolrFields {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "field" => Ok(SolrFields::Field),
            "copyField" => Ok(SolrFields::CopyField),
            "fieldType" => Ok(SolrFields::FieldType),
            "dynamicField" => Ok(SolrFields::DynamicField),
            _ => Ok(SolrFields::Unknown(s.to_string())),
        }
    }
}

pub fn schema_parser(
    names: &mut Vec<String>,
    types: &mut HashMap<String, String>,
    name: &OwnedName,
    attributes: Vec<OwnedAttribute>,
) {
    let local_name = name.local_name.as_str();
    if !SCHEME_FIELDS.contains(&local_name) {
        panic!("Found unsupported schema field: {}.", &local_name)
    }
    let required_fields: Vec<&str> = FIELD_DEFINITIONS.iter().copied().collect();
    let attribute_names: Vec<&str> = attributes
        .iter()
        .map(|attr| attr.name.local_name.as_str())
        .collect();

    match SolrFields::from_str(&local_name) {
        Ok(field_enum) => match field_enum {
            SolrFields::Field | SolrFields::DynamicField => {
                let all_required = check_required_field(&required_fields, attribute_names);
                if !all_required {
                    panic!(
                        "Found unsupported field key or property for 'field': {:?}.",
                        required_fields,
                    )
                }
                for attribute in &attributes {
                    let field_property = attribute.name.local_name.as_str();
                    if !OPTIONAL_FIELD_PROPERTIES.contains(&field_property) {
                        match attribute.name.local_name.as_str() {
                            "name" => {}
                            "type" => {}
                            "default" => {}
                            _ => {
                                panic!(
                                    "Found some optional fields are incorrectly defined for 'field': {}.",
                                    &field_property
                                )
                            }
                        }
                    }
                    if OPTIONAL_FIELD_PROPERTIES.contains(&field_property) {
                        match attribute.name.local_name.as_str() {
                            "default" => {}
                            _ => {
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
                    }
                    if field_property == "type" {
                        let field_name = &attributes.iter().find(|x| x.name.local_name == "name");
                        let field_name_owned = field_name.unwrap();
                        if types.contains_key(&field_name_owned.value.to_owned()) {
                            panic!(
                                "Found duplicate types with the same name: '{}'.",
                                field_name_owned.value
                            )
                        }
                        types.insert(
                            field_name_owned.value.to_owned(),
                            attribute.value.to_string(),
                        );
                    }
                }
                check_duplicate_field_names(names, &local_name, &attributes);
            }
            SolrFields::CopyField => {
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
            SolrFields::FieldType => {
                let deprecated_attribute = attributes
                    .iter()
                    .filter(|e| e.name.local_name == "class")
                    .any(|e| {
                        return DEPRECATED_FIELD_TYPES
                            .contains(&&e.value.split('.').last().unwrap());
                    });
                if deprecated_attribute {
                    panic!("Found deprecated class in the fieldType declaration: {:?}. Please consider changing it with the new equivalent type: https://solr.apache.org/guide/solr/latest/indexing-guide/field-types-included-with-solr.html#deprecated-field-types", &attributes)
                }
                // check a class that starts with "org.apache.solr.schema" or "solr" and has support one of FIELD_TYPE_CLASSES
                let class_attribute: Vec<_> = attributes
                    .iter()
                    .filter(|e| &e.name.local_name == &"class")
                    .filter(|e| {
                        FIELD_TYPE_CLASSES_NAMES
                            .iter()
                            .any(|prefix| e.value.starts_with(prefix))
                    })
                    .filter(|e| {
                        FIELD_TYPE_CLASSES
                            .iter()
                            .any(|class_name| e.value.ends_with(class_name))
                    })
                    .cloned()
                    .collect();
                if class_attribute.is_empty() {
                    panic!(
                        "Found an undefined class type in the fieldType declaration: {:?}",
                        &attributes
                    )
                }
                let not_any_attribute = attributes
                    .iter()
                    .all(|s| !FIELD_TYPE_GENERAL_PROPERTIES.contains(&s.name.local_name.as_str()));
                if not_any_attribute {
                    panic!(
                        "Could not find any attributes of the fieldType: {:?}.",
                        FIELD_TYPE_GENERAL_PROPERTIES
                    );
                }
                check_duplicate_field_names(names, &local_name, &attributes);
            }
            SolrFields::Unknown(e) => {
                println!("skipping field, {:?}", &e)
            }
        },
        Err(_) => (),
    }
}

fn check_required_field(required_fields: &Vec<&str>, attribute_names: Vec<&str>) -> bool {
    let all_required = required_fields
        .iter()
        .all(|&field| attribute_names.contains(&field));
    all_required
}

fn check_duplicate_field_names(
    names: &mut Vec<String>,
    local_name: &str,
    attributes: &Vec<OwnedAttribute>,
) {
    let local_name_option = &attributes.iter().find(|x| x.name.local_name == "name");
    let name_value = &local_name_option.unwrap().value;
    let name_with_tag = format!("{}:{}", local_name, name_value.as_str());
    if PRESERVED_SOLR_NAMES.contains(&name_value.as_str()) {
        panic!("Found the reserved keyword '{name_value}' being used in '{local_name}'.")
    }
    if names.contains(&name_with_tag) && !SOLR_CONSTANT_TYPE_NAMES.contains(&name_value.as_str()) {
        panic!("Found duplicate field names '{}'.", name_value)
    }
    names.push(name_with_tag);
}
