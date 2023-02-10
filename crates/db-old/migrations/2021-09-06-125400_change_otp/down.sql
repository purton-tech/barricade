ALTER TABLE sessions DROP otp_code_encrypted;
ALTER TABLE sessions ADD otp_code INTEGER NOT NULL DEFAULT (random() * 100000 + 1)::int;