-- GPA belongs to the degree, not the school: one experience can hold two of them.
-- Free text so "3.87/4.00" and "3.9 (Major: 4.0)" both print as written.
ALTER TABLE roles ADD COLUMN gpa TEXT;
