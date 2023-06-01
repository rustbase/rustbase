use std::collections::HashMap;

use bson::Bson;

pub struct VariableManager {
    pub variables: HashMap<String, Bson>,
}

impl VariableManager {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: Bson) {
        self.variables.insert(name.to_owned(), value);
    }

    pub fn get(&self, name: &str) -> Option<&Bson> {
        self.variables.get(name)
    }
}
