use futures::StreamExt;
use sebolinha::db::Database;
use sebolinha::scrape::Scraper;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_path = std::env::var_os("SEBOLINHA_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("sebolinha.sqlite3"));
    let mut database = Database::open(&database_path)?;
    let run_id = database.start_run()?;

    let scraper = Scraper::new();
    let mut stream = Box::pin(scraper.books().enumerate());
    let (mut ok, mut failed) = (0usize, 0usize);

    while let Some((index, batch)) = stream.next().await {
        let page = index + 1;
        match batch {
            Ok(books) => {
                if let Err(error) = database.save_books(run_id, &books) {
                    eprintln!("page {page} was not stored: {error}");
                    failed += 1;
                    continue;
                }

                ok += books.len();
                eprintln!("page {page} committed ({} books)", books.len());
            }
            Err(error) => {
                eprintln!("page {page} failed: {error}");
                failed += 1;
            }
        }
    }

    database.finish_run(run_id)?;
    eprintln!(
        "\nrun {run_id}: {ok} books stored, {failed} pages failed; database: {}",
        database_path.display()
    );
    Ok(())
}
