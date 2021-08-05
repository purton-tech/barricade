import { Controller } from 'stimulus'
import { DecryptMasterKeyRequest, DecryptMasterKeyResult, Jobs } from '../crypto_types'


export default class extends Controller {

    static targets = ['form', 'progress', 'publicKey', 'email', 'protectedPrivateKey', 'protectedSymmetricKey', 'path']

    readonly formTarget!: HTMLFormElement
    readonly protectedPrivateKeyTarget!: HTMLInputElement
    readonly protectedSymmetricKeyTarget!: HTMLInputElement
    readonly publicKeyTarget!: HTMLInputElement
    readonly emailTarget!: HTMLInputElement
    readonly pathTarget!: SVGPathElement

    connect() {

        const w = new Worker('../crypto_worker.ts');
        const controller = this
        w.onmessage = e => {

            const data = e.data;
            const fieldResult : DecryptMasterKeyResult = data.response
            console.log(data)

            if (data.status == 'done') {
                console.log(data)
                this.formTarget.submit()
            }
            else {
                console.log(data)
            }
        }

        const req: DecryptMasterKeyRequest = {
            protectedPrivateKey: this.protectedPrivateKeyTarget.value,
            protectedSymmetricKey: this.protectedSymmetricKeyTarget.value,
            publicKey: this.publicKeyTarget.value
        }

        w.postMessage({
            cmd: Jobs[Jobs.DecryptMasterKey],
            request: req,
        })
    }
}