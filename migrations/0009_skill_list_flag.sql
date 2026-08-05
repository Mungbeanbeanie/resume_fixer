-- `always_list` is the only switch deciding whether a skill prints. A tagged skill used to
-- print regardless of the flag, so every skill the vault already tags is turned on here to
-- keep existing resumes identical.
UPDATE skills SET always_list = TRUE
WHERE id IN (SELECT skill_id FROM bullet_skills)
   OR id IN (SELECT skill_id FROM experience_skills);
