import { Controller } from 'stimulus'
import { CreateMasterKeyRequest, CreateMasterKeyResult, Jobs } from '../crypto_types'


export default class extends Controller {

  static targets = ['button', 'form', 'password', 'confirmPassword', 'email',
    'emailCopy', 'protectedPrivateKey', 'publicKey', 'protectedSymmetricKey', 
    'masterPasswordHash']

  readonly buttonTarget!: HTMLButtonElement
  readonly formTarget!: HTMLFormElement
  readonly passwordTarget!: HTMLInputElement
  readonly emailTarget!: HTMLInputElement
  readonly confirmPasswordTarget!: HTMLInputElement

  // The hidden form
  readonly emailCopyTarget!: HTMLInputElement
  readonly masterPasswordHashTarget!: HTMLInputElement
  readonly protectedSymmetricKeyTarget!: HTMLInputElement
  readonly publicKeyTarget!: HTMLInputElement
  readonly protectedPrivateKeyTarget!: HTMLInputElement

  register(event: MouseEvent) {
    event.preventDefault()
    document.querySelectorAll('span.error').forEach(e => e.remove());
    this.passwordTarget.classList.remove('error')

    const pass1 = this.passwordTarget.value;
    const pass2 = this.confirmPasswordTarget.value;

    if (pass1 != pass2) {
      this.passwordTarget.classList.add('error')
      this.passwordTarget.insertAdjacentHTML('afterend', "<span class='error'>The passwords don't match</span>");
      return false;
    }

    this.buttonTarget.disabled = true
    this.emailTarget.disabled = true
    this.passwordTarget.disabled = true
    this.confirmPasswordTarget.disabled = true

    const w = new Worker('../crypto_worker.ts');
    const controller = this
    w.onmessage = e => {

      const data = e.data;
      if (data.status == 'done') {

        const masterKeyResult : CreateMasterKeyResult = data.response
        console.log(masterKeyResult)
        this.emailCopyTarget.value = this.emailTarget.value
        controller.protectedPrivateKeyTarget.value = masterKeyResult.protectedPrivateKey
        controller.publicKeyTarget.value = masterKeyResult.publicKey
        controller.protectedSymmetricKeyTarget.value = masterKeyResult.protectedSymmetricKey
        controller.masterPasswordHashTarget.value = masterKeyResult.masterPasswordHash
        controller.formTarget.submit()
      }
      else {
        console.log(data)
      }
    }

    const masterReq: CreateMasterKeyRequest = {
      masterPassword: pass1,
      email: controller.emailTarget.value,
      pbkdf2Iterations: 100000
    }

    w.postMessage({
      cmd: Jobs[Jobs.CreateMasterKey],
      request: masterReq
    })
  }
}