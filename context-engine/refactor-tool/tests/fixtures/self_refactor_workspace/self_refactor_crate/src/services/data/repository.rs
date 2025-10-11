use std::collections::HashMap;

pub trait Repository<T> {
    fn save(&mut self, id: u64, item: T) -> Result<(), String>;
    fn find(&self, id: u64) -> Option<&T>;
    fn delete(&mut self, id: u64) -> bool;
    fn list_all(&self) -> Vec<&T>;
}

pub struct InMemoryRepository<T> {
    data: HashMap<u64, T>,
}

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<T> Repository<T> for InMemoryRepository<T> {
    fn save(&mut self, id: u64, item: T) -> Result<(), String> {
        self.data.insert(id, item);
        Ok(())
    }
    
    fn find(&self, id: u64) -> Option<&T> {
        self.data.get(&id)
    }
    
    fn delete(&mut self, id: u64) -> bool {
        self.data.remove(&id).is_some()
    }
    
    fn list_all(&self) -> Vec<&T> {
        self.data.values().collect()
    }
}

pub fn create_user_repository<T>() -> InMemoryRepository<T> {
    InMemoryRepository::new()
}

pub fn backup_data<T>(repo: &InMemoryRepository<T>) -> usize {
    repo.data.len()
}