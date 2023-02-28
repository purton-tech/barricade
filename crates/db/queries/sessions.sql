--: Session(user_id?)

--! get_session : Session
SELECT 
    id,
    session_verifier,
    otp_code_encrypted,
    otp_code_attempts,
    verified,
    user_id,
    email
FROM 
    sessions
WHERE
    id = :id;

--! create_session
INSERT INTO sessions (
    session_verifier, 
    otp_code_encrypted,
    email
)
VALUES(:session_verifier, :otp_code_encrypted, :email) RETURNING id;

--! set_verified_and_increase_attempts
UPDATE 
    sessions 
SET verified = :verified, user_id = :user_id, otp_code_attempts = otp_code_attempts + 1;