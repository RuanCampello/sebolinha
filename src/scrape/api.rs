//! Wire types for the store API
//!
//! Deliberately dumb mirrors of the JSON, kept separate from [Book](crate::Book)
//! so a change in the store's response shape lands here and nowhere else
//! `taxonomy` is optional because real records ship it as `null`

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub permalink: String,
    pub sku: String,
    pub short_description: String,
    pub description: String,
    pub prices: Prices,
    pub categories: Vec<Term>,
    pub tags: Vec<Term>,
    pub attributes: Vec<Attribute>,
    pub on_sale: bool,
    pub is_in_stock: bool,
}

#[derive(Debug, Deserialize)]
pub struct Prices {
    pub price: String,
    pub regular_price: String,
}

#[derive(Debug, Deserialize)]
pub struct Term {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct Attribute {
    pub taxonomy: Option<String>,
    pub terms: Vec<Term>,
}
