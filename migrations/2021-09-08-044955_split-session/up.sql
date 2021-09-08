-- Log everyone out
DELETE FROM sessions;
ALTER TABLE sessions DROP COLUMN session_uuid;
ALTER TABLE sessions ADD session_verifier VARCHAR NOT NULL;