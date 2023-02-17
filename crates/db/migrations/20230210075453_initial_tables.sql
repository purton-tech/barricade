-- migrate:up
CREATE TABLE keypair_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    master_password_hash VARCHAR NOT NULL, 
    protected_symmetric_key VARCHAR NOT NULL, 
    protected_ecdsa_private_key VARCHAR NOT NULL,
    ecdsa_public_key VARCHAR NOT NULL,
    protected_ecdh_private_key VARCHAR NOT NULL,
    ecdh_public_key VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE bcrypt_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    hashed_password VARCHAR NOT NULL, 
    reset_password_selector VARCHAR,
    reset_password_sent_at TIMESTAMP,
    reset_password_validator_hash VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO bcrypt_users(email, hashed_password) VALUES('test1@test1.com', 'aasdsaddasad');
INSERT INTO bcrypt_users(email, hashed_password) VALUES('test2@test1.com', 'aasdsaddasad');
INSERT INTO bcrypt_users(email, hashed_password) VALUES('test3@test1.com', 'aasdsaddasad');

CREATE UNIQUE INDEX ON keypair_users ((lower(email)));
CREATE UNIQUE INDEX ON bcrypt_users ((lower(email)));

-- migrate:down
DROP TABLE keypair_users;
DROP TABLE bcrypt_users;