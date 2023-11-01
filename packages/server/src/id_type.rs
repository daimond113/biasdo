use derive_more::{Display, From, FromStr};
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Display, From, FromStr, sqlx::Type, TS)]
#[sqlx(transparent)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct Id(#[ts(type = "`${number}`")] pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, From, sqlx::Type, TS)]
#[sqlx(transparent)]
#[ts(export, export_to = "../server-utils/src/")]
pub struct OptionId(#[ts(type = "`${number}` | undefined")] pub Option<u64>);

impl OptionId {
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct IdVisitor;

        impl<'de> de::Visitor<'de> for IdVisitor {
            type Value = Id;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing a number")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match u64::from_str(value) {
                    Ok(id) => Ok(Id(id)),
                    Err(_) => Err(E::custom("invalid id")),
                }
            }
        }

        deserializer.deserialize_str(IdVisitor)
    }
}

impl Serialize for OptionId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self.0 {
            Some(id) => serializer.serialize_str(&id.to_string()),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for OptionId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OptionIdVisitor;

        impl<'de> de::Visitor<'de> for OptionIdVisitor {
            type Value = OptionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string containing a number or null")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match u64::from_str(value) {
                    Ok(id) => Ok(OptionId(Some(id))),
                    Err(_) => Err(E::custom("invalid id")),
                }
            }

            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(OptionId(None))
            }
        }

        deserializer.deserialize_option(OptionIdVisitor)
    }
}
