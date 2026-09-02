-- What a project's `url` reads as on the page. The date slot is narrow, so a full repo
-- path crowds the right margin; "GitHub" or "Devpost" in the author's own words fits.
-- Blank falls back to the shortened URL — see `render::tex::link_label`.
ALTER TABLE experiences ADD COLUMN link_text TEXT;
