use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use serde::de::{Error as DeError, Visitor};
use std::fmt;

pub fn serialize<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Always store in Mongo as a real date
    chrono_datetime_as_bson_datetime::serialize(value, serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DateTimeFallbackVisitor)
}

struct DateTimeFallbackVisitor;

impl<'de> Visitor<'de> for DateTimeFallbackVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ISO8601 string OR a BSON date sub-document")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        // If it's a plain string (like from JSON request), parse it as ISO8601
        value.parse::<DateTime<Utc>>().map_err(DeError::custom)
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        // If it's a map, we assume it's the MongoDB { "$date": ... } sub-document
        chrono_datetime_as_bson_datetime::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}
