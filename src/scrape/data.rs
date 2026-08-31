//! Parser for the `short_description` HTML fragment
//!
//! The API returns author, publisher, ISBN and friends only as markup, so this
//! is the one place left that touches HTML

use scraper::{Html, Selector};
use std::sync::LazyLock;

static LI: LazyLock<Selector> = LazyLock::new(|| Selector::parse("li").unwrap());
static SPAN: LazyLock<Selector> = LazyLock::new(|| Selector::parse("span").unwrap());

#[derive(Debug, Default, PartialEq)]
pub struct Data {
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub translator: Option<String>,
    pub edition: Option<String>,
    pub collection: Option<String>,
    pub pages: Option<u32>,
    pub isbn: Option<String>,
    pub stock_code: Option<String>,
    pub condition_note: Option<String>,
}

impl Data {
    pub fn parse(fragment: &str) -> Self {
        let html = Html::parse_fragment(fragment);
        let mut data = Self::default();

        for item in html.select(&LI) {
            let Some(label) = item.select(&SPAN).next() else {
                continue;
            };

            let label: String = label.text().collect();
            let value: String = item
                .children()
                .filter_map(|node| node.value().as_text().map(|t| t.to_string()))
                .collect();

            let value = value.trim();
            if value.is_empty() {
                continue;
            }

            match label.trim() {
                "Autor:" => data.author = Some(value.into()),
                "Editora:" => data.publisher = Some(value.into()),
                "Tradução:" => data.translator = Some(value.into()),
                "Edição:" => data.edition = Some(value.into()),
                "Coleção:" => data.collection = Some(value.into()),
                "Qtd. Páginas:" => data.pages = value.parse().ok(),
                "Isbn:" => data.isbn = Some(value.into()),
                "Código Estoque:" => data.stock_code = Some(value.into()),
                "Estado de Conservação:" => data.condition_note = Some(value.into()),
                _ => {}
            }
        }
        data
    }
}

/// turn a piece of provided rendered html into plain text
pub(crate) fn plain_text(rendered: &str) -> String {
    Html::parse_fragment(rendered)
        .root_element()
        .text()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::plain_text;

    #[test]
    fn decodes_html_character_references() {
        assert_eq!(
            plain_text("Que E Energia Nuclear, O &#8211; Colecao &amp; Guia"),
            "Que E Energia Nuclear, O – Colecao & Guia"
        );
    }

    #[test]
    fn leaves_unicode_text_unchanged() {
        assert_eq!(
            plain_text("O príncipe – Maquiavel"),
            "O príncipe – Maquiavel"
        );
    }
}
