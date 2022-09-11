use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Cache {
    cache: BTreeMap<String, CacheNode>,
    cache_size: usize,
    max_size: usize,
}

#[derive(Clone, Debug)]
struct CacheNode {
    value: bson::Document,
    size: usize,
    insert_at: std::time::SystemTime,
}

impl Cache {
    pub fn new(max_cache_size: usize) -> Self {
        Cache {
            cache: BTreeMap::new(),
            cache_size: 0,
            max_size: max_cache_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<&bson::Document> {
        if !self.cache.contains_key(key) {
            return None;
        }

        let value = &self.cache.get(key).unwrap().value;
        Some(value)
    }

    pub fn insert(&mut self, key: String, value: bson::Document) -> CResult<()> {
        if self.cache.contains_key(&key) {
            return Err(CacheError {
                code: CacheErrorCode::KeyExists,
            });
        }
        let value_size = std::mem::size_of_val(&value);

        if self.is_cache_full() || self.cache_size + value_size > self.max_size {
            self.manage_cache(value_size);

            // check again
            if self.is_cache_full() || self.cache_size + value_size > self.max_size {
                return Err(CacheError {
                    code: CacheErrorCode::CacheFull,
                });
            }
        }

        let node = CacheNode {
            value,
            size: value_size,
            insert_at: std::time::SystemTime::now(),
        };

        self.add_size(value_size);

        self.cache.insert(key, node);
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> CResult<()> {
        if !self.cache.contains_key(key) {
            return Err(CacheError {
                code: CacheErrorCode::KeyNotExists,
            });
        }

        let value = self.cache.remove(key).unwrap();
        self.remove_size(value.size);

        Ok(())
    }

    pub fn contains(&self, key: String) -> bool {
        self.cache.contains_key(&key)
    }

    fn is_cache_full(&self) -> bool {
        if self.cache_size >= self.max_size {
            return true;
        }

        false
    }

    fn add_size(&mut self, size: usize) {
        self.cache_size += size;
    }

    fn remove_size(&mut self, size: usize) {
        self.cache_size -= size;
    }

    fn manage_cache(&mut self, size_to_insert: usize) {
        // remove oldest entry with size_to_insert

        let cache = self.cache.clone();
        let cache = cache.iter();

        let cache = cache.filter(|(_, node)| node.size <= size_to_insert);

        let mut cache = cache.collect::<Vec<(_, _)>>();
        cache.sort_by(|a, b| a.1.insert_at.cmp(&b.1.insert_at));

        let (key, _) = cache.pop().unwrap();
        self.remove(key).unwrap();
    }
}

pub type CResult<T> = std::result::Result<T, CacheError>;

#[derive(Debug)]
pub enum CacheErrorCode {
    KeyNotExists,
    KeyExists,
    CacheFull,
}

#[derive(Debug)]
pub struct CacheError {
    code: CacheErrorCode,
}

impl std::fmt::Display for CacheErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheErrorCode::KeyExists => write!(f, "KeyExists"),
            CacheErrorCode::CacheFull => write!(f, "CacheFull"),
            CacheErrorCode::KeyNotExists => write!(f, "KeyNotExists"),
        }
    }
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheError {}", self.code)
    }
}
