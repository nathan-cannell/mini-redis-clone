use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct Db {
    data: Arc<DashMap<String, Vec<u8>>>,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }
}

impl Db {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).map(|v| v.clone())
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn delete(&self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        let db = Db::new();
        let key = "test_key".to_string();
        let value = b"test_value".to_vec();
        
        db.set(key.clone(), value.clone());
        assert_eq!(db.get(&key), Some(value));
    }

    #[test]
    fn test_delete() {
        let db = Db::new();
        let key = "test_key".to_string();
        let value = b"test_value".to_vec();
        
        db.set(key.clone(), value);
        assert!(db.delete(&key));
        assert_eq!(db.get(&key), None);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;
        
        let db = Arc::new(Db::new());
        let mut handles = vec![];
        
        for i in 0..10 {
            let db = db.clone();
            let handle = thread::spawn(move || {
                let key = format!("key_{}", i);
                let value = format!("value_{}", i).into_bytes();
                db.set(key.clone(), value.clone());
                assert_eq!(db.get(&key), Some(value));
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
    }
} 