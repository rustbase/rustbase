#[derive(Clone)]
pub struct Data {
    pub key: String,
    pub value: bson::Bson,
}

impl Data {
    pub fn new(key: String, value: bson::Bson) -> Self {
        match value {
            bson::Bson::JavaScriptCode(_) => {
                panic!("JavaScriptCode is not supported");
            }

            bson::Bson::JavaScriptCodeWithScope(_) => {
                panic!("JavaScriptCodeWithScope is not supported");
            }

            bson::Bson::DbPointer(_) => {
                panic!("DbPointer is not supported");
            }

            _ => ()
        }

        Self {
            key,
            value,
        }
    }
}