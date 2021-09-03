ALTER TABLE keypair_users ADD protected_ecdsa_private_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD ecdsa_public_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD protected_ecdh_private_key VARCHAR NOT NULL;
ALTER TABLE keypair_users ADD ecdh_public_key VARCHAR NOT NULL;
ALTER TABLE keypair_users DROP public_key;
ALTER TABLE keypair_users DROP protected_private_key;