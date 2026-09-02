create table if not exists scrape_runs (
  id integer primary key,
  started_at integer not null default (unixepoch()),
  finished_at integer,
  books_written integer not null default 0 check (books_written >= 0)
) strict;

create table if not exists books (
  id integer primary key check (id >= 0),
  sku text not null,
  title text not null,
  permalink text not null,
  author text,
  publisher text,
  translator text,
  edition text,
  collection text,
  pages integer check (pages is null or pages >= 0),
  isbn text,
  stock_code text,
  condition_note text,
  condition text,
  year integer check (year is null or (year > 0 and year < 9999)),
  language text,
  format text,
  price_cents integer not null,
  regular_price_cents integer not null,
  on_sale integer not null check (on_sale in (0, 1)),
  in_stock integer not null check (in_stock in (0, 1)),
  store text check (store is null or store in ('centro', 'bancarios', 'manaira')),
  description text
) strict;

create table if not exists book_categories (
  book_id integer not null references books(id) on delete cascade,
  category text not null,
  primary key (book_id, category)
) strict, without rowid;

create index if not exists book_categories_by_category
  on book_categories(category, book_id);
