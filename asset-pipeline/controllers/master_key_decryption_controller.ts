import { Controller } from 'stimulus'
import { Vault, ProtectedKeys, Cipher } from '../vault'


export default class extends Controller {

    static targets = ['form', 'progress', 
        'publicECDSAKey', 
        'protectedECDSAPrivateKey',
        'publicECDHKey', 
        'protectedECDHPrivateKey',
        'email',  
        'protectedSymmetricKey', 'path']

    readonly formTarget!: HTMLFormElement
    readonly protectedSymmetricKeyTarget!: HTMLInputElement
    readonly emailTarget!: HTMLInputElement
    readonly pathTarget!: SVGPathElement
    // ECDSA
    readonly protectedECDSAPrivateKeyTarget!: HTMLInputElement
    readonly publicECDSAKeyTarget!: HTMLInputElement
    // ECDH
    readonly protectedECDHPrivateKeyTarget!: HTMLInputElement
    readonly publicECDHKeyTarget!: HTMLInputElement

    connect() {

        const keysToRestore: ProtectedKeys = {
            protectedSymmetricKey: Cipher.fromString(this.protectedSymmetricKeyTarget.value),
            protectedECDSAPrivateKey: Cipher.fromString(this.protectedECDSAPrivateKeyTarget.value),
            publicECDSAKey: this.publicECDSAKeyTarget.value,
            protectedECDHPrivateKey: Cipher.fromString(this.protectedECDHPrivateKeyTarget.value),
            publicECDHKey: this.publicECDHKeyTarget.value
        }

        Vault.restore(keysToRestore)

        this.formTarget.submit()
    }
}