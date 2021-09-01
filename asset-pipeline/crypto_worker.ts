import {
    Jobs, InitialiseVaultWithNewKeysRequest, InitialiseVaultWithNewKeysResult,
    ImportProtectedKeysIntoVaultRequest,
    ByteData, Cipher
} from './crypto_types'
import { openDB } from 'idb';

// We alias self to ctx and give it our newly created type
const ctx: Worker = self as any

const PBKDF2_ITERATIONS = 100_000

const ECDSA_OPTIONS = {
    name: "ECDSA",
    namedCurve: "P-384"
};

const ECDH_OPTIONS = {
    name: "ECDH",
    namedCurve: "P-384"
};

const AES_OPTIONS = {
    name: "AES-GCM",
    length: 256
};

ctx.onmessage = async e => {

    const data = e.data

    switch (data.cmd) {
        case Jobs[Jobs.UnlockVaultWithMasterPassword]: {
            const mkr: InitialiseVaultWithNewKeysRequest = data.request

            const [masterCryptoKey, masterKeyData, masterKeyHash, masterKeyHashData] = 
                await generateMasterKey(mkr.email, mkr.masterPassword, PBKDF2_ITERATIONS)

            // Now store it in the vault
            await storeMasterKey(masterKeyData)

            // Post the auth token back to the caller
            ctx.postMessage({
                status: 'done',
                response: masterKeyHashData.b64
            })

            break
        }
        case Jobs[Jobs.InitialiseVaultWithNewKeys]: {

            // Generate a random private key. Stretch the users password and encrypt that key.
            // Produces the encrypted key, the nonce used and a blind index (bloom filter)

            const mkr: InitialiseVaultWithNewKeysRequest = data.request

            const [masterCryptoKey, masterKeyData, masterKeyHash, masterKeyHashData] = 
                await generateMasterKey(mkr.email, mkr.masterPassword, mkr.pbkdf2Iterations)

            const symKey = await self.crypto.subtle.generateKey(
                AES_OPTIONS,
                true,
                ['decrypt', 'encrypt'])
            const symKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', symKey))
            const protectedSymKey = await aesEncrypt(symKeyData.arr, masterCryptoKey);

            // ECDSA
            const keyPair = await generateECDSAKeyPair();
            const publicECDSAKey = keyPair.publicKey;
            const protectedECDSAPrivateKey = await aesEncrypt(keyPair.privateKey.arr, masterCryptoKey);

            // ECDH
            const keyPairDH = await generateECDHKeyPair();
            const publicECDHKey = keyPairDH.publicKey;
            const protectedECDHPrivateKey = await aesEncrypt(keyPairDH.privateKey.arr, masterCryptoKey);

            const masterResponse: InitialiseVaultWithNewKeysResult = {
                masterPasswordHash: masterKeyHashData.b64,
                protectedSymmetricKey: protectedSymKey.string,
                protectedECDSAPrivateKey: protectedECDSAPrivateKey.string,
                publicECDSAKey: publicECDSAKey.b64,
                protectedECDHPrivateKey: protectedECDHPrivateKey.string,
                publicECDHKey: publicECDHKey.b64
            }

            // Now store it in the vault
            await storeMasterKey(masterKeyData)

            ctx.postMessage({
                status: 'done',
                response: masterResponse
            })

            break
        }
        case Jobs[Jobs.ImportProtectedKeysIntoVault]: {
            const db = await openIndexedDB()
            const masterKey = await db.get('keyval', 'master_key') as CryptoKey;

            const decryptKeyRequest: ImportProtectedKeysIntoVaultRequest = data.request

            const semKeyCipher = Cipher.fromString(decryptKeyRequest.protectedSymmetricKey);

            const decryptedSymmetric = await aesDecrypt(semKeyCipher, masterKey)

            const unprotectedSymKey: CryptoKey = await self.crypto.subtle.importKey(
                'raw', decryptedSymmetric.arr.buffer, AES_OPTIONS, false, ['decrypt', 'encrypt']);

            // ECDSA
            const privKeyCipher = Cipher.fromString(decryptKeyRequest.protectedECDSAPrivateKey)
            let decryptedECDSAPrivateKey = await aesDecrypt(privKeyCipher, masterKey);
            const unprotectedECDSAPrivateKey: CryptoKey = await self.crypto.subtle.importKey(
                'pkcs8', decryptedECDSAPrivateKey.arr.buffer, ECDSA_OPTIONS, false, ['sign'])
            const publicECDSAKey: CryptoKey = await self.crypto.subtle.importKey(
                'spki', ByteData.fromB64(decryptKeyRequest.publicECDSAKey).arr.buffer, 
                ECDSA_OPTIONS,
                true, ['verify'])

            // ECDH
            const privKeyCipherDH = Cipher.fromString(decryptKeyRequest.protectedECDHPrivateKey)
            let decryptedECDHPrivateKey = await aesDecrypt(privKeyCipherDH, masterKey);
            console.log('about to import')
            const unprotectedECDHPrivateKey: CryptoKey = await self.crypto.subtle.importKey(
                'pkcs8', decryptedECDHPrivateKey.arr.buffer, ECDH_OPTIONS, false, 
                ['deriveKey', 'deriveBits'])
            console.log('about to import public')
            const publicECDHKey: CryptoKey = await self.crypto.subtle.importKey(
                'spki', ByteData.fromB64(decryptKeyRequest.publicECDSAKey).arr.buffer, 
                ECDH_OPTIONS,
                true, [])
            console.log('it worked')

            try {
                const db = await openIndexedDB()
                await db.put('keyval', unprotectedSymKey, 'unprotected_symmetric_key')
                await db.put('keyval', unprotectedECDSAPrivateKey, 'unprotected_ecdsa_private_key')
                await db.put('keyval', publicECDSAKey, 'ecdsa_public_key')
                await db.put('keyval', unprotectedECDHPrivateKey, 'unprotected_ecdh_private_key')
                await db.put('keyval', publicECDHKey, 'ecdh_public_key')
                db.close()
            } catch (e) {
                console.log(e)
            }

            ctx.postMessage({
                status: 'done',
                response: {}
            })
            break
        }
        default:
            ctx.postMessage('Unknown command: ' + data.cmd);
    }
}

async function storeMasterKey(masterKey: ByteData) {

    // Now store it in the vault
    try {
        const key: CryptoKey = await self.crypto.subtle.importKey(
            'raw', masterKey.arr.buffer, AES_OPTIONS, false, ['decrypt', 'encrypt']);
        const db = await openIndexedDB()
        await db.put('keyval', key, 'master_key');
        db.close()
    } catch (e) {
        console.log(e)
    }
}

async function generateMasterKey(username: string, password: string, iterations: number):
    Promise<[CryptoKey, ByteData, CryptoKey, ByteData]> {


    const masterPassword = fromUtf8(password)
    const email = fromUtf8(username)

    const masterCryptoKey = await pbkdf2(masterPassword, email, iterations, 256)
    const masterKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', masterCryptoKey))
    const masterKeyHash = await pbkdf2(masterKeyData.arr, masterPassword, 1, 256)
    const masterKeyHashData = new ByteData(await self.crypto.subtle.exportKey('raw', masterKeyHash))

    return [masterCryptoKey, masterKeyData, masterKeyHash, masterKeyHashData]
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

async function pbkdf2(password: ArrayBuffer, salt: ArrayBuffer,
    iterations: number, length: number): Promise<CryptoKey> {
    const importAlg = {
        name: 'PBKDF2'
    };

    const deriveAlg = {
        name: 'PBKDF2',
        salt: salt,
        iterations: iterations,
        hash: { name: 'SHA-256' }
    };

    try {
        const importedKey = await self.crypto.subtle.importKey(
            'raw', password, importAlg, false, ['deriveKey']);
        const derivedKey = await self.crypto.subtle.deriveKey(
            deriveAlg, importedKey, AES_OPTIONS, true, ['encrypt', 'decrypt']);
        return derivedKey;
    } catch (err) {
        console.log(err);
    }
}

async function aesEncrypt(data: Uint8Array, key: CryptoKey): Promise<Cipher> {

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

    const decOptions = {
        name: 'AES-GCM',
        iv: cipher.iv.arr.buffer
    };

    return new ByteData(await self.crypto.subtle.decrypt(decOptions, key, cipher.ct.arr.buffer));
}

async function generateECDSAKeyPair() {

    try {
        const keyPair = await self.crypto.subtle.generateKey(ECDSA_OPTIONS, true, ['sign', 'verify']);
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

async function generateECDHKeyPair() {

    try {
        const keyPair = await self.crypto.subtle.generateKey(ECDH_OPTIONS, true, ['deriveKey', 'deriveBits']);
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
