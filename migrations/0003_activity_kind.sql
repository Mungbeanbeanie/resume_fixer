-- no-transaction
-- Postgres will not let a new enum value be added and used inside one transaction, and sqlx
-- wraps every migration in one unless this directive is present.
ALTER TYPE experience_kind ADD VALUE IF NOT EXISTS 'activity';
