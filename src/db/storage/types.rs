use std::any::Any;

pub enum Types {
    String,
    Integer,
    Float,
    Uuid,
    Boolean,
    Array,
    Hash,
    Date
}

pub struct Data {
    pub key: String,
    pub value: Box<dyn Any>,
    pub type_: Types
}