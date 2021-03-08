
export enum Jobs {
    BlindIndex,
    FieldEncryption,
    FieldDecryption,
    CreateMasterKey,
    DecryptMasterKey,
    PasswordBlindIndex
}

// The types of searching we currently support.
export enum BlindIndexType {
    Exact,
    Lowercase,
    ThreeLetter
}

// Used on search pages. Create the bloom filter lookups.
export interface BlindIndexRequest {
    type: BlindIndexType,
    table: string,
    fieldName: string,
    value: string,
    privateKey: string,
}

export interface BlindIndexResult {
    type: BlindIndexType,
    blindIndex: string,
}

// On form save, encrypt form data and create the blind indexes
export interface FieldEncryptionRequest {
    plainText: string,
    fieldName: string,
    tableName: string,
    blindIndexes: Array<BlindIndexType>,
    privateKey: string,
}

export interface FieldEncryptionResult {
    cipherText: string,
    nonce: string,
    fieldName: string,
    blindIndexes: Array<string>
}

// On view pages. Decrypt the data and show it.
export interface FieldDecryptionRequest {
    cipherTextAndNonce: string,
    fieldName: string,
    privateKey: string,
}

export interface FieldDecryptionResult {
    plainText: string,
    fieldName: string
}

// On registration - create a new wrapped private key and verify it.
export interface CreateMasterKeyRequest {
    password: string,
    email: string
}

export interface CreateMasterKeyResult {
    encryptedPrivateKey: string,
    privateKey: string
    publicKey: string
    initVector: string
    blindIndex: string
}

// On login - stretch users password and create a blind index
export interface PasswordBlindIndexRequest {
    password: string,
    email: string
}

export interface PasswordBlindIndexResult {
    blindIndex: string
}

// On login. Unwrap the master key.
export interface DecryptMasterKeyRequest {
    password: string,
    initVector: string,
    encryptedPrivateKey: string
}

export interface DecryptMasterKeyResult {
    privateKey: string
}