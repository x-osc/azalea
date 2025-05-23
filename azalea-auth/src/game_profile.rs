use std::{collections::HashMap, sync::Arc};

use azalea_buf::AzBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(AzBuf, Debug, Clone, Default, Eq, PartialEq)]
pub struct GameProfile {
    /// The UUID of the player.
    pub uuid: Uuid,
    /// The username of the player.
    pub name: String,
    // this is an arc to make GameProfile cheaper to clone when the properties are big
    pub properties: Arc<HashMap<String, ProfilePropertyValue>>,
}

impl GameProfile {
    pub fn new(uuid: Uuid, name: String) -> Self {
        GameProfile {
            uuid,
            name,
            properties: Arc::new(HashMap::new()),
        }
    }
}

impl From<SerializableGameProfile> for GameProfile {
    fn from(value: SerializableGameProfile) -> Self {
        let mut properties = HashMap::new();
        for value in value.properties {
            properties.insert(
                value.name,
                ProfilePropertyValue {
                    value: value.value,
                    signature: value.signature,
                },
            );
        }
        Self {
            uuid: value.id,
            name: value.name,
            properties: Arc::new(properties),
        }
    }
}

#[derive(AzBuf, Debug, Clone, Eq, PartialEq)]
pub struct ProfilePropertyValue {
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<SerializableProfilePropertyValue>,
}

impl From<GameProfile> for SerializableGameProfile {
    fn from(value: GameProfile) -> Self {
        let mut properties = Vec::new();
        for (key, value) in &*value.properties {
            properties.push(SerializableProfilePropertyValue {
                name: key.clone(),
                value: value.value.clone(),
                signature: value.signature.clone(),
            });
        }
        Self {
            id: value.uuid,
            name: value.name,
            properties,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableProfilePropertyValue {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_game_profile() {
        let json = r#"{
            "id": "f1a2b3c4-d5e6-f7a8-b9c0-d1e2f3a4b5c6",
            "name": "Notch",
            "properties": [
                {
                    "name": "qwer",
                    "value": "asdf",
                    "signature": "zxcv"
                }
            ]
        }"#;
        let profile =
            GameProfile::from(serde_json::from_str::<SerializableGameProfile>(json).unwrap());
        assert_eq!(
            profile,
            GameProfile {
                uuid: Uuid::parse_str("f1a2b3c4-d5e6-f7a8-b9c0-d1e2f3a4b5c6").unwrap(),
                name: "Notch".to_string(),
                properties: {
                    let mut map = HashMap::new();
                    map.insert(
                        "qwer".to_string(),
                        ProfilePropertyValue {
                            value: "asdf".to_string(),
                            signature: Some("zxcv".to_string()),
                        },
                    );
                    map.into()
                },
            }
        );
    }
}
