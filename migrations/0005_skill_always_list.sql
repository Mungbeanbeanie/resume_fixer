-- A skill the user claims but never wrote a bullet about still belongs on the resume.
ALTER TABLE skills ADD COLUMN always_list BOOLEAN NOT NULL DEFAULT FALSE;
