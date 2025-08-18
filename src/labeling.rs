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
        map.push(("iFood".to_string(), Category::Food));
        map
    }

    fn compile_regex(&self) {
        unimplemented!();
    }

    fn classify_title(&self, title: String) {
        unimplemented!();
    }
}
