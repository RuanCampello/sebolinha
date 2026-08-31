use futures::StreamExt;
use sebolinha::scrape::Scraper;

#[tokio::main]
async fn main() {
    let scraper = Scraper::new();
    let mut stream = Box::pin(scraper.books());
    let (mut ok, mut failed) = (0usize, 0usize);

    while let Some(batch) = stream.next().await {
        match batch {
            Ok(books) => {
                for book in &books {
                    println!(
                        "{:>7}  {:>8}  {:<52.52}  {}",
                        book.id,
                        format!(
                            "R$ {},{:02}",
                            book.price_cents / 100,
                            book.price_cents % 100
                        ),
                        book.title,
                        book.author.as_deref().unwrap_or("—"),
                    );
                }
                ok += books.len();
            }
            Err(error) => {
                eprintln!("batch failed: {error}");
                failed += 1;
            }
        }
    }

    eprintln!("\n{ok} books parsed, {failed} pages failed");
}
