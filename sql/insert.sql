insert into books (
  id, sku, title, permalink, author, publisher, translator, edition,
  collection, pages, isbn, stock_code, condition_note, condition, year,
  language, format, price_cents, regular_price_cents, on_sale, in_stock,
  store, description
) values (
  :id, :sku, :title, :permalink, :author, :publisher, :translator, :edition,
  :collection, :pages, :isbn, :stock_code, :condition_note, :condition, :year,
  :language, :format, :price_cents, :regular_price_cents, :on_sale, :in_stock,
  :store, :description
)
on conflict(id) do update set
  sku = excluded.sku,
  title = excluded.title,
  permalink = excluded.permalink,
  author = excluded.author,
  publisher = excluded.publisher,
  translator = excluded.translator,
  edition = excluded.edition,
  collection = excluded.collection,
  pages = excluded.pages,
  isbn = excluded.isbn,
  stock_code = excluded.stock_code,
  condition_note = excluded.condition_note,
  condition = excluded.condition,
  year = excluded.year,
  language = excluded.language,
  format = excluded.format,
  price_cents = excluded.price_cents,
  regular_price_cents = excluded.regular_price_cents,
  on_sale = excluded.on_sale,
  in_stock = excluded.in_stock,
  store = excluded.store,
  description = excluded.description
