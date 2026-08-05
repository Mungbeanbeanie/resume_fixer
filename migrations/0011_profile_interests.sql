-- Interests, as one comma-separated line on the profile.
--
-- A free-text line rather than rows: nothing scores, ranks, or retrieves against interests,
-- so a table would buy only the ability to reorder what the user can already reorder by
-- typing. Only the Simplify layout names the variable, so Jake's ignores it.
ALTER TABLE profile ADD COLUMN interests TEXT;
