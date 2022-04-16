use serde::{Serialize, Deserialize};

// Current only support string values 
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub key: String,
    pub value: String,
}

impl Data {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key,
            value
        }
    }
}