import { Controller } from 'stimulus'
import { InitialiseVaultWithNewKeysRequest, InitialiseVaultWithNewKeysResult, Jobs } from '../crypto_types'


export default class extends Controller {

  static targets = ['button', 
    'form', 'password', 'confirmPassword', 'email',
    'emailCopy', 
    'protectedECDSAPrivateKey', 
    'publicECDSAKey', 
    'protectedECDHPrivateKey', 
    'publicECDHKey', 
    'protectedSymmetricKey', 
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
  readonly publicECDHKeyTarget!: HTMLInputElement
  readonly protectedECDHPrivateKeyTarget!: HTMLInputElement
  readonly publicECDSAKeyTarget!: HTMLInputElement
  readonly protectedECDSAPrivateKeyTarget!: HTMLInputElement

  register(event: MouseEvent) {
    event.preventDefault()
    document.querySelectorAll('span.error').forEach(e => e.remove());
    this.passwordTarget.classList.remove('error')

    const pass1 = this.passwordTarget.value.normalize('NFC');
    const pass2 = this.confirmPasswordTarget.value.normalize('NFC');

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

        const masterKeyResult : InitialiseVaultWithNewKeysResult = data.response
        console.log(masterKeyResult)
        this.emailCopyTarget.value = this.emailTarget.value
        controller.protectedECDSAPrivateKeyTarget.value = masterKeyResult.protectedECDHPrivateKey
        controller.publicECDSAKeyTarget.value = masterKeyResult.publicECDHKey
        controller.protectedECDHPrivateKeyTarget.value = masterKeyResult.protectedECDSAPrivateKey
        controller.publicECDHKeyTarget.value = masterKeyResult.publicECDSAKey
        controller.protectedSymmetricKeyTarget.value = masterKeyResult.protectedSymmetricKey
        controller.masterPasswordHashTarget.value = masterKeyResult.masterPasswordHash
        controller.formTarget.submit()
      }
      else {
        console.log(data)
      }
    }

    const masterReq: InitialiseVaultWithNewKeysRequest = {
      masterPassword: pass1,
      email: controller.emailTarget.value,
      pbkdf2Iterations: 100000
    }

    w.postMessage({
      cmd: Jobs[Jobs.InitialiseVaultWithNewKeys],
      request: masterReq
    })
  }
}