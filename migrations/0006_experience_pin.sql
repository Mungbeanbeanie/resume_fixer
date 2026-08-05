-- An experience the fit loop may never retire, however it scores.
ALTER TABLE experiences ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT FALSE;
