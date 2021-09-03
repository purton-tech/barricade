ALTER TABLE keypair_users DROP protected_ecdsa_private_key;
ALTER TABLE keypair_users DROP ecdsa_public_key;
ALTER TABLE keypair_users DROP protected_ecdh_private_key;
ALTER TABLE keypair_users DROP ecdh_public_key;
ALTER TABLE keypair_users ADD public_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD protected_private_key VARCHAR NOT NULL;