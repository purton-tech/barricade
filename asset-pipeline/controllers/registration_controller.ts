import { Controller } from 'stimulus'
import { setPrivateKey } from './util'
import { CreateMasterKeyRequest, CreateMasterKeyResult, Jobs } from '../crypto_types'


export default class extends Controller {

  static targets = ['button', 'form', 'password', 'confirmPassword', 'email',
    'emailCopy', 'encryptedPrivateKey', 'publicKey', 'blindIndex', 'initVector']

  readonly buttonTarget!: HTMLButtonElement
  readonly formTarget!: HTMLFormElement
  readonly passwordTarget!: HTMLInputElement
  readonly emailTarget!: HTMLInputElement
  readonly emailCopyTarget!: HTMLInputElement
  readonly blindIndexTarget!: HTMLInputElement
  readonly confirmPasswordTarget!: HTMLInputElement
  readonly encryptedPrivateKeyTarget!: HTMLInputElement
  readonly publicKeyTarget!: HTMLInputElement
  readonly initVectorTarget!: HTMLInputElement

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
        console.log(data)
        this.emailCopyTarget.value = this.emailTarget.value
        controller.encryptedPrivateKeyTarget.value = masterKeyResult.encryptedPrivateKey
        controller.publicKeyTarget.value = masterKeyResult.publicKey
        controller.initVectorTarget.value = masterKeyResult.initVector
        controller.blindIndexTarget.value = masterKeyResult.blindIndex
        setPrivateKey(masterKeyResult.privateKey)
        controller.formTarget.submit()
      }
      else if (data.status == 'working-encryption') {
        controller.buttonTarget.innerText = `Stretching Password ${data.percent}%`
      }
      else if (data.status == 'working-master-key') {
        controller.buttonTarget.innerText = `Verifying Encryption ${data.percent}%`
      }
      else if (data.status == 'working-bloom') {
        controller.buttonTarget.innerText = `Generating Blind Index ${data.percent}%`
      }
      else {
        console.log(data)
      }
    }

    const masterReq: CreateMasterKeyRequest = {
      password: pass1,
      email: this.emailTarget.value
    }

    w.postMessage({
      cmd: Jobs[Jobs.CreateMasterKey],
      request: masterReq
    })
  }
}