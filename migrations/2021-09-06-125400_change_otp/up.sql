ALTER TABLE sessions DROP COLUMN otp_code;
-- Log everyone out
DELETE FROM sessions;
ALTER TABLE sessions ADD otp_code_encrypted VARCHAR NOT NULL;