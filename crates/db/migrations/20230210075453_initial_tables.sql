-- migrate:up
CREATE TABLE keypair_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    master_password_hash VARCHAR NOT NULL, 
    protected_symmetric_key VARCHAR NOT NULL, 
    protected_private_key VARCHAR NOT NULL, 
    public_key VARCHAR NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE bcrypt_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    hashed_password VARCHAR NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX ON keypair_users ((lower(email)));
CREATE UNIQUE INDEX ON bcrypt_users ((lower(email)));

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY, 
    session_uuid UUID NOT NULL DEFAULT gen_random_uuid(), 
    user_id INT NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE keypair_users ADD protected_ecdsa_private_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD ecdsa_public_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD protected_ecdh_private_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD ecdh_public_key VARCHAR NOT NULL;
ALTER TABLE keypair_users DROP public_key;
ALTER TABLE keypair_users DROP protected_private_key;

ALTER TABLE bcrypt_users ADD reset_password_token UUID;
ALTER TABLE bcrypt_users ADD reset_password_sent_at TIMESTAMP;

ALTER TABLE bcrypt_users DROP COLUMN reset_password_token;
ALTER TABLE bcrypt_users ADD COLUMN reset_password_selector VARCHAR;
ALTER TABLE bcrypt_users ADD COLUMN reset_password_validator_hash VARCHAR;

ALTER TABLE sessions ADD otp_code INTEGER NOT NULL DEFAULT (random() * 100000 + 1)::int;
ALTER TABLE sessions ADD otp_code_attempts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD otp_code_confirmed BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE sessions ADD otp_code_sent BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE sessions DROP COLUMN otp_code;
-- Log everyone out
DELETE FROM sessions;
ALTER TABLE sessions ADD otp_code_encrypted VARCHAR NOT NULL;

-- Log everyone out
DELETE FROM sessions;
ALTER TABLE sessions DROP COLUMN session_uuid;
ALTER TABLE sessions ADD session_verifier VARCHAR NOT NULL;

-- migrate:down
DROP TABLE keypair_users;
DROP TABLE bcrypt_users;
DROP TABLE sessions;