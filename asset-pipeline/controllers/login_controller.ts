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
    this.passwordTarget.classList.remove('error')

    this.buttonTarget.disabled = true
    this.emailTarget.disabled = true
    this.passwordTarget.disabled = true

    const authToken = await Vault.unlock(this.emailTarget.value, this.passwordTarget.value)
    this.emailCopyTarget.value = this.emailTarget.value
    this.masterPasswordHashTarget.value = authToken.b64
    this.formTarget.submit()
  }
}