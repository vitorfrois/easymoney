use std::collections::HashMap;

use crate::models::Category;

#[derive(Clone)]
pub struct FieldMap<V> {
    pub map: HashMap<String, V>,
}

impl<T: Clone> FieldMap<T> {
    pub fn get(&self, title: &String) -> Option<T> {
        for (substring, value) in &self.map {
            if title.to_ascii_lowercase().contains(substring) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn insert(&mut self, title: &String, somevalue: &Option<T>) {
        match somevalue {
            Some(value) => {
                self.map
                    .insert(title.to_string().to_ascii_lowercase(), value.clone());
            }
            None => (),
        }
    }
}

impl FieldMap<Category> {
    pub fn new() -> Self {
        let map = FieldMap::create_category_map();
        FieldMap { map }
    }

    fn create_category_map() -> HashMap<String, Category> {
        let mut map: HashMap<String, Category> = HashMap::new();
        map.insert("ifood".to_string(), Category::Food);
        map.insert("uber".to_string(), Category::Transportation);
        map.insert("supermercado".to_string(), Category::Supermarket);
        map.insert("restaurante".to_string(), Category::Food);
        map.insert("lanche".to_string(), Category::Food);
        map.insert("viacao".to_string(), Category::Trips);
        map.insert("airbnb".to_string(), Category::Trips);
        map.insert("spotify".to_string(), Category::Personal);
        map.insert("doces".to_string(), Category::Food);
        map.insert("esporte".to_string(), Category::Personal);
        map.insert("culinaria".to_string(), Category::Food);
        map
    }
}

impl FieldMap<String> {
    pub fn new() -> Self {
        let map: HashMap<String, String> = HashMap::new();
        FieldMap { map }
    }
}
