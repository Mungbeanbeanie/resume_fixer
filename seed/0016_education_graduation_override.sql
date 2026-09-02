-- The degree line prints an expected graduation rather than a start--end range, so the
-- calendar dates come out and `date_override` carries the text. Applied by hand:
--   psql resume_fixer -f seed/0016_education_graduation_override.sql
UPDATE roles
   SET start_date    = NULL,
       end_date      = NULL,
       date_override = 'Spring 2029'
 WHERE id = '22222222-2222-4222-8222-000000000001';
