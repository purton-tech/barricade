import { openDB } from 'idb';

export interface ProtectedKeys {
    protectedSymmetricKey: Cipher
    protectedECDSAPrivateKey: Cipher
    publicECDSAKey: string
    protectedECDHPrivateKey: Cipher
    publicECDHKey: string
}

// NIST recommends ten million for secure applications.
const NIST_RECOMMENDED_PBKDF_ITERATIONS = 10_000_000

const ECDSA_OPTIONS = {
    name: "ECDSA",
    namedCurve: "P-256"
};

const ECDH_OPTIONS = {
    name: "ECDH",
    namedCurve: "P-256"
};

const AES_OPTIONS = {
    name: "AES-GCM",
    length: 256
};

/**
 * Adds two numbers together.
 * @example
 * A logon workflow would look something like:
 * ```
 * // Unlock the vault and get an auth token.
 * const authToken = Vault.unlock('test@example.com', 'myverysecurepassword##')
 * // If auth is successful, we get our protected keys from the server
 * Vault.restoreSymmetricKey('asddaasdas')
 * Vault.restoreECDHKey('asdasdds')
 * ```
 * @example
 * A registration workflow would look something like:
 * ```
 * // Get the auth token and pass to server
 * const authToken = Vault.unlock('test@example.com', 'myverysecurepassword##')
 * const protectedSymmetric = Vault.createSymmetricKey()
 * // Prints 
 * ```
 */
export class Vault {

    public static async unlock(masterPassword: string, email: string) : Promise<ByteData> {

        const [masterCryptoKey, masterKeyData, authKey, authKeyData] = 
            await this.generateMasterKey(email, masterPassword, NIST_RECOMMENDED_PBKDF_ITERATIONS)

        // Now store it in the vault
        await this.storeMasterKey(masterKeyData)

        return authKeyData
    }

    public static async new() : Promise<ProtectedKeys> {

        const db = await this.openIndexedDB()
        const masterCryptoKey = await db.get('keyval', 'master_key') as CryptoKey;

        const symKey = await self.crypto.subtle.generateKey(
            AES_OPTIONS,
            true,
            ['decrypt', 'encrypt'])
        const symKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', symKey))
        const protectedSymKey = await this.aesEncrypt(symKeyData.arr, masterCryptoKey);

        // ECDSA
        const keyPair = await this.generateECDSAKeyPair();
        const publicECDSAKey = keyPair.publicKey;
        const protectedECDSAPrivateKey = await this.aesEncrypt(keyPair.privateKey.arr, masterCryptoKey);

        // ECDH
        const keyPairDH = await this.generateECDHKeyPair();
        const publicECDHKey = keyPairDH.publicKey;
        const protectedECDHPrivateKey = await this.aesEncrypt(keyPairDH.privateKey.arr, masterCryptoKey);

        const masterResponse: ProtectedKeys = {
            protectedSymmetricKey: protectedSymKey,
            protectedECDSAPrivateKey: protectedECDSAPrivateKey,
            publicECDSAKey: publicECDSAKey.b64,
            protectedECDHPrivateKey: protectedECDHPrivateKey,
            publicECDHKey: publicECDHKey.b64
        }
        return masterResponse
    }

    public static async restore(protectedKeys: ProtectedKeys) {
        
            const db = await this.openIndexedDB()
            const masterKey = await db.get('keyval', 'master_key') as CryptoKey;

            const decryptedSymmetric = await this.aesDecrypt(protectedKeys.protectedSymmetricKey, masterKey)

            const unprotectedSymKey: CryptoKey = await self.crypto.subtle.importKey(
                'raw', decryptedSymmetric.arr.buffer, AES_OPTIONS, false, ['decrypt', 'encrypt']);

            // ECDSA
            let decryptedECDSAPrivateKey = await this.aesDecrypt(protectedKeys.protectedECDSAPrivateKey, masterKey);
            const unprotectedECDSAPrivateKey: CryptoKey = await self.crypto.subtle.importKey(
                'pkcs8', decryptedECDSAPrivateKey.arr.buffer, ECDSA_OPTIONS, false, ['sign'])
            const publicECDSAKey: CryptoKey = await self.crypto.subtle.importKey(
                'spki', ByteData.fromB64(protectedKeys.publicECDSAKey).arr.buffer, 
                ECDSA_OPTIONS,
                true, ['verify'])

            // ECDH
            let decryptedECDHPrivateKey = await this.aesDecrypt(protectedKeys.protectedECDHPrivateKey, masterKey);
            const unprotectedECDHPrivateKey: CryptoKey = await self.crypto.subtle.importKey(
                'pkcs8', decryptedECDHPrivateKey.arr.buffer, ECDH_OPTIONS, false, 
                ['deriveKey', 'deriveBits'])
            const publicECDHKey: CryptoKey = await self.crypto.subtle.importKey(
                'spki', ByteData.fromB64(protectedKeys.publicECDHKey).arr.buffer, 
                ECDH_OPTIONS,
                true, [])

            try {
                const db = await this.openIndexedDB()
                await db.put('keyval', unprotectedSymKey, 'unprotected_symmetric_key')
                await db.put('keyval', unprotectedECDSAPrivateKey, 'unprotected_ecdsa_private_key')
                await db.put('keyval', publicECDSAKey, 'ecdsa_public_key')
                await db.put('keyval', unprotectedECDHPrivateKey, 'unprotected_ecdh_private_key')
                await db.put('keyval', publicECDHKey, 'ecdh_public_key')
                db.close()
            } catch (e) {
                console.log(e)
            }
    }

    private static async generateMasterKey(username: string, password: string, iterations: number = 100_000):
        Promise<[CryptoKey, ByteData, CryptoKey, ByteData]> {
            
        let enc = new TextEncoder();
        const masterPassword = enc.encode(password.normalize('NFKC'))
        const email = enc.encode(username.normalize('NFKC'))

        const masterCryptoKey = await this.pbkdf2(masterPassword, email, iterations, 256)
        const masterKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', masterCryptoKey))
        const authKey = await this.pbkdf2(masterKeyData.arr, masterPassword, 1, 256)
        const authKeyData = new ByteData(await self.crypto.subtle.exportKey('raw', authKey))
    
        return [masterCryptoKey, masterKeyData, authKey, authKeyData]
    }


    private static async pbkdf2(password: ArrayBuffer, salt: ArrayBuffer,
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

    private static async generateECDSAKeyPair() {
    
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
    
    private static async generateECDHKeyPair() {
    
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

    private static async aesEncrypt(data: Uint8Array, key: CryptoKey): Promise<Cipher> {
    
        const encOptions = {
            name: 'AES-GCM',
            iv: new Uint8Array(16)
        };
        self.crypto.getRandomValues(encOptions.iv);
        const ivData = new ByteData(encOptions.iv.buffer);
        const cipher = new ByteData(await self.crypto.subtle.encrypt(encOptions, key, data))
    
        return new Cipher(ivData, cipher)
    }
    
    private static async aesDecrypt(cipher: Cipher, key: CryptoKey) {
    
        const decOptions = {
            name: 'AES-GCM',
            iv: cipher.iv.arr.buffer
        };
    
        return new ByteData(await self.crypto.subtle.decrypt(decOptions, key, cipher.ct.arr.buffer));
    }

    private static async openIndexedDB() {
        return await openDB('Vault', 1, {
            upgrade(db) {
                db.createObjectStore("keyval");
            },
        });
    }

    private static async storeMasterKey(masterKey: ByteData) {

        // Now store it in the vault
        try {
            const key: CryptoKey = await self.crypto.subtle.importKey(
                'raw', masterKey.arr.buffer, AES_OPTIONS, false, ['decrypt', 'encrypt']);
            const db = await this.openIndexedDB()
            await db.put('keyval', key, 'master_key');
            db.close()
        } catch (e) {
            console.log(e)
        }
    }
}

export class ByteData {

    arr: Uint8Array
    b64: string

    constructor(buf) {
        if (!arguments.length) {
            this.arr = null;
            this.b64 = null;
            return;
        }

        this.arr = new Uint8Array(buf);
        this.b64 = this.toB64(buf);
    }

    toB64(buf) {
        let binary = '';
        const bytes = new Uint8Array(buf);
        for (let i = 0; i < bytes.byteLength; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        return btoa(binary);
    }
    
    static fromB64(base64: string) : ByteData {
        var binary_string = atob(base64);
        var len = binary_string.length;
        var bytes = new Uint8Array(len);
        for (var i = 0; i < len; i++) {
            bytes[i] = binary_string.charCodeAt(i);
        }
        return new this(bytes);
    }
}

export class Cipher {

    iv: ByteData
    ct: ByteData
    string: string

    constructor(iv: ByteData, ct: ByteData) {
        if (!arguments.length) {
            this.iv = null;
            this.ct = null;
            this.string = null;
            return;
        }

        this.iv = iv;
        this.ct = ct;
        this.string = iv.b64 + '|' + ct.b64;
    }

    static fromString(string: string) : Cipher {
        const iv = ByteData.fromB64(string.split('|')[0])
        const ct = ByteData.fromB64(string.split('|')[1])
        return new this(iv, ct)
    }
}