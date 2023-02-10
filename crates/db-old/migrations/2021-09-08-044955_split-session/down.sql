ALTER TABLE sessions DROP session_verifier;
ALTER TABLE sessions ADD session_uuid UUID NOT NULL DEFAULT gen_random_uuid();