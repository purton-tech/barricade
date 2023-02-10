import './node_modules/node-snackbar/dist/snackbar.css'
import './index.scss'

import { Application } from "stimulus"

import MasterController from './controllers/master_key_decryption_controller'
import RegistrationController from './controllers/registration_controller'
import LoginController from './controllers/login_controller'
import PasswordController from './controllers/password_controller'

const application = Application.start()
application.register("master", MasterController)
application.register("registration", RegistrationController)
application.register("login", LoginController)
application.register("password", PasswordController)
