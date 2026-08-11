-- The pipeline a student actually walks: an online assessment arrives, gets finished, and
-- then the interviews are numbered — "interview" alone loses which round you are sitting on,
-- which is the one thing worth knowing when three companies are mid-loop at once.
--
-- Rebuilt rather than extended: ALTER TYPE can add a value but never drop one, and a status
-- dropdown offering both 'interview' and 'interview_1' asks a question with no right answer.
-- Existing rows land on round one.
CREATE TYPE application_status_new AS ENUM
    ('saved', 'applied', 'oa_received', 'oa_completed',
     'interview_1', 'interview_2', 'interview_3', 'offer', 'rejected', 'withdrawn');

ALTER TABLE applications
    ALTER COLUMN status DROP DEFAULT,
    ALTER COLUMN status TYPE application_status_new
        USING (CASE WHEN status = 'interview' THEN 'interview_1' ELSE status::text END)
              ::application_status_new,
    ALTER COLUMN status SET DEFAULT 'saved';

ALTER TABLE application_status_history
    ALTER COLUMN status TYPE application_status_new
        USING (CASE WHEN status = 'interview' THEN 'interview_1' ELSE status::text END)
              ::application_status_new;

DROP TYPE application_status;
ALTER TYPE application_status_new RENAME TO application_status;
