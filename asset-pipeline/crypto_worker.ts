import { Jobs, CreateMasterKeyRequest, CreateMasterKeyResult,
    DecryptMasterKeyRequest, DecryptMasterKeyResult,
    ByteData, SymmetricCryptoKey  } from './crypto_types'


const FIELD_STORAGE_FORMAT = "base64"

// We alias self to ctx and give it our newly created type
const ctx: Worker = self as any

ctx.onmessage = async e => {

    const data = e.data

    switch (data.cmd) {
        case Jobs[Jobs.CreateMasterKey]: {

            // Generate a random private key. Stretch the users password and encrypt that key.
            // Produces the encrypted key, the nonce used and a blind index (bloom filter)

            const mkr: CreateMasterKeyRequest = data.request

            const masterPassword = fromUtf8(mkr.masterPassword)
            const email = fromUtf8(mkr.email)

            const masterKey = await pbkdf2(masterPassword, email, mkr.pbkdf2Iterations, 256);
            const stretchedMasterKey = await stretchKey(masterKey.arr)
            const masterKeyHash = await pbkdf2(masterKey.arr, masterPassword, 1, 256);

            let symKey = new Uint8Array(512 / 8);
            self.crypto.getRandomValues(symKey);
            const symmetricKey = new SymmetricCryptoKey(symKey);
            const protectedSymKey = await aesEncrypt(symmetricKey.key.arr, 
                stretchedMasterKey.encKey,
                stretchedMasterKey.macKey);

            const keyPair = await generateRsaKeyPair();
            const publicKey = keyPair.publicKey;
            const privateKey = keyPair.privateKey;
            const protectedPrivateKey = await aesEncrypt(privateKey.arr, symmetricKey.encKey,
                symmetricKey.macKey);

            const masterResponse: CreateMasterKeyResult = {
                masterPasswordHash: masterKeyHash.b64,
                protectedSymmetricKey: protectedSymKey.string,
                protectedPrivateKey: protectedPrivateKey.string,
                publicKey: publicKey.b64
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.CreateMasterKey],
                status: 'done',
                response: masterResponse
            })

            break
        }
        case Jobs[Jobs.DecryptMasterKey]: {

            const decryptKeyRequest: DecryptMasterKeyRequest = data.request

            const masterPassword = fromUtf8(decryptKeyRequest.masterPassword)
            const email = fromUtf8(decryptKeyRequest.email)

            const masterKey = await pbkdf2(masterPassword, email, 
                decryptKeyRequest.pbkdf2Iterations, 256);
            const stretchedMasterKey = await stretchKey(masterKey.arr)

            let decryptedSymKey = await aesDecrypt(decryptKeyRequest.protectedSymmetricKey, 
                stretchedMasterKey.encKey,
                stretchedMasterKey.macKey);
            const unprotectedSymKey = new SymmetricCryptoKey(decryptedSymKey);

            let decryptedPrivateKey = await aesDecrypt(decryptKeyRequest.protectedPrivateKey, 
                unprotectedSymKey.encKey,
                unprotectedSymKey.macKey);

            const decryptResult: DecryptMasterKeyResult = {
                unprotectedSymmetricKey: unprotectedSymKey,
                unprotectedPrivateKey: decryptedPrivateKey,
                publicKey: decryptedPrivateKey
            }

            ctx.postMessage({
                cmd: 'master-key',
                status: 'done',
                response: decryptResult
            })
            break
        }
        default:
            ctx.postMessage('Unknown command: ' + data.cmd);
    }
}

const encTypes = {
    AesCbc256_B64: 0,
    AesCbc128_HmacSha256_B64: 1,
    AesCbc256_HmacSha256_B64: 2,
    Rsa2048_OaepSha256_B64: 3,
    Rsa2048_OaepSha1_B64: 4,
    Rsa2048_OaepSha256_HmacSha256_B64: 5,
    Rsa2048_OaepSha1_HmacSha256_B64: 6
};

// Object Classes

class Cipher {

    encType: number
    iv: ByteData
    ct: ByteData
    mac: ByteData
    string: string

    constructor(encType, iv, ct, mac) {
        if (!arguments.length) {
            this.encType = null;
            this.iv = null;
            this.ct = null;
            this.mac = null;
            this.string = null;
            return;
        }

        this.encType = encType;
        this.iv = iv;
        this.ct = ct;
        this.string = encType + '.' + iv.b64 + '|' + ct.b64;

        this.mac = null;
        if (mac) {
            this.mac = mac;
            this.string += ('|' + mac.b64);
        }
    }
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

async function pbkdf2(password, salt, iterations, length) {
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
        name: 'AES-CBC',
        length: length
    };

    try {
        const importedKey = await self.crypto.subtle.importKey(
            'raw', password, importAlg, false, ['deriveKey']);
        const derivedKey = await self.crypto.subtle.deriveKey(
            deriveAlg, importedKey, aesOptions, true, ['encrypt']);
        const exportedKey = await self.crypto.subtle.exportKey('raw', derivedKey);
        return new ByteData(exportedKey);
    } catch (err) {
        console.log(err);
    }
}

async function aesEncrypt(data, encKey, macKey) {
    const keyOptions = {
        name: 'AES-CBC'
    };

    const encOptions = {
        name: 'AES-CBC',
        iv: new Uint8Array(16)
    };
    self.crypto.getRandomValues(encOptions.iv);
    const ivData = new ByteData(encOptions.iv.buffer);

    try {
        const importedKey = await self.crypto.subtle.importKey(
            'raw', encKey.arr.buffer, keyOptions, false, ['encrypt']);
        const encryptedBuffer = await self.crypto.subtle.encrypt(encOptions, importedKey, data);
        const ctData = new ByteData(encryptedBuffer);
        let type = encTypes.AesCbc256_B64;
        let macData;
        if (macKey) {
            const dataForMac = buildDataForMac(ivData.arr, ctData.arr);
            const macBuffer = await computeMac(dataForMac.buffer, macKey.arr.buffer);
            type = encTypes.AesCbc256_HmacSha256_B64;
            macData = new ByteData(macBuffer);
        }
        return new Cipher(type, ivData, ctData, macData);
    } catch (err) {
        console.error(err);
    }
}

async function aesDecrypt(cipher, encKey, macKey) {
    const keyOptions = {
        name: 'AES-CBC'
    };

    const decOptions = {
        name: 'AES-CBC',
        iv: cipher.iv.arr.buffer
    };

    try {
        const checkMac = cipher.encType != encTypes.AesCbc256_B64;
        if (checkMac) {
            if (!macKey) {
                throw 'MAC key not provided.';
            }
            const dataForMac = buildDataForMac(cipher.iv.arr, cipher.ct.arr);
            const macBuffer = await computeMac(dataForMac.buffer, macKey.arr.buffer);
            const macsMatch = await macsEqual(cipher.mac.arr.buffer, macBuffer, macKey.arr.buffer);
            if (!macsMatch) {
                throw 'MAC check failed.';
            }
            const importedKey = await self.crypto.subtle.importKey(
                'raw', encKey.arr.buffer, keyOptions, false, ['decrypt']);
            return self.crypto.subtle.decrypt(decOptions, importedKey, cipher.ct.arr.buffer);
        }
    } catch (err) {
        console.error(err);
    }
}

async function computeMac(data, key) {
    const alg = {
        name: 'HMAC',
        hash: { name: 'SHA-256' }
    };
    const importedKey = await self.crypto.subtle.importKey('raw', key, alg, false, ['sign']);
    return self.crypto.subtle.sign(alg, importedKey, data);
}

async function macsEqual(mac1Data, mac2Data, key) {
    const alg = {
        name: 'HMAC',
        hash: { name: 'SHA-256' }
    };

    const importedMacKey = await self.crypto.subtle.importKey('raw', key, alg, false, ['sign']);
    const mac1 = await self.crypto.subtle.sign(alg, importedMacKey, mac1Data);
    const mac2 = await self.crypto.subtle.sign(alg, importedMacKey, mac2Data);

    if (mac1.byteLength !== mac2.byteLength) {
        return false;
    }

    const arr1 = new Uint8Array(mac1);
    const arr2 = new Uint8Array(mac2);

    for (let i = 0; i < arr2.length; i++) {
        if (arr1[i] !== arr2[i]) {
            return false;
        }
    }

    return true;
}

function buildDataForMac(ivArr, ctArr) {
    const dataForMac = new Uint8Array(ivArr.length + ctArr.length);
    dataForMac.set(ivArr, 0);
    dataForMac.set(ctArr, ivArr.length);
    return dataForMac;
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

// Not tested.
async function importRsaKeyPair(publicKeyData: ByteData, privateKeyData: ByteData) {
    const rsaOptions = {
        name: 'RSA-OAEP',
        modulusLength: 2048,
        publicExponent: new Uint8Array([0x01, 0x00, 0x01]), // 65537
        hash: { name: 'SHA-1' }
    };

    try {
        const publicKey = new ByteData(await self.crypto.subtle.importKey('spki', publicKeyData.arr, rsaOptions, true, ['encrypt', 'decrypt']));
        const privateKey = new ByteData(await self.crypto.subtle.importKey('pkcs8', privateKeyData.arr, rsaOptions, true, ['encrypt', 'decrypt']));
        return {
            publicKey: publicKey,
            privateKey: privateKey
        };
    } catch (err) {
        console.error(err);
    }
}

async function stretchKey(key) {
    const newKey = new Uint8Array(64);
    newKey.set(await hkdfExpand(key, new Uint8Array(fromUtf8('enc')), 32));
    newKey.set(await hkdfExpand(key, new Uint8Array(fromUtf8('mac')), 32), 32);
    return new SymmetricCryptoKey(newKey.buffer);
}

// ref: https://tools.ietf.org/html/rfc5869
async function hkdfExpand(prk, info, size) {
    const alg = {
        name: 'HMAC',
        hash: { name: 'SHA-256' }
    };
    const importedKey = await self.crypto.subtle.importKey('raw', prk, alg, false, ['sign']);
    const hashLen = 32; // sha256
    const okm = new Uint8Array(size);
    let previousT = new Uint8Array(0);
    const n = Math.ceil(size / hashLen);
    for (let i = 0; i < n; i++) {
        const t = new Uint8Array(previousT.length + info.length + 1);
        t.set(previousT);
        t.set(info, previousT.length);
        t.set([i + 1], t.length - 1);
        previousT = new Uint8Array(await self.crypto.subtle.sign(alg, importedKey, t.buffer));
        okm.set(previousT, i * hashLen);
    }
    return okm;
}
