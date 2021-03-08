import { SodiumPlus, CryptographyKey } from 'sodium-plus'
import { Jobs, BlindIndexType, BlindIndexRequest,
    BlindIndexResult, FieldEncryptionRequest, FieldEncryptionResult, FieldDecryptionRequest,
    FieldDecryptionResult, CreateMasterKeyRequest, CreateMasterKeyResult, PasswordBlindIndexRequest,
    PasswordBlindIndexResult, DecryptMasterKeyRequest, DecryptMasterKeyResult } from './crypto_types'


const FIELD_STORAGE_FORMAT = "base64"

// We alias self to ctx and give it our newly created type
const ctx: Worker = self as any

ctx.onmessage = async e => {

    const data = e.data

    const sodium = await SodiumPlus.auto();

    switch (data.cmd) {
        case Jobs[Jobs.BlindIndex]: {

            const blindIndex: BlindIndexRequest = data.request
    
            const blind = BlindIndexType[blindIndex.type]

            switch(blindIndex.type)  {
                case BlindIndexType.Lowercase: {

                    const lcase = blindIndex.value.toLowerCase()

                    const lcaseBlindIndex = await bloomFromValue(lcase, blindIndex.table, blindIndex.fieldName, 
                        blindIndex.type, sodium) 

                    const blindIndexResult: BlindIndexResult = {
                        type: blindIndex.type,
                        blindIndex: lcaseBlindIndex,
                    }

                    console.log(lcase + ' >> lowercase ' + blindIndexResult.blindIndex)

                    ctx.postMessage({
                        cmd: Jobs[Jobs.BlindIndex],
                        status: 'done',
                        response: blindIndexResult
                    })

                    break;
                }
                case BlindIndexType.ThreeLetter: {

                    const letter3 = blindIndex.value.toLowerCase().substr(0, 3)

                    const letter3BlindIndex = await bloomFromValue(letter3, blindIndex.table, blindIndex.fieldName, 
                        blindIndex.type, sodium) 

                    const letter3BlindIndexResult: BlindIndexResult = {
                        type: blindIndex.type,
                        blindIndex: letter3BlindIndex,
                    }

                    console.log(letter3 + ' >> 3 letter ' + letter3BlindIndexResult.blindIndex)

                    ctx.postMessage({
                        cmd: 'blind-index',
                        status: 'done',
                        response: letter3BlindIndexResult
                    })
                    
                    break;
                }
                default:
                    break;
            }

            break
        }
        case Jobs[Jobs.FieldDecryption]: {

            const fieldDecryptRequest: FieldDecryptionRequest = data.request

            let fieldDecryptPlainText = await decrypt(fieldDecryptRequest.privateKey,
                Buffer.from(fieldDecryptRequest.cipherTextAndNonce.split(':')[1], FIELD_STORAGE_FORMAT),
                Buffer.from(fieldDecryptRequest.cipherTextAndNonce.split(':')[0], FIELD_STORAGE_FORMAT),
                sodium)

            const fieldDecryptResponse: FieldDecryptionResult = {
                plainText: fieldDecryptPlainText,
                fieldName: fieldDecryptRequest.fieldName,
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.FieldDecryption],
                status: 'done',
                response: fieldDecryptResponse
            })

            break
        }
        case Jobs[Jobs.FieldEncryption]: {

            const fieldRequest: FieldEncryptionRequest = data.request

            let [ciphertextField, encryptNonceField] = await encrypt(fieldRequest.privateKey, 
                fieldRequest.plainText, sodium)

            let feBlinds : string[] = []

            for(var feBlindIndex of fieldRequest.blindIndexes) {
    
                switch(feBlindIndex) {
                    case BlindIndexType.Lowercase:

                        const lcase = fieldRequest.plainText.toLowerCase()

                        const lcaseBlindIndex = await bloomFromValue(lcase, fieldRequest.tableName, fieldRequest.fieldName, 
                            feBlindIndex, sodium) 
    
                        feBlinds.push(lcaseBlindIndex)

                        break;
                    case BlindIndexType.ThreeLetter:

                        const letter3 = fieldRequest.plainText.toLowerCase().substr(0, 3)

                        const letter3BlindIndex = await bloomFromValue(letter3, fieldRequest.tableName, fieldRequest.fieldName, 
                            feBlindIndex, sodium) 
    
                        feBlinds.push(letter3BlindIndex)
                        
                        break;
                    default:
                        break;
                }
            }

            const fieldResponse: FieldEncryptionResult = {
                cipherText: ciphertextField.toString(FIELD_STORAGE_FORMAT),
                nonce: encryptNonceField.toString(FIELD_STORAGE_FORMAT),
                fieldName: fieldRequest.fieldName,
                blindIndexes: feBlinds
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.FieldEncryption],
                status: 'done',
                response: fieldResponse
            })

            break
        }
        case Jobs[Jobs.CreateMasterKey]: {

            // Generate a random private key. Stretch the users password and encrypt that key.
            // Produces the encrypted key, the nonce used and a blind index (bloom filter)

            const masterKeyRequest: CreateMasterKeyRequest = data.request

            // Generate a random private key
            const keyPair = await sodium.crypto_box_keypair();
            const secretKey = await sodium.crypto_box_secretkey(keyPair);
            const keyToHideBase64 = secretKey.toString('base64')
            const publicKey = await sodium.crypto_box_publickey(keyPair);
            const publicKeyBase64 = publicKey.toString('base64')
            const nonce = await sodium.randombytes_buf(24);
            const salt = nonce.slice(0, 16)
            const nonceBase64 = nonce.toString('base64')

            const stretchedPwd: Buffer = await stretch(masterKeyRequest.password, salt, sodium, 'encryption')

            // Turn our password into a key
            const key = new CryptographyKey(stretchedPwd)
            const wrappedMasterKey = await sodium.crypto_secretbox(
                keyToHideBase64,
                nonce,
                key
            );

            // Verify it.
            const wrapperMasterKeyBase64 = wrappedMasterKey.toString('base64');
            const verifyMasterKeyBase64 = await decryptMasterKey(nonceBase64, masterKeyRequest.password, wrapperMasterKeyBase64, sodium)

            if (verifyMasterKeyBase64 == keyToHideBase64) {

                // Generate key for bloom filter
                const blind = await stretchWithEmail(masterKeyRequest.email, masterKeyRequest.password, sodium)

                const masterResponse: CreateMasterKeyResult = {
                    encryptedPrivateKey: wrapperMasterKeyBase64,
                    privateKey: keyToHideBase64,
                    publicKey: publicKeyBase64,
                    initVector: nonceBase64,
                    blindIndex: blind.slice(0, 8).toString('base64')
                }

                ctx.postMessage({
                    cmd: Jobs[Jobs.CreateMasterKey],
                    status: 'done',
                    response: masterResponse
                })
            }

            break
        }
        case Jobs[Jobs.PasswordBlindIndex]: {

            const passwordToIndexRequest: PasswordBlindIndexRequest = data.request

            const stretchedPassword = await stretchWithEmail(passwordToIndexRequest.email, 
                passwordToIndexRequest.password, sodium)

            const passwordToIndexResult: PasswordBlindIndexResult = {
                blindIndex: stretchedPassword.slice(0, 8).toString('base64')
            }

            ctx.postMessage({
                cmd: Jobs[Jobs.PasswordBlindIndex],
                status: 'done',
                // The bloom filter key is just the first 8 bytes.
                response: passwordToIndexResult
            })
            break
        }
        case Jobs[Jobs.DecryptMasterKey]: {

            const decryptKeyRequest: DecryptMasterKeyRequest = data.request

            let masterKeyBase64 = await decryptMasterKey(decryptKeyRequest.initVector, 
                decryptKeyRequest.password, decryptKeyRequest.encryptedPrivateKey, sodium)

            const decryptResult: DecryptMasterKeyResult = {
                privateKey: masterKeyBase64
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

async function bloomFromValue(value : string, tableName : string, fieldName : string, 
    blindType : BlindIndexType, sodium: SodiumPlus) : Promise<string> {

    
    const blind = BlindIndexType[blindType]
    // TODO derive form private key
    const salt = await sodium.crypto_generichash(tableName + value, null, 16);

    let blindBuf: Buffer = await stretch(
                value, 
                salt, sodium, fieldName + '-' + blind)

    return blindBuf.slice(0, 8).toString(FIELD_STORAGE_FORMAT)
}

async function decrypt(privateKey: string, cipherText: Buffer, nonce: Buffer, sodium: SodiumPlus): Promise<string> {

    const decryptionKey: CryptographyKey = new CryptographyKey(Buffer.from(privateKey, "base64"))
    const plainText = await sodium.crypto_secretbox_open(cipherText, nonce, decryptionKey)

    return plainText.toString()
}

async function encrypt(privateKey: string, plainText: string, sodium: SodiumPlus): Promise<[Buffer, Buffer]> {
    const encryptionKey: CryptographyKey = new CryptographyKey(Buffer.from(privateKey, "base64"))
    const encryptNonce = await sodium.randombytes_buf(24);
    const ciphertext = await sodium.crypto_secretbox(plainText, encryptNonce, encryptionKey)

    return [ciphertext, encryptNonce]
}

async function decryptMasterKey(initVector: string, password: string, encrypted_private_key: string, sodium: SodiumPlus): Promise<string> {

    const masterNonce = Buffer.from(initVector, 'base64')
    const masterSalt = masterNonce.slice(0, 16)
    const encryptedMasterKey = Buffer.from(encrypted_private_key, 'base64')
    const strPassword = await stretch(password, masterSalt, sodium, 'master-key')
    const passKey = new CryptographyKey(strPassword)

    let masterKeyBuffer: Buffer = await sodium.crypto_secretbox_open(
        encryptedMasterKey,
        masterNonce,
        passKey
    )

    return masterKeyBuffer.toString()
}

async function stretchWithEmail(email: string, password: string, sodium: SodiumPlus): Promise<Buffer> {

    // Generate key for bloom filter
    let salt = await sodium.crypto_generichash(email + password, null, 16);

    let stretched: Buffer = await stretch(password, salt, sodium, 'bloom')

    return stretched
}

async function stretch(valueToStretch: string, salt: Buffer, sodium: SodiumPlus, msg: string): Promise<Buffer> {

    // 1st round
    let hash = await sodium.crypto_pwhash(
        32,
        valueToStretch,
        salt,
        sodium.CRYPTO_PWHASH_OPSLIMIT_INTERACTIVE,
        sodium.CRYPTO_PWHASH_MEMLIMIT_INTERACTIVE
    )

    // Ten rounds of argon 2.
    for (var i = 0; i < 10; i++) {
        hash = await sodium.crypto_pwhash(
            32,
            hash.getBuffer().toString('hex'),
            salt,
            sodium.CRYPTO_PWHASH_OPSLIMIT_INTERACTIVE,
            sodium.CRYPTO_PWHASH_MEMLIMIT_INTERACTIVE
        )
        ctx.postMessage({ cmd: 'decrypt', status: `working-${msg}`, percent: ((i + 1) * 10) });
    }
    return hash.getBuffer()
}
