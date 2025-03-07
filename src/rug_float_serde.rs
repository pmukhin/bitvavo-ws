use rug::float::Round::Nearest;
use rug::Float;
use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::Serialize;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct FloatWrapper {
    pub float: Float,
    pub str_repr: String,
}

impl Display for FloatWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Float(repr={} fl={})",
            self.str_repr,
            self.float.to_string_radix_round(10, Some(10), Nearest)
        )
    }
}

impl<'de> Deserialize<'de> for FloatWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FloatVisitor;

        impl Visitor<'_> for FloatVisitor {
            type Value = FloatWrapper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON string representing a floating-point number")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Float::parse(value)
                    .map_err(|err| de::Error::custom(format!("failed to parse Float: {}", err)))
                    .map(|parse_incomplete| FloatWrapper {
                        float: Float::with_val(53, parse_incomplete),
                        str_repr: value.to_owned(),
                    })
            }
        }

        // Deserialize the JSON as a string, then parse it into a Float
        deserializer.deserialize_str(FloatVisitor)
    }
}

// Optional: Implement Serialize for the wrapper
impl Serialize for FloatWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.str_repr.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize_float() {
        let json_data = r#"
            "100.50"
        "#;

        // Deserialize the JSON into a PriceLevel
        let float: FloatWrapper = serde_json::from_str(json_data).unwrap();
        assert_eq!("100.50", float.str_repr);
        assert_eq!(Float::with_val(10, 100.5), float.float);
    }
}
