import { Controller } from 'stimulus'
import { setPrivateKey, getPassword, removePassword } from './util'
import { DecryptMasterKeyRequest, DecryptMasterKeyResult, Jobs } from '../crypto_types'


export default class extends Controller {

    static targets = ['form', 'progress', 'initVector', 'encryptedPrivateKey', 'path']

    readonly formTarget!: HTMLFormElement
    readonly initVectorTarget!: HTMLInputElement
    readonly encryptedPrivateKeyTarget!: HTMLInputElement
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
                //setPrivateKey(fieldResult.privateKey)
                removePassword()
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

        /**const req: DecryptMasterKeyRequest = {
            masterPassword: password,
            encryptedPrivateKey: this.encryptedPrivateKeyTarget.value
        }

        w.postMessage({
            cmd: Jobs[Jobs.DecryptMasterKey],
            request: req,
        })**/
    }
}