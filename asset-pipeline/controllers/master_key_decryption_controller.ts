import { Controller } from 'stimulus'
import { ImportProtectedKeysIntoVaultRequest, Jobs } from '../crypto_types'


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

        const w = new Worker('../crypto_worker.ts');
        w.onmessage = e => {

            const data = e.data;
            this.formTarget.submit()
        }

        const req: ImportProtectedKeysIntoVaultRequest = {
            protectedSymmetricKey: this.protectedSymmetricKeyTarget.value,
            protectedECDSAPrivateKey: this.protectedECDSAPrivateKeyTarget.value,
            publicECDSAKey: this.publicECDSAKeyTarget.value,
            protectedECDHPrivateKey: this.protectedECDHPrivateKeyTarget.value,
            publicECDHKey: this.publicECDHKeyTarget.value
        }

        w.postMessage({
            cmd: Jobs[Jobs.ImportProtectedKeysIntoVault],
            request: req,
        })
    }
}