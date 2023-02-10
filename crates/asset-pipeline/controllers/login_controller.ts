import { Controller } from 'stimulus'
import { Vault } from '../vault'


export default class extends Controller {

  static targets = ['button', 'form', 'password', 'email', 'emailCopy', 'masterPasswordHash']

  readonly buttonTarget!: HTMLButtonElement
  readonly formTarget!: HTMLFormElement
  readonly passwordTarget!: HTMLInputElement
  readonly emailTarget!: HTMLInputElement
  readonly emailCopyTarget!: HTMLInputElement
  readonly masterPasswordHashTarget!: HTMLInputElement

  async login(event : MouseEvent) {
    event.preventDefault()
    this.emailTarget.classList.remove('error')
    document.querySelectorAll('.error').forEach(e => e.remove());

    this.buttonTarget.disabled = true
    this.emailTarget.disabled = true
    this.passwordTarget.disabled = true
    this.emailTarget.classList.add('disabled')
    this.passwordTarget.classList.add('disabled')
    this.buttonTarget.classList.add('disabled')
    this.buttonTarget.innerText = "Generating Your Keys"

    // We have to force email to lowercase. On safari it sometimes gets capitalized
    const email = this.emailTarget.value.toLowerCase()
    const password = this.passwordTarget.value
    
    const authToken = await Vault.unlock(password, email)
    this.emailCopyTarget.value = email
    this.masterPasswordHashTarget.value = authToken.b64
    this.formTarget.submit()
  }
}