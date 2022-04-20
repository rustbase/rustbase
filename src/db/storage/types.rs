use serde::{Serialize, Deserialize};

// Current only support string values 
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub key: String,
    pub value: String,
}

impl Data {
    pub fn new(key: String, value: Types) -> Data {
        if let Types::Hash(hash) = value {
            let hash = hash::hash_content(hash);

            return Data {
                key,
                value: Types::Hash(hash),
            }
        };

        Data {
            key,
            value,
        }
    }
}
