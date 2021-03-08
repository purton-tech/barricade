import '../node_modules/node-snackbar/dist/snackbar.css'
import './index.scss'

import { Application } from "stimulus"

//import SnackbarController from '../src/components/snackbar_controller'
//import ApplicationLayoutController from './controllers/application_layout_controller'
import MasterController from './master_key_decryption_controller'
import RegistrationController from './controllers/registration_controller'
import LoginController from './controllers/login_controller'
//import DecryptController from './controllers/decrypt_controller'
//import LogoutController from './controllers/logout_controller'

const application = Application.start()
//application.register("notice", SnackbarController)
//application.register("application-layout", ApplicationLayoutController)
application.register("master", MasterController)
//application.register("decrypt", DecryptController)
//application.register("logout", LogoutController)
application.register("registration", RegistrationController)
application.register("login", LoginController)
