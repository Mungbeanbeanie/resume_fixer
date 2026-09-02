-- A role may print a date range it does not have calendar dates for: an expected
-- graduation ("Spring 2029") has no start and no end the vault could compute it from.
-- `date_override` already wins over the computed range in `render::tex::format_dates`,
-- so the range itself becomes optional and the override stands alone.
ALTER TABLE roles ALTER COLUMN start_date DROP NOT NULL;
