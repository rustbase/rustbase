use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct Cache {
    index: HashMap<String, usize>,
    cache: VecDeque<bson::Bson>,
    cache_size: usize,
    max_size: usize,
}

impl Cache {
    pub fn new(max_cache_size: usize) -> Self {
        Cache {
            index: HashMap::new(),
            cache: VecDeque::new(),
            cache_size: 0,
            max_size: max_cache_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<&bson::Bson> {
        if !self.index.contains_key(key) {
            return None;
        }

        let index = self.index.get(key).unwrap();
        let value = self.cache.get(*index).unwrap();
        Some(value)
    }

    pub fn insert(&mut self, key: String, value: bson::Bson) -> CResult<()> {
        if self.index.contains_key(&key) {
            return Err(CacheError {
                code: CacheErrorCode::KeyExists,
            });
        }
        let value_size = std::mem::size_of_val(&value);

        self.manage_cache(value_size);

        self.cache.push_back(value);
        let index = self.cache.len() - 1;
        self.index.insert(key, index);

        self.cache_size += value_size;

        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> CResult<()> {
        if !self.index.contains_key(key) {
            return Err(CacheError {
                code: CacheErrorCode::KeyNotExists,
            });
        }

        let index = self.index.remove(key).unwrap();
        let value = self.cache.remove(index).unwrap();
        let value_size = std::mem::size_of_val(&value);

        self.cache_size -= value_size;

        Ok(())
    }

    fn manage_cache(&mut self, size_to_insert: usize) {
        while self.cache_size + size_to_insert > self.max_size {
            let value = self.cache.pop_front();

            if value.is_none() {
                break;
            }

            let value_size = std::mem::size_of_val(&value);
            println!("[Cache] removing {} bytes", value_size);
            self.cache_size -= value_size;
        }
    }
}

pub type CResult<T> = std::result::Result<T, CacheError>;

#[derive(Debug)]
pub enum CacheErrorCode {
    KeyNotExists,
    KeyExists,
}

#[derive(Debug)]
pub struct CacheError {
    code: CacheErrorCode,
}

impl std::fmt::Display for CacheErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheErrorCode::KeyExists => write!(f, "KeyExists"),
            CacheErrorCode::KeyNotExists => write!(f, "KeyNotExists"),
        }
    }
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheError {}", self.code)
    }
}
