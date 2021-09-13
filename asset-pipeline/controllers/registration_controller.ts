import { Controller } from 'stimulus'
import { Vault } from '../vault'

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

    async register(event: MouseEvent) {
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
        this.emailTarget.classList.add('disabled')
        this.passwordTarget.classList.add('disabled')
        this.confirmPasswordTarget.classList.add('disabled')
        this.buttonTarget.classList.add('disabled')

        // Derive an authToken
        this.buttonTarget.innerText = "Generating Your Keys"
        const authToken = await Vault.unlock(pass1, this.emailTarget.value)
        const protectedKeys = await Vault.new()

        this.emailCopyTarget.value = this.emailTarget.value
        this.protectedECDSAPrivateKeyTarget.value = protectedKeys.protectedECDHPrivateKey.string
        this.publicECDSAKeyTarget.value = protectedKeys.publicECDHKey
        this.protectedECDHPrivateKeyTarget.value = protectedKeys.protectedECDSAPrivateKey.string
        this.publicECDHKeyTarget.value = protectedKeys.publicECDSAKey
        this.protectedSymmetricKeyTarget.value = protectedKeys.protectedSymmetricKey.string
        this.masterPasswordHashTarget.value = authToken.b64
        this.formTarget.submit()
    }
}