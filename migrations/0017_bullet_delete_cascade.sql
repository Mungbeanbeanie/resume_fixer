-- A bullet any saved resume had printed could not be deleted from the vault at all: the
-- provenance foreign key held ON DELETE RESTRICT, and work bullets are the ones that print,
-- so the vault silently refused exactly the deletes the user meant.
--
-- The record of what was sent is `resumes.tex_source` and the stored PDF, and neither moves
-- when a bullet goes. The provenance row cannot outlive the bullet it points at, so it
-- follows it out.
ALTER TABLE resume_bullets
    DROP CONSTRAINT resume_bullets_bullet_id_fkey,
    ADD CONSTRAINT resume_bullets_bullet_id_fkey
        FOREIGN KEY (bullet_id) REFERENCES bullets(id) ON DELETE CASCADE;
