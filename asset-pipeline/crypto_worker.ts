import { Jobs, CreateMasterKeyRequest, CreateMasterKeyResult,
    DecryptMasterKeyRequest, DecryptMasterKeyResult,
    ByteData, Cipher, HashMasterPasswordRequest,
    HashMasterPasswordResult  } from './crypto_types'
import { openDB } from 'idb';

// We alias self to ctx and give it our newly created type
const ctx: Worker = self as any

ctx.onmessage = async e => {

    const data = e.data

    switch (data.cmd) {
        case Jobs[Jobs.HashMasterPassword]: {
            const mkr: HashMasterPasswordRequest = data.request

            const masterPassword = fromUtf8(mkr.masterPassword)
            const email = fromUtf8(mkr.email)

            const masterCryptoKey = await pbkdf2(masterPassword, email, mkr.pbkdf2Iterations, 256)
            const masterKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', masterCryptoKey))
            const masterKeyHash = await pbkdf2(masterKeyData.arr, masterPassword, 1, 256)
            const masterKeyHashData = new ByteData(await self.crypto.subtle.exportKey('raw', masterKeyHash))
            
            let result: HashMasterPasswordResult = {
                masterPasswordHash: masterKeyHashData.b64
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.CreateMasterKey],
                status: 'done',
                response: result
            })

            break
        }
        case Jobs[Jobs.CreateMasterKey]: {

            // Generate a random private key. Stretch the users password and encrypt that key.
            // Produces the encrypted key, the nonce used and a blind index (bloom filter)

            const mkr: CreateMasterKeyRequest = data.request

            const masterPassword = fromUtf8(mkr.masterPassword)
            const email = fromUtf8(mkr.email)
            const masterCryptoKey = await pbkdf2(masterPassword, email, mkr.pbkdf2Iterations, 256)
            const masterKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', masterCryptoKey))
            const masterKeyHash = await pbkdf2(masterKeyData.arr, masterPassword, 1, 256)
            const masterKeyHashData = new ByteData(await self.crypto.subtle.exportKey('raw', masterKeyHash))

            const symKey = await self.crypto.subtle.generateKey({
                name: 'AES-GCM',
                length: 128
            },
            true,
            ['decrypt', 'encrypt'])
            const symKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', symKey))
            const protectedSymKey = await aesEncrypt(symKeyData.arr, masterCryptoKey);

            const keyPair = await generateRsaKeyPair();
            const publicKey = keyPair.publicKey;
            const privateKey = keyPair.privateKey;
            const protectedPrivateKey = await aesEncrypt(privateKey.arr, masterCryptoKey);

            const masterResponse: CreateMasterKeyResult = {
                masterPasswordHash: masterKeyHashData.b64,
                protectedSymmetricKey: protectedSymKey.string,
                protectedPrivateKey: protectedPrivateKey.string,
                publicKey: publicKey.b64
            }

            // Now store it in the vault
            try {
                const keyOptions = {
                    name: 'AES-GCM'
                };
                const key: CryptoKey = await self.crypto.subtle.importKey(
                    'raw', masterKeyData.arr.buffer, keyOptions, false, ['decrypt', 'encrypt']);
                const db = await openIndexedDB()
                await db.put('keyval', key, 'master_key');
                db.close()
            } catch (e) {
                console.log(e)
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.CreateMasterKey],
                status: 'done',
                response: masterResponse
            })

            break
        }
        case Jobs[Jobs.DecryptMasterKey]: {
            const db = await openIndexedDB()
            const masterKey = await db.get('keyval', 'master_key') as CryptoKey;

            console.log(masterKey)

            const decryptKeyRequest: DecryptMasterKeyRequest = data.request

            const semKeyCipher = Cipher.fromString(decryptKeyRequest.protectedSymmetricKey);

            console.log(semKeyCipher)

            const decryptedSymmetric = await aesDecrypt(semKeyCipher, masterKey)

            console.log(decryptedSymmetric)

            const keyOptions = {
                name: 'AES-GCM'
            };
            const unprotectedSymKey: CryptoKey = await self.crypto.subtle.importKey(
                'raw', decryptedSymmetric.arr.buffer, keyOptions, false, ['decrypt', 'encrypt']);

            console.log(unprotectedSymKey)

            const privKeyCipher = Cipher.fromString(decryptKeyRequest.protectedPrivateKey)

            console.log(privKeyCipher)

            let decryptedPrivateKey = await aesDecrypt(privKeyCipher, masterKey);

            console.log(decryptedPrivateKey)

            const rsaOptions = {
                name: 'RSA-OAEP',
                modulusLength: 2048,
                publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537
                hash: { name: 'SHA-1' }
            };
            // RSA private keys can only decrypt
            const unprotectedPrivateKey: CryptoKey = await self.crypto.subtle.importKey(
                'pkcs8', decryptedPrivateKey.arr.buffer, rsaOptions, false, ['decrypt'])

            const publicKey: CryptoKey = await self.crypto.subtle.importKey(
                'spki', ByteData.fromB64(decryptKeyRequest.publicKey).arr.buffer, rsaOptions, 
                true, ['encrypt'])

            try {
                const db = await openIndexedDB()
                await db.put('keyval', unprotectedSymKey, 'unprotected_symmetric_key')
                await db.put('keyval', unprotectedPrivateKey, 'unprotected_private_key')
                await db.put('keyval', publicKey, 'public_key')
                db.close()
            } catch (e) {
                console.log(e)
            }

            ctx.postMessage({
                cmd: 'master-key',
                status: 'done',
                response: {}
            })
            break
        }
        default:
            ctx.postMessage('Unknown command: ' + data.cmd);
    }
}

async function openIndexedDB() {
    return await openDB('Vault', 1, {
        upgrade(db) {
            db.createObjectStore("keyval");
        },
    });
}

// Helpers

function fromUtf8(str) {
    const strUtf8 = unescape(encodeURIComponent(str));
    const bytes = new Uint8Array(strUtf8.length);
    for (let i = 0; i < strUtf8.length; i++) {
        bytes[i] = strUtf8.charCodeAt(i);
    }
    return bytes.buffer;
}

// Crypto

async function pbkdf2(password : ArrayBuffer, salt : ArrayBuffer, 
    iterations: number, length: number) : Promise<CryptoKey> {
    const importAlg = {
        name: 'PBKDF2'
    };

    const deriveAlg = {
        name: 'PBKDF2',
        salt: salt,
        iterations: iterations,
        hash: { name: 'SHA-256' }
    };

    const aesOptions = {
        name: 'AES-GCM',
        length: length
    };

    try {
        const importedKey = await self.crypto.subtle.importKey(
            'raw', password, importAlg, false, ['deriveKey']);
        const derivedKey = await self.crypto.subtle.deriveKey(
            deriveAlg, importedKey, aesOptions, true, ['encrypt', 'decrypt']);
        return derivedKey;
    } catch (err) {
        console.log(err);
    }
}

async function aesEncrypt(data : Uint8Array, key: CryptoKey) : Promise<Cipher> {
    const keyOptions = {
        name: 'AES-GCM'
    };

    const encOptions = {
        name: 'AES-GCM',
        iv: new Uint8Array(16)
    };
    self.crypto.getRandomValues(encOptions.iv);
    const ivData = new ByteData(encOptions.iv.buffer);
    const cipher = new ByteData(await self.crypto.subtle.encrypt(encOptions, key, data))

    return new Cipher(ivData, cipher)
}

async function aesDecrypt(cipher: Cipher, key: CryptoKey) { 
    const keyOptions = {
        name: 'AES-GCM'
    };

    const decOptions = {
        name: 'AES-GCM',
        iv: cipher.iv.arr.buffer
    };

    return new ByteData(await self.crypto.subtle.decrypt(decOptions, key, cipher.ct.arr.buffer));
}

async function generateRsaKeyPair() {
    const rsaOptions = {
        name: 'RSA-OAEP',
        modulusLength: 2048,
        publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537
        hash: { name: 'SHA-1' }
    };

    try {
        const keyPair = await self.crypto.subtle.generateKey(rsaOptions, true, ['encrypt', 'decrypt']);
        const publicKey = new ByteData(await self.crypto.subtle.exportKey('spki', keyPair.publicKey));
        const privateKey = new ByteData(await self.crypto.subtle.exportKey('pkcs8', keyPair.privateKey));
        return {
            publicKey: publicKey,
            privateKey: privateKey
        };
    } catch (err) {
        console.error(err);
    }
}
