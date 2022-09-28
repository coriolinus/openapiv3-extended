use crate::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaData {
    #[serde(default, skip_serializing_if = "is_false")]
    pub nullable: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub write_only: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deprecated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocumentation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<Discriminator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Inline extensions to this object.
    #[serde(flatten, deserialize_with = "crate::util::deserialize_extensions")]
    pub extensions: IndexMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    #[serde(flatten)]
    pub schema_data: SchemaData,
    #[serde(flatten)]
    pub schema_kind: SchemaKind,
}

#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum SchemaKind {
    Type(Type),
    OneOf {
        #[serde(rename = "oneOf")]
        one_of: Vec<ReferenceOr<Schema>>,
    },
    AllOf {
        #[serde(rename = "allOf")]
        all_of: Vec<ReferenceOr<Schema>>,
    },
    AnyOf {
        #[serde(rename = "anyOf")]
        any_of: Vec<ReferenceOr<Schema>>,
    },
    Not {
        not: Box<ReferenceOr<Schema>>,
    },
    Any(AnySchema),
}


impl Schema {
    pub fn new_number() -> Self {
        Self {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Number(NumberType::default())),
        }
    }

    pub fn new_object() -> Self {
        Self {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType::default())),
        }
    }

    pub fn new_str_enum(values: Vec<String>) -> Self {
        Self {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(StringType {
                enumeration: values.into_iter().map(|s| Some(s)).collect(),
                ..StringType::default()
            })),
        }
    }

    pub fn new_string() -> Self {
        Self {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::String(StringType::default())),
        }
    }

    pub fn new_array_ref(reference: &str) -> Self {
        Self {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: Some(ReferenceOr::schema_ref(reference)),
                ..ArrayType::default()
            })),
        }
    }

    pub fn add_property(&mut self, s: &str, schema: Schema) -> Result<()> {
        if let SchemaKind::Type(Type::Object(object_type)) = &mut self.schema_kind {
            object_type.properties.insert(s.to_string(), ReferenceOr::Item(schema));
            Ok(())
        } else {
            Err(anyhow!("Schema is not an object"))
        }
    }

    pub fn add_property_ref(&mut self, s: &str, reference: &str) -> Result<()> {
        if let SchemaKind::Type(Type::Object(object_type)) = &mut self.schema_kind {
            object_type.properties.insert(s.to_string(), ReferenceOr::schema_ref(reference));
            Ok(())
        } else {
            Err(anyhow!("Schema is not an object"))
        }
    }
}

// impl<'de> Deserialize<'de> for SchemaKind {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//         where
//             D: Deserializer<'de>,
//     {
//         let mut map = serde_json::Map::deserialize(deserializer)?;
//         println!("mapf: {:?}", map.keys().collect::<Vec<&String>>());
//         if let Some(one_of) = map.remove("oneOf") {
//             let one_of = serde_json::from_value(one_of).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::OneOf { one_of })
//         } else if let Some(all_of) = map.remove("allOf") {
//             let all_of = serde_json::from_value(all_of).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::AllOf { all_of })
//         } else if let Some(any_of) = map.remove("anyOf") {
//             let any_of = serde_json::from_value(any_of).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::AnyOf { any_of })
//         } else if let Some(not) = map.remove("not") {
//             let not = serde_json::from_value(not).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::Not { not })
//         } else if map.contains_key("type") {
//             println!("found a type: {:?}", map);
//             let typ = serde_json::from_value(serde_json::Value::Object(map)).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::Type(typ))
//         } else if map.contains_key("properties") {
//             let assumed_object = serde_json::Value::Object(map);
//             let object = serde_json::from_value(assumed_object).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::Type(Type::Object(object)))
//         } else {
//             let assumed_object = serde_json::Value::Object(map);
//             let object = serde_json::from_value(assumed_object).map_err(serde::de::Error::custom)?;
//             Ok(SchemaKind::Any(object))
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Type {
    String(StringType),
    Number(NumberType),
    Integer(IntegerType),
    Object(ObjectType),
    Array(ArrayType),
    Boolean {},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Any(bool),
    Schema(Box<ReferenceOr<Schema>>),
}

/// Catch-all for any combination of properties that doesn't correspond to one
/// of the predefined subsets.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnySchema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, ReferenceOr<Schema>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<AdditionalProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<ReferenceOr<Box<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enumeration: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub one_of: Vec<ReferenceOr<Schema>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ReferenceOr<Schema>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub any_of: Vec<ReferenceOr<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<ReferenceOr<Schema>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StringType {
    #[serde(default, skip_serializing_if = "VariantOrUnknownOrEmpty::is_empty")]
    pub format: VariantOrUnknownOrEmpty<StringFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enumeration: Vec<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NumberType {
    #[serde(default, skip_serializing_if = "VariantOrUnknownOrEmpty::is_empty")]
    pub format: VariantOrUnknownOrEmpty<NumberFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub exclusive_minimum: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub exclusive_maximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enumeration: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IntegerType {
    #[serde(default, skip_serializing_if = "VariantOrUnknownOrEmpty::is_empty")]
    pub format: VariantOrUnknownOrEmpty<IntegerFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub exclusive_minimum: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub exclusive_maximum: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
    #[serde(rename = "enum", default, skip_serializing_if = "Vec::is_empty")]
    pub enumeration: Vec<Option<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ObjectType {
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, ReferenceOr<Schema>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<AdditionalProperties>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArrayType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<ReferenceOr<Box<Schema>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub unique_items: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NumberFormat {
    Float,
    Double,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IntegerFormat {
    Int32,
    Int64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StringFormat {
    Date,
    #[serde(rename = "date-time")]
    DateTime,
    Password,
    Byte,
    Binary,
}


impl Schema {
    pub fn properties(&self) -> Option<&IndexMap<String, ReferenceOr<Schema>>> {
        match &self.schema_kind {
            SchemaKind::Type(Type::Object(o)) => Some(&o.properties),
            SchemaKind::Any(AnySchema { properties, .. }) => Some(properties),
            _ => None,
        }
    }

    pub fn properties_iter<'a>(&'a self, spec: &'a OpenAPI) -> Result<Box<dyn Iterator<Item=(&'a String, &'a ReferenceOr<Schema>)> + 'a>> {
        match &self.schema_kind {
            SchemaKind::Type(Type::Object(o)) => Ok(Box::new(o.properties.iter())),
            SchemaKind::Any(AnySchema { properties, .. }) => Ok(Box::new(properties.iter())),
            SchemaKind::AllOf { all_of } => {
                let mut vec = Vec::new();
                for schema in all_of {
                    let schema = schema.resolve(spec).properties_iter(spec)?;
                    vec.extend(schema);
                }
                Ok(Box::new(vec.into_iter()))
            }
            _ => Err(anyhow!("Schema is not an object")),
        }
    }

    pub fn properties_mut(&mut self) -> Option<&mut IndexMap<String, ReferenceOr<Schema>>> {
        match &mut self.schema_kind {
            SchemaKind::Type(Type::Object(ref mut o)) => Some(&mut o.properties),
            SchemaKind::Any(AnySchema { ref mut properties, .. }) => Some(properties),
            _ => None,
        }
    }

    pub fn required(&self, field: &str) -> bool {
        match &self.schema_kind {
            SchemaKind::Type(Type::Object(o)) => o.required.iter().any(|s| s == field),
            SchemaKind::Any(AnySchema { required, .. }) => required.iter().any(|s| s == field),
            _ => true,
        }
    }

    pub fn is_anonymous_object(&self) -> bool {
        match &self.schema_kind {
            SchemaKind::Type(Type::Object(o)) => o.properties.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::{AnySchema, OpenAPI, Schema, SchemaData, SchemaKind};

    #[test]
    fn test_schema_with_extensions() {
        let schema = serde_json::from_str::<Schema>(
            r#"{
                "type": "boolean",
                "x-foo": "bar"
            }"#,
        )
            .unwrap();

        assert_eq!(
            schema.schema_data.extensions.get("x-foo"),
            Some(&json!("bar"))
        );
    }

    #[test]
    fn test_any() {
        let value = json! { {} };
        serde_json::from_value::<AnySchema>(value).unwrap();
    }

    #[test]
    fn test_not() {
        let value = json! {
            {
                "not": {}
            }
        };

        let schema = serde_json::from_value::<Schema>(value).unwrap();
        assert!(matches!(schema.schema_kind, SchemaKind::Not { not: _ }));
    }

    #[test]
    fn test_null() {
        let value = json! {
            {
                "nullable": true,
                "enum": [ null ],
            }
        };

        let schema = serde_json::from_value::<Schema>(value).unwrap();
        assert!(matches!(
            &schema.schema_data,
            SchemaData { nullable: true, .. }
        ));
        assert!(matches!(
            &schema.schema_kind,
            SchemaKind::Any(AnySchema { enumeration, .. }) if enumeration[0] == json!(null)));
    }

    #[test]
    fn test_default_to_object() {
        let s = r##"
required:
  - definition
properties:
  definition:
    type: string
    description: >
      Serialized definition of the version. This should be an OpenAPI 2.x, 3.x or AsyncAPI 2.x file
      serialized as a string, in YAML or JSON.
    example: |
      {asyncapi: "2.0", "info": { "title: … }}
  references:
    type: array
    description: Import external references used by `definition`. It's usually resources not accessible by Bump servers, like local files or internal URLs.
    items:
      $ref: "#/components/schemas/Reference"
"##.trim();
        let s = serde_yaml::from_str::<Schema>(s).unwrap();
        // assert!(matches!(s.schema_kind, SchemaKind::Type(crate::Type::Object(_))), "Schema kind was not expected {:?}", s.schema_kind);
        assert!(matches!(s.schema_kind, SchemaKind::Any(crate::AnySchema{ ref properties, ..}) if properties.len() == 2), "Schema kind was not expected {:?}", s.schema_kind);
    }

    #[test]
    fn test_all_of() {
        let s = r##"
allOf:
  - $ref: "#/components/schemas/DocumentationRequest"
  - $ref: "#/components/schemas/PreviewRequest"
        "##.trim();
        let s = serde_yaml::from_str::<Schema>(s).unwrap();
        match &s.schema_kind {
            SchemaKind::AllOf { all_of } => {
                assert_eq!(all_of.len(), 2);
                assert!(matches!(all_of[0].as_ref_str(), Some("#/components/schemas/DocumentationRequest")));
                assert!(matches!(all_of[1].as_ref_str(), Some("#/components/schemas/PreviewRequest")));
            }
            _ => panic!("Schema kind was not expected {:?}", s.schema_kind)
        }
    }
}

