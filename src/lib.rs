use chrono::NaiveDateTime;
use noisy_float::prelude::*;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "dataset")]
pub struct Dataset {
    #[serde(rename = "$value")]
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "frame")]
pub struct Frame {
    pub number: usize,
    pub file: PathBuf,
    #[serde(with = "serde_utc")]
    pub utc: NaiveDateTime,
    pub color: Option<Color>,
    pub depth: Option<Depth>,
    pub gsd: Option<R64>,
    pub x: R64,
    pub y: R64,
    pub lat: R64,
    pub lon: R64,
    #[serde(with = "serde_zero_one_bool")]
    pub sunny: bool,
    #[serde(rename = "$value")]
    pub object_list: ObjectList,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "objectlist")]
pub struct ObjectList {
    #[serde(rename = "$value")]
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "object")]
pub struct Object {
    pub id: usize,
    pub r#box: Box,
    pub representation: Representation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "box")]
pub struct Box {
    pub xc: R64,
    pub yc: R64,
    pub w: R64,
    pub h: R64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "representation")]
pub struct Representation {
    pub r#type: RepresentationType,
    pub xc: R64,
    pub yc: R64,
    pub w: R64,
    pub h: R64,
    pub o: R64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepresentationType {
    RotatedRectangle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    #[serde(rename = "rgb")]
    Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Depth {
    #[serde(rename = "byte")]
    Byte,
}

mod serde_utc {
    use super::*;

    const FORMAT: &str = "%Y-%b-%d %H:%M:%S%.f";

    pub fn serialize<S>(value: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        format!("{}", value.format(FORMAT)).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&text, FORMAT).map_err(|err| {
            D::Error::custom(format!(
                "unable to deserialize string '{}' to date: {:?}",
                text, err
            ))
        })
    }
}

mod serde_zero_one_bool {
    use super::*;

    pub fn serialize<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *value { "1" } else { "0" }.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        let value = match &*text {
            "0" => false,
            "1" => true,
            text => {
                return Err(D::Error::custom(format!(
                    r#"expect "0" or "1", but get "{}""#,
                    text
                )))
            }
        };
        Ok(value)
    }
}
