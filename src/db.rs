//! durable SQLite storage for parsed catalog data

use crate::Book;
use rusqlite::{Connection, TransactionBehavior, named_params, params};
use std::path::Path;
use std::time::Duration;

const UPSERT_BOOK: &str = include_str!("../sql/insert.sql");
const SCHEMA: &str = include_str!("../sql/schema.sql");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error("book id {0} does not fit SQLite's signed 64-bit integer")]
    BookIdOutOfRange(u64),
    #[error("{0} does not fit SQLite's signed 64-bit integer")]
    ValueOutOfRange(&'static str),
    #[error("scrape run {0} does not exist or is already finished")]
    RunNotActive(i64),
}

pub type Result<T> = std::result::Result<T, Error>;

/// a single SQLite connection owned by the scraper process
pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self> {
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(
            r#"
            pragma foreign_keys = on;
            pragma journal_mode = wal;
            pragma synchronous = full;
        "#,
        )?;

        connection.execute_batch(SCHEMA)?;

        Ok(Self { connection })
    }

    /// start a run
    /// a process that crashes leaves `finished_at` null
    pub fn start_run(&self) -> Result<i64> {
        self.connection
            .execute("insert into scrape_runs default values", [])?;
        Ok(self.connection.last_insert_rowid())
    }

    /// atomically store one batch of fetched books
    pub fn save_books(&mut self, run_id: i64, books: &[Book]) -> Result<()> {
        let book_count =
            i64::try_from(books.len()).map_err(|_| Error::ValueOutOfRange("book count"))?;
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        {
            let mut upsert_book = transaction.prepare(UPSERT_BOOK)?;
            let mut delete_categories =
                transaction.prepare("delete from book_categories where book_id = ?1")?;
            let mut insert_category = transaction.prepare(
                "insert or ignore into book_categories (book_id, category) values (?1, ?2)",
            )?;

            for book in books {
                let id = i64::try_from(book.id).map_err(|_| Error::BookIdOutOfRange(book.id))?;
                let store = book.store.map(|store| store.as_str());
                upsert_book.execute(named_params! {
                    ":id": id,
                    ":sku": book.sku,
                    ":title": book.title,
                    ":permalink": book.permalink,
                    ":author": book.author,
                    ":publisher": book.publisher,
                    ":translator": book.translator,
                    ":edition": book.edition,
                    ":collection": book.collection,
                    ":pages": book.pages,
                    ":isbn": book.isbn,
                    ":stock_code": book.stock_code,
                    ":condition_note": book.condition_note,
                    ":condition": book.condition,
                    ":year": book.year,
                    ":language": book.language,
                    ":format": book.format,
                    ":price_cents": book.price_cents,
                    ":regular_price_cents": book.regular_price_cents,
                    ":on_sale": book.on_sale,
                    ":in_stock": book.in_stock,
                    ":store": store,
                    ":description": book.description,
                })?;

                delete_categories.execute([id])?;
                for category in &book.categories {
                    insert_category.execute(params![id, category])?;
                }
            }
        }

        let updated = transaction.execute(
            r#"update scrape_runs
               set books_written = books_written + ?2
               where id = ?1 and finished_at is null"#,
            params![run_id, book_count],
        )?;

        if updated != 1 {
            return Err(Error::RunNotActive(run_id));
        }

        Ok(transaction.commit()?)
    }

    pub fn finish_run(&self, run_id: i64) -> Result<()> {
        let updated = self.connection.execute(
            r#"update scrape_runs
               set finished_at = unixepoch()
               where id = ?1 and finished_at is null"#,
            [run_id],
        )?;

        if updated != 1 {
            return Err(Error::RunNotActive(run_id));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::{Book, Store};
    use rusqlite::Connection;

    fn book(id: u64) -> Book {
        Book {
            id,
            sku: format!("sku-{id}"),
            title: "A book".into(),
            permalink: format!("https://example.test/{id}"),
            author: Some("An author".into()),
            publisher: Some("A publisher".into()),
            translator: None,
            edition: Some("2".into()),
            collection: None,
            pages: Some(240),
            isbn: Some("9780000000000".into()),
            stock_code: Some("ABC".into()),
            condition_note: Some("Used".into()),
            condition: Some("Good".into()),
            year: Some(2020),
            language: Some("Português".into()),
            format: Some("Brochura".into()),
            price_cents: 1_250,
            regular_price_cents: 1_500,
            on_sale: true,
            in_stock: true,
            store: Some(Store::Centro),
            categories: vec!["Livros".into(), "Ficção".into()],
            description: Some("Synopsis".into()),
        }
    }

    fn memory_database() -> Database {
        Database::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn stores_and_updates_books_and_categories() {
        let mut database = memory_database();
        let first_run = database.start_run().unwrap();
        database.save_books(first_run, &[book(7)]).unwrap();
        database.finish_run(first_run).unwrap();

        let second_run = database.start_run().unwrap();
        let mut updated = book(7);
        updated.title = "Updated title".into();
        updated.categories = vec!["Livros".into()];
        database.save_books(second_run, &[updated]).unwrap();
        database.finish_run(second_run).unwrap();

        let title: String = database
            .connection
            .query_row("select title from books where id = 7", [], |row| row.get(0))
            .unwrap();
        assert_eq!(title, "Updated title");

        let categories: i64 = database
            .connection
            .query_row(
                "select count(*) from book_categories where book_id = 7",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(categories, 1);
    }

    #[test]
    fn a_failed_page_keeps_previously_committed_pages() {
        let mut database = memory_database();
        let run = database.start_run().unwrap();
        database.save_books(run, &[book(1)]).unwrap();

        let error = database.save_books(run, &[book(2), book(u64::MAX)]);
        assert!(error.is_err());

        let ids: Vec<i32> = database
            .connection
            .prepare("select id from books order by id")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(ids, vec![1]);

        let books_written: i64 = database
            .connection
            .query_row(
                "select books_written from scrape_runs where id = ?1",
                [run],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(books_written, 1);
    }

    #[test]
    fn records_only_run_timing_and_write_count() {
        let database = memory_database();
        let run = database.start_run().unwrap();
        database.finish_run(run).unwrap();

        let (finished, books_written): (i64, i64) = database
            .connection
            .query_row(
                r#"select finished_at is not null, books_written
                   from scrape_runs where id = ?1"#,
                [run],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(finished, 1);
        assert_eq!(books_written, 0);
    }
}
