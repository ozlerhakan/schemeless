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

const FIELD_TYPE_PROPERTIES: [&'static str; 10] = [
    "name",
    "class",
    "positionIncrementGap",
    "autoGeneratePhraseQueries",
    "synonymQueryStyle",
    "enableGraphQueries",
    "docValuesFormat",
    "postingsFormat",
    "subFieldSuffix",
    "dimension",
];

const field_type_classes: [&'static str; 2] = ["solr.", "org.apache.solr.schema."];

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
    } else if &local_name == &"fieldType" {
        // "org.apache.solr.schema" or "solr"
        let class_attribute = attributes
            .iter()
            .filter(|e| &e.name.local_name == &"class")
            .filter(|e| {
                field_type_classes
                    .iter()
                    .any(|prefix| e.value.starts_with(prefix))
            })
            .find(|e| FIELD_TYPE_CLASSES.contains(&e.value.as_str()))
            .expect("");
    }
}
