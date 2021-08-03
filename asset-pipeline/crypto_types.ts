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
    publicKey: ByteData
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