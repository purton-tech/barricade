import { Controller } from 'stimulus'
import { setPassword } from './util'
import { HashMasterPasswordRequest, HashMasterPasswordResult, Jobs } from '../crypto_types'


export default class extends Controller {

  static targets = ['button', 'form', 'password', 'email', 'emailCopy', 'blindIndex']

  readonly buttonTarget!: HTMLButtonElement
  readonly formTarget!: HTMLFormElement
  readonly passwordTarget!: HTMLInputElement
  readonly emailTarget!: HTMLInputElement
  readonly emailCopyTarget!: HTMLInputElement
  readonly blindIndexTarget!: HTMLInputElement

  login(event : MouseEvent) {
    event.preventDefault()
    this.passwordTarget.classList.remove('error')

    this.buttonTarget.disabled = true
    this.emailTarget.disabled = true
    this.passwordTarget.disabled = true

    const w = new Worker('../crypto_worker.ts');
    const controller = this
    w.onmessage = e => {

      const data = e.data;

      if (data.status == 'done') {
        const masterKeyResult : HashMasterPasswordResult = data.response
        console.log(masterKeyResult)
        this.emailCopyTarget.value = this.emailTarget.value
        this.blindIndexTarget.value = masterKeyResult.masterPasswordHash
        setPassword(this.passwordTarget.value)
        controller.formTarget.submit()
      }
      else if (data.status == 'working-bloom') {
        controller.buttonTarget.innerText = `Stretching Password ${data.percent}%`
      }
      else {
        console.log(data)
      }
    }

    const masterReq: HashMasterPasswordRequest = {
      masterPassword: this.passwordTarget.value,
      email: this.emailTarget.value,
      pbkdf2Iterations: 100000
    }

    w.postMessage({ cmd: Jobs[Jobs.HashMasterPassword], request: masterReq })
  }
}