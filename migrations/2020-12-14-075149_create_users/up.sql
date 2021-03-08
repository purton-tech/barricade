-- X25519 secret key and X25519 public key.
CREATE TABLE keypair_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    encrypted_private_key VARCHAR NOT NULL, 
    public_key VARCHAR NOT NULL, 
    init_vector VARCHAR NOT NULL, 
    blind_index VARCHAR NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE bip38_users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE, 
    blind_index VARCHAR NOT NULL, 
    bip38 VARCHAR NOT NULL, 
    xpub VARCHAR NOT NULL, 
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
CREATE UNIQUE INDEX ON bip38_users ((lower(email)));
CREATE UNIQUE INDEX ON bcrypt_users ((lower(email)));

-- Manage the updated_at column
SELECT diesel_manage_updated_at('keypair_users');
SELECT diesel_manage_updated_at('bip38_users');
SELECT diesel_manage_updated_at('bcrypt_users');