-- migrate:up

CREATE TABLE users (
    id SERIAL PRIMARY KEY, 
    email VARCHAR NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO users(email) VALUES('test1@test1.com');
INSERT INTO users(email) VALUES('test2@test1.com');
INSERT INTO users(email) VALUES('test3@test1.com');

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
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE sessions IS 'The users login sessions';
COMMENT ON COLUMN sessions.verified IS 'Once the user enters a correct OTP code this gets set to true.';
COMMENT ON COLUMN sessions.session_verifier IS 'The session is a 32 byte random number stored in their cookie. Only use constant time compares for this column';
COMMENT ON COLUMN sessions.otp_code_encrypted IS 'A 6 digit code that is encrypted here to prevent attackers with read access to the database being able to use it.';
COMMENT ON COLUMN sessions.otp_code_attempts IS 'We count OTP attempts to prevent brute forcing.';
COMMENT ON COLUMN sessions.otp_code_sent IS 'Have we sent the OTP code?';

-- migrate:down
DROP TABLE users;
DROP TABLE sessions;
