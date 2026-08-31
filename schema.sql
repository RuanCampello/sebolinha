create table books (
  id integer primary key,
  title text not null,
  year integer check (year > 0 and year < 9999),
  description text,
  price_cents integer not null,
  permalink text not null,
  author text, -- books may have multiple authors, so this is the wrong design
  translator text,
  isbn text,
  condition text,
  language text,
  store text,
);

