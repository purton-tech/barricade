ALTER TABLE bcrypt_users ADD COLUMN reset_password_token UUID;
ALTER TABLE bcrypt_users DROP COLUMN reset_password_selector;
ALTER TABLE bcrypt_users DROP COLUMN reset_password_validator_hash;
