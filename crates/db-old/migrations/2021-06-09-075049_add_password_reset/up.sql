ALTER TABLE bcrypt_users ADD reset_password_token UUID;
ALTER TABLE bcrypt_users ADD reset_password_sent_at TIMESTAMP;