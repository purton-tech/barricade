import { Controller } from 'stimulus'
import { UnlockVaultWithMasterPasswordRequest, Jobs } from '../crypto_types'


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
        // When we unlock the vault we get an auth token
        console.log(data.response)
        this.emailCopyTarget.value = this.emailTarget.value
        this.masterPasswordHashTarget.value = data.response
        controller.formTarget.submit()
      }
      else {
        console.log(data)
      }
    }

    const masterReq: UnlockVaultWithMasterPasswordRequest = {
      masterPassword: this.passwordTarget.value,
      email: this.emailTarget.value
    }

    w.postMessage({ cmd: Jobs[Jobs.UnlockVaultWithMasterPassword], request: masterReq })
  }
}