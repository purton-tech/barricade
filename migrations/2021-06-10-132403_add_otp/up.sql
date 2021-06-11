ALTER TABLE sessions ADD otp_code INTEGER NOT NULL DEFAULT (random() * 100000 + 1)::int;
ALTER TABLE sessions ADD otp_code_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD otp_code_confirmed BOOLEAN NOT NULL DEFAULT false;