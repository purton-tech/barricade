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
    protectedSymmetricKey: string
    protectedPrivateKey: string
    publicKey: string
}

export interface DecryptMasterKeyResult {
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