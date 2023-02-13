-- migrate:up
CREATE TABLE passcodes (
    id SERIAL PRIMARY KEY, 
    user_id INT NOT NULL, 
    ttl INT NOT NULL, 
    code_hashed VARCHAR NOT NULL,
    try_count INT NOT NULL DEFAULT 0, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE password_credentials (
    id SERIAL PRIMARY KEY, 
    user_id INT NOT NULL,
    password_hashed VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE webauthn_credential_transports (
    id SERIAL PRIMARY KEY, 
    name VARCHAR NOT NULL,
    webauthn_credential_id INT NOT NULL
);

CREATE TABLE webauthn_credentials (
    id SERIAL PRIMARY KEY, 
    user_id INT NOT NULL, 
    public_key VARCHAR NOT NULL,
    attestation_type VARCHAR NOT NULL,
    aaguid UUID NOT NULL,
    sign_count INT NOT NULL DEFAULT 0, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE webauthn_session_data (
    id SERIAL PRIMARY KEY, 
    user_id INT NOT NULL, 
    challenge VARCHAR NOT NULL,
    user_verfication VARCHAR NOT NULL,
    operation VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE webauthn_session_data_allowed_credentials (
    id SERIAL PRIMARY KEY, 
    credential_id VARCHAR NOT NULL,
    webauthn_session_data_id INT NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- migrate:down

DROP TABLE passcodes;
DROP TABLE password_credentials;
DROP TABLE webauthn_credential_transports;
DROP TABLE webauthn_credentials;
DROP TABLE webauthn_session_data;
DROP TABLE webauthn_session_data_allowed_credentials;
DROP TABLE users;
