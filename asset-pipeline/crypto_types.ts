export enum Jobs {
    CreateMasterKey,
    DecryptMasterKey,
    HashMasterPassword
}

export interface HashMasterPasswordRequest {
    masterPassword: string
    email: string
    pbkdf2Iterations: number
}

export interface HashMasterPasswordResult {
    masterPasswordHash: string
}

// On registration - create protected rsa and symmetric keys.
export interface CreateMasterKeyRequest {
    masterPassword: string
    email: string
    pbkdf2Iterations: number
}

export interface CreateMasterKeyResult {
    masterPasswordHash: string
    protectedSymmetricKey: string
    protectedPrivateKey: string
    publicKey: string
}

// After login. Unwrap the master key.
export interface DecryptMasterKeyRequest {
    masterPassword: string,
    email: string
    protectedSymmetricKey: string
    protectedPrivateKey: string
    pbkdf2Iterations: number
}

export interface DecryptMasterKeyResult {
    unprotectedSymmetricKey: SymmetricCryptoKey
    unprotectedPrivateKey: ByteData
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

    encType: number
    iv: ByteData
    ct: ByteData
    mac: ByteData
    string: string

    constructor(encType: number, iv: ByteData, ct: ByteData, mac: ByteData) {
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

    static fromString(string: string) : Cipher {
        const encType = parseInt(string.split('.')[0])
        const iv = ByteData.fromB64(string.split('.')[1].split('|')[0])
        const ct = ByteData.fromB64(string.split('.')[1].split('|')[1])
        const mac = ByteData.fromB64(string.split('.')[1].split('|')[2])
        return new this(encType, iv, ct, mac)
    }
}

export class SymmetricCryptoKey {

    key: ByteData
    encKey: ByteData
    macKey: ByteData

    constructor(buf) {
        if (!arguments.length) {
            this.key = new ByteData(null);
            this.encKey = new ByteData(null);
            this.macKey = new ByteData(null);
            return;
        }

        this.key = new ByteData(buf);

        // First half
        const encKey = this.key.arr.slice(0, this.key.arr.length / 2).buffer;
        this.encKey = new ByteData(encKey);

        // Second half
        const macKey = this.key.arr.slice(this.key.arr.length / 2).buffer;
        this.macKey = new ByteData(macKey);
    }
}