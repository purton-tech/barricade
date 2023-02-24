use crate::errors::CustomError;
use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead, Payload};
use aes_gcm::AeadInPlace;
use aes_gcm::Aes256Gcm;
use rand::RngCore;
use rand::rngs::OsRng; // Or `Aes128Gcm`

pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;

/// Encrypts the plain text with authenticated encryption providing
/// confidentiality, integrity, and authenticity.
pub fn encrypt(
    plain_text: &str,
    aad: &str,
    secret_key: &[u8],
) -> Result<String, CustomError> {
    // Create a vec to hold the [nonce | cookie value | tag].
    let val = plain_text.as_bytes();
    let mut data = vec![0; NONCE_LEN + val.len() + TAG_LEN];

    // Split data into three: nonce, input/output, tag. Copy input.
    let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
    let (in_out, tag) = in_out.split_at_mut(val.len());
    in_out.copy_from_slice(val);

    // Fill nonce piece with random data.
    OsRng.fill_bytes(nonce);
    let nonce = GenericArray::clone_from_slice(nonce);

    // Perform the actual sealing operation, using aad as
    // associated data to prevent value swapping.
    let aad = aad.as_bytes();
    let aead = Aes256Gcm::new(GenericArray::from_slice(secret_key));
    let aad_tag = aead
        .encrypt_in_place_detached(&nonce, aad, in_out)
        .expect("encryption failure!");

    // Copy the tag into the tag piece.
    tag.copy_from_slice(&aad_tag);

    // Base64 encode [nonce | encrypted value | tag].
    Ok(hex::encode(&data))
}

/// Given a encrypted value `str` and an aad, where the nonce is
/// prepended to the original value and then both are Base64 encoded,
/// verifies and decrypts the sealed value and returns it. If there's a
/// problem, returns an `Err` with a string describing the issue.
pub fn decrypt(cipher: &str, aad: &str, secret_key: &[u8]) -> Result<String, CustomError> {
    let data = hex::decode(cipher)
        .map_err(|_| CustomError::FaultySetup("bad base64 value".into()))?;
    if data.len() <= NONCE_LEN {
        return Err(CustomError::FaultySetup(
            "length of decoded data is <= NONCE_LEN".into(),
        ));
    }

    let (nonce, cipher) = data.split_at(NONCE_LEN);
    let payload = Payload {
        msg: cipher,
        aad: aad.as_bytes(),
    };

    let aead = Aes256Gcm::new(GenericArray::from_slice(secret_key));
    let decrypted = aead
        .decrypt(GenericArray::from_slice(nonce), payload)
        .map_err(|e| CustomError::FaultySetup(e.to_string()))?;

    let decrypted =
        String::from_utf8(decrypted).map_err(|e| CustomError::FaultySetup(e.to_string()))?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[tokio::test]
    async fn test_encryption() {
        let random_bytes = rand::thread_rng().gen::<[u8; 32]>();

        let cipher_text = encrypt("Hello World", "AAD", &random_bytes).unwrap();

        let plain_text = decrypt(&cipher_text, "AAD", &random_bytes).unwrap();

        assert_eq!(plain_text, "Hello World");
    }
}
