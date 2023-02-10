CREATE TABLE sessions (
    id SERIAL PRIMARY KEY, 
    session_uuid UUID NOT NULL DEFAULT gen_random_uuid(), 
    user_id INT NOT NULL, 
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);