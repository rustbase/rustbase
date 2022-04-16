use crate::db::crypto;
use chrono::prelude::*;
use hex::ToHex;

use super::types::Data;

#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub content: Vec<u8>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn create(name: String, content: &[u8]) -> Self {
        let id = crypto::generate_bytes::generate_random_bytes();

        Self {
            id: id.encode_hex::<String>(),
            name,
            content: content.to_vec(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

pub fn create_document(name: String, data: Vec<Data>) -> Document {
    // Parse the data into a vector of bytes
    let bytes_data = unsafe {
        any_as_u8_slice(&data)
    };

    // Create the document
    let document = Document::create(name, bytes_data);

    return document;
}
