import { Controller } from 'stimulus'
import { HashMasterPasswordRequest, HashMasterPasswordResult, Jobs } from '../crypto_types'


export default class extends Controller {

  static targets = ['button', 'form', 'password', 'email', 'emailCopy', 'masterPasswordHash']

  readonly buttonTarget!: HTMLButtonElement
  readonly formTarget!: HTMLFormElement
  readonly passwordTarget!: HTMLInputElement
  readonly emailTarget!: HTMLInputElement
  readonly emailCopyTarget!: HTMLInputElement
  readonly masterPasswordHashTarget!: HTMLInputElement

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
        this.masterPasswordHashTarget.value = masterKeyResult.masterPasswordHash
        controller.formTarget.submit()
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