ALTER TABLE bcrypt_users DROP COLUMN reset_password_token;
ALTER TABLE bcrypt_users ADD COLUMN reset_password_selector VARCHAR;
ALTER TABLE bcrypt_users ADD COLUMN reset_password_validator_hash VARCHAR;
