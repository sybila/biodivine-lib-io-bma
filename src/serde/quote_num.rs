use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// In rare cases, the XML and JSON representations can contain numbers inside quotes (e.g. `"32"`
/// instead of `32`). To fix this, we try to parse all numbers using this special struct
/// with a dedicated serialization methods.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) struct QuoteNum(u32);

impl From<u32> for QuoteNum {
    fn from(value: u32) -> Self {
        QuoteNum(value)
    }
}

impl From<QuoteNum> for u32 {
    fn from(value: QuoteNum) -> Self {
        value.0
    }
}

impl Serialize for QuoteNum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        u32::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for QuoteNum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct QuoteNumVisitor;

        impl<'de> Visitor<'de> for QuoteNumVisitor {
            type Value = QuoteNum;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number or a number string")
            }

            fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(QuoteNum(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value <= u32::MAX as u64 {
                    Ok(QuoteNum(value as u32))
                } else {
                    Err(E::custom(format!("number {} is too large for u32", value)))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let trimmed = value.trim_matches('"');
                let parsed = u32::from_str(trimmed).map_err(E::custom)?;
                Ok(QuoteNum(parsed))
            }
        }

        deserializer.deserialize_any(QuoteNumVisitor)
    }
}

#[cfg(test)]
mod tests {
    use crate::serde::quote_num::QuoteNum;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_quote_num_serialization() {
        #[derive(Serialize, Deserialize)]
        struct Test {
            item: QuoteNum,
        }

        let good_1 = r#"{ "item": 1 }"#;
        let good_2 = r#"{ "item": "1" }"#;
        let good_3 = r#"{ "item": "\"1\"" }"#;

        let x_1: Test = serde_json::from_str(good_1).unwrap();
        let x_2: Test = serde_json::from_str(good_2).unwrap();
        let x_3: Test = serde_json::from_str(good_3).unwrap();

        assert_eq!(x_1.item, QuoteNum(1));
        assert_eq!(x_2.item, QuoteNum(1));
        assert_eq!(x_3.item, QuoteNum(1));

        let x = Test { item: QuoteNum(2) };
        let x_json = serde_json::to_string(&x).unwrap();
        assert_eq!(x_json, r#"{"item":2}"#);
    }
}
