# sebolinha

Scrapes the [Sebo Cultural](https://www.instagram.com/osebocultural) book catalog to do data analysis and vectorial and semantic search on it

## Run

```sh
cargo run --release
```

The default database is `sebolinha.sqlite3` in the working directory. Override
it with `SEBOLINHA_DB=/path/to/catalog.sqlite3`. Re-running is safe: books are
upserted by their store product id, and categories are replaced with the latest
values from the scraper.

## Run with Docker

```sh
docker build -t sebolinha .
docker run --rm -v sebolinha-data:/data sebolinha
```

SQLite runs inside the scraper process, so no database container is needed
The volume keeps the database, WAL, and shared-memory files together across
container runs

Inspect the latest runs with any SQLite client:

```sql
select * from scrape_runs order by id desc limit 10;
select count(*) from books;
```
