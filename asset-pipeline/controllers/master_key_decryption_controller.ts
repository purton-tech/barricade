import { Controller } from 'stimulus'
import { setPrivateKey, getPassword, removePassword } from './util'
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
                localStorage.setItem('unprotected_private_key', fieldResult.unprotectedPrivateKey.b64)
                localStorage.setItem('unprotected_symmetric_key', fieldResult.unprotectedSymmetricKey.key.b64)
                localStorage.setItem('public_key', this.publicKeyTarget.value)
                //removePassword()
                this.formTarget.submit()
            }
            else if (data.status == 'working-master-key') {
                let length = this.pathTarget.getTotalLength();
                let to = length * ((90 - data.percent) / 100);
                this.pathTarget.getBoundingClientRect();
                // Set the Offset
                this.pathTarget.style.strokeDashoffset = "" + Math.max(0, to);  
            }
            else {
                console.log(data)
            }
        }

        const password = getPassword()

        const req: DecryptMasterKeyRequest = {
            masterPassword: password,
            protectedPrivateKey: this.protectedPrivateKeyTarget.value,
            protectedSymmetricKey: this.protectedSymmetricKeyTarget.value,
            pbkdf2Iterations: 100000,
            email: this.emailTarget.value
        }

        w.postMessage({
            cmd: Jobs[Jobs.DecryptMasterKey],
            request: req,
        })
    }
}