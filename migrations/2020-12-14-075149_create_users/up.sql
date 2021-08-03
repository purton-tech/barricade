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

-- Manage the updated_at column
SELECT diesel_manage_updated_at('keypair_users');
SELECT diesel_manage_updated_at('bcrypt_users');