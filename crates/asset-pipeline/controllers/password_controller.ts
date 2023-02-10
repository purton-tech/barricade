import { Controller } from 'stimulus'
import { zxcvbn, ZxcvbnOptions } from '@zxcvbn-ts/core'
import zxcvbnCommonPackage from '@zxcvbn-ts/language-common'
import zxcvbnEnPackage from '@zxcvbn-ts/language-en'
import { FeedbackType } from '@zxcvbn-ts/core/dist/types'

const options = {
    translations: zxcvbnEnPackage.translations,
    graphs: zxcvbnCommonPackage.adjacencyGraphs,
    dictionary: {
      ...zxcvbnCommonPackage.dictionary,
      ...zxcvbnEnPackage.dictionary,
    },
  }


export default class extends Controller {

    static targets = ['help', 'password', 'warning', 'suggestions', 'button']

    readonly helpTarget!: HTMLSpanElement
    readonly warningTarget!: HTMLSpanElement
    readonly suggestionsTarget!: HTMLSpanElement
    readonly passwordTarget!: HTMLInputElement
    readonly buttonTarget!: HTMLButtonElement

    connect() {
        ZxcvbnOptions.setOptions(options)
        this.warningTarget.style.display = 'none'
        this.suggestionsTarget.style.display = 'none'
        this.buttonTarget.innerText = "Password Strength 0 out of 4" 
        this.buttonTarget.disabled = true
        this.buttonTarget.classList.add('disabled')
    }

    async keyPress(event: InputEvent) {
        const result = zxcvbn(this.passwordTarget.value)
        if(result.feedback) {
            const feedback: FeedbackType = result.feedback
            if(feedback.warning != '') {
                this.warningTarget.innerText = feedback.warning
                this.warningTarget.style.display = 'inline'
                this.helpTarget.style.display = 'none'
            } else {
                this.helpTarget.style.display = 'inline'
                this.warningTarget.style.display = 'none'
            }
        } else {
            //this.helpTarget.innerText = this.helpText
            this.helpTarget.style.display = 'inline'
            this.warningTarget.style.display = 'none'
        }
        if(result.score < 3) {
            this.buttonTarget.innerText = "Password Strength " + result.score + " out of 4"
            this.buttonTarget.disabled = true
            this.buttonTarget.classList.add('disabled')
        } else {
            this.buttonTarget.innerText = "Sign Up"
            this.buttonTarget.disabled = false
            this.buttonTarget.classList.remove('disabled')
        }
    }
}