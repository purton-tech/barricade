--: Session()

--! get_session : Session
SELECT 
    id,
    session_verifier,
    otp_code_encrypted,
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