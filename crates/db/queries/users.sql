--: User()

--! get_users : User
SELECT 
    id, 
    email
FROM users;

--! find_by_email : User
SELECT 
    id,
    email 
FROM 
    users 
WHERE email = :email;

--! create_user
INSERT INTO
    users
(email)
VALUES(:email)
RETURNING id;