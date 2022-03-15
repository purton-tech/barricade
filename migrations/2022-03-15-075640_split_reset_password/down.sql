UPDATE TABLE bcrypt_users ADD COLUMN reset_password_token;
UPDATE TABLE bcrypt_users DROP COLUMN reset_password_selector;
UPDATE TABLE bcrypt_users DROP COLUMN reset_password_validsator_hash;
