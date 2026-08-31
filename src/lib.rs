//! Book data from the catalog

pub mod scrape;

use scrape::api::Product;
use scrape::data::Data;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub struct Book {
    pub id: u64,
    pub sku: String,
    pub title: String,
    pub permalink: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub translator: Option<String>,
    pub edition: Option<String>,
    pub collection: Option<String>,
    pub pages: Option<u32>,
    pub isbn: Option<String>,
    pub stock_code: Option<String>,
    pub condition_note: Option<String>,
    pub condition: Option<String>,
    pub year: Option<u16>,
    pub language: Option<String>,
    pub format: Option<String>,
    pub price_cents: i64,
    pub regular_price_cents: i64,
    pub on_sale: bool,
    pub in_stock: bool,
    pub store: Option<Store>,
    pub categories: Vec<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Centro,
    Bancarios,
    Manaira,
}

impl From<Product> for Book {
    fn from(product: Product) -> Self {
        let data = Data::parse(&product.short_description);
        let term = |taxonomy: &str| {
            product
                .attributes
                .iter()
                .find(|a| a.taxonomy.as_deref() == Some(taxonomy))
                .and_then(|a| a.terms.first())
                .map(|t| t.name.clone())
        };

        let condition = term("pa_condicao");
        let format = term("pa_formato");
        let language = term("pa_idioma");
        let year = term("pa_ano").and_then(|y| y.parse().ok());
        let store = product
            .tags
            .iter()
            .find_map(|t| Store::from_str(&t.slug).ok());
        let categories = product.categories.into_iter().map(|c| c.name).collect();
        let description = (!product.description.is_empty()).then_some(product.description);

        Self {
            id: product.id,
            sku: product.sku,
            title: product.name,
            permalink: product.permalink,
            author: data.author,
            publisher: data.publisher,
            translator: data.translator,
            edition: data.edition,
            collection: data.collection,
            pages: data.pages,
            isbn: data.isbn,
            stock_code: data.stock_code,
            condition_note: data.condition_note,
            condition,
            year,
            language,
            format,
            price_cents: product.prices.price.parse().unwrap_or(0),
            regular_price_cents: product.prices.regular_price.parse().unwrap_or(0),
            on_sale: product.on_sale,
            in_stock: product.is_in_stock,
            store,
            categories,
            description,
        }
    }
}

impl FromStr for Store {
    type Err = ();
    fn from_str(slug: &str) -> Result<Self, ()> {
        Ok(match slug {
            "loja-centro" => Self::Centro,
            "loja-bancarios" => Self::Bancarios,
            "loja-manaira" => Self::Manaira,
            _ => return Err(()),
        })
    }
}
