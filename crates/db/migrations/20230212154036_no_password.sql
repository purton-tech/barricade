-- migrate:up

CREATE TABLE users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE
);

CREATE UNIQUE INDEX ON users ((lower(email)));

CREATE TABLE sessions (
    id SERIAL PRIMARY KEY, 
    session_verifier VARCHAR NOT NULL, 
    -- The user_id is set once we have passed OTP.
    -- We don't set it before then or we get timing issues
    user_id INT, 
    -- Until we pass OTP we need the email to lookup a user
    email VARCHAR NOT NULL, 
    -- We were able to confirm this session out of band i.e. OTP
    verified BOOLEAN NOT NULL DEFAULT false,
    otp_code_encrypted VARCHAR NOT NULL,
    otp_code_attempts INTEGER NOT NULL DEFAULT 0,
    otp_code_sent BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

COMMENT ON TABLE sessions IS 'The users login sessions';
COMMENT ON COLUMN sessions.verified IS 'Once the user enters a correct OTP code this gets set to true.';
COMMENT ON COLUMN sessions.session_verifier IS 'The session is a 32 byte random number stored in their cookie. Only use constant time compares for this column';
COMMENT ON COLUMN sessions.otp_code_encrypted IS 'A 6 digit code that is encrypted here to prevent attackers with read access to the database being able to use it.';
COMMENT ON COLUMN sessions.otp_code_attempts IS 'We count OTP attempts to prevent brute forcing.';
COMMENT ON COLUMN sessions.otp_code_sent IS 'Have we sent the OTP code?';

CREATE TABLE encryption_keys (
    id SERIAL PRIMARY KEY, 
    user_id INT NOT NULL, 
    master_password_hash VARCHAR NOT NULL, 
    protected_symmetric_key VARCHAR NOT NULL, 
    protected_ecdsa_private_key VARCHAR NOT NULL,
    ecdsa_public_key VARCHAR NOT NULL,
    protected_ecdh_private_key VARCHAR NOT NULL,
    ecdh_public_key VARCHAR NOT NULL,
    UNIQUE(user_id),
    FOREIGN KEY(user_id) REFERENCES users(id)
);

COMMENT ON TABLE encryption_keys IS 'The users encryption keys generated during registration or sign in.';
COMMENT ON COLUMN encryption_keys.master_password_hash IS 'Hash of the users master password for authentication';
COMMENT ON COLUMN encryption_keys.protected_symmetric_key IS 'Wrapped AES-GCM key for symmetric encryption and decryption';
COMMENT ON COLUMN encryption_keys.protected_ecdsa_private_key IS 'Wrapped ECDSA key for signing';
COMMENT ON COLUMN encryption_keys.ecdsa_public_key IS 'Public ECDSA key for signature verification';
COMMENT ON COLUMN encryption_keys.protected_ecdh_private_key IS 'Wrapped ECDH key for public key encryption and key negotiation';
COMMENT ON COLUMN encryption_keys.ecdh_public_key IS 'Public ECDH key for public key encryption and key negotiation';

-- migrate:down
DROP TABLE sessions;
DROP TABLE encryption_keys;
DROP TABLE users;
