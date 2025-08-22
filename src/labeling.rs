use chrono::Datelike;
use itertools::multizip;
use polars::prelude::*;
use rusqlite::Result;
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

use crate::models::Category;

pub struct CategoryMap {
    pub map: Vec<(String, Category)>,
}

impl CategoryMap {
    pub fn new() -> Result<Self> {
        let map = CategoryMap::create_category_map();
        Ok(CategoryMap { map })
    }

    fn create_category_map() -> Vec<(String, Category)> {
        let mut map: Vec<(String, Category)> = Vec::new();
        map.push(("ifood".to_string(), Category::Food));
        map.push(("uber".to_string(), Category::Transportation));
        map.push(("supermercado".to_string(), Category::Supermarket));
        map.push(("restaurante".to_string(), Category::Food));
        map.push(("lanche".to_string(), Category::Food));
        map.push(("viacao".to_string(), Category::Trips));
        map.push(("airbnb".to_string(), Category::Trips));
        map.push(("spotify".to_string(), Category::Personal));
        map.push(("doces".to_string(), Category::Food));
        map.push(("esporte".to_string(), Category::Personal));
        map.push(("culinaria".to_string(), Category::Food));
        map
    }

    pub fn classify_title(&self, title: &String) -> Option<Category> {
        for (substring, category) in &self.map {
            if title.to_ascii_lowercase().contains(substring) {
                return Some(category.clone());
            }
        }
        None
    }
}
