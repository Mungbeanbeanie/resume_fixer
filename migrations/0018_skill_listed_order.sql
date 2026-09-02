-- The base resume prints the checked skills in the order `skills` comes back in, which was
-- alphabetical. `listed_at` records when a skill was checked so the printed order is the
-- order the user picked them in; unchecking clears it, so re-checking sends a skill to the
-- end, which is how a skill is moved.
ALTER TABLE skills ADD COLUMN listed_at TIMESTAMPTZ;

-- Skills already checked keep the order they print in today. Backdated, so anything checked
-- from here lands after them rather than in the middle.
WITH ordered AS (
    SELECT id, row_number() OVER (ORDER BY name) AS n FROM skills WHERE always_list
)
UPDATE skills s
   SET listed_at = now() - interval '1 day' + (o.n * interval '1 second')
  FROM ordered o
 WHERE s.id = o.id;
