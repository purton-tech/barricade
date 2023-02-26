--: EncryptionKey()

--! get_user_keys : EncryptionKey
SELECT 
    master_password_hash, 
    protected_symmetric_key, 
    protected_ecdsa_private_key,
    ecdsa_public_key,
    protected_ecdh_private_key,
    ecdh_public_key
FROM 
    encryption_keys
WHERE user_id = :user_id;