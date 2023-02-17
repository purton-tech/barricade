--: Session()

--! get_sessions : Session
SELECT 
    id
FROM sessions;

--! create_session
INSERT INTO sessions (
    user_id, 
    session_verifier, 
    otp_code_encrypted
)
VALUES(:user_id, :session_verifier, :otp_code_encrypted) RETURNING id;