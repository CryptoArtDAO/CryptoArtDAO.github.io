import {
  Inject,
  Injectable,
} from '@angular/core';
import getConfig from '../../config'
import {
  connect,
  Contract,
  keyStores,
  WalletConnection,
} from 'near-api-js'
import {
  formatNearAmount,
} from "near-api-js/lib/utils/format";
const nearConfig = getConfig(process.env.NODE_ENV || 'development')

interface Society extends Contract {
  balance(): Promise<string>
  set_greeting(value: { message: string }): void
  get_greeting(value: { account_id: string }): string | null
}

@Injectable({
  providedIn: 'root'
})
export class WalletService {
  connection: WalletConnection
  contractName: string
  contract: Society
  accountId: string
  balance: number = 0

  // constructor(@Inject(WINDOW) private window: Window) {
  constructor() {
    console.log(nearConfig)
    this.initContract().catch(reason => {
      throw new Error(reason.message)
    })
  }

  async initContract(): Promise<void> {
    // Initialize connection to the NEAR testnet
    const near = await connect(Object.assign({deps: {keyStore: new keyStores.BrowserLocalStorageKeyStore()}}, nearConfig))

    // Initializing Wallet based Account. It can work with NEAR testnet wallet that
    // is hosted at https://wallet.testnet.near.org
    // @ts-ignore
    this.connection = new WalletConnection(near)

    // Getting the Account ID. If still unauthorized, it's just empty string
    this.accountId = this.connection.getAccountId()
    this.contractName = nearConfig.contractName

    // Initializing our contract APIs by contract name and configuration
    // @ts-ignore
    this.contract = new Contract(this.connection.account(), this.contractName, {
      viewMethods: [
        'balance',
        'get_greeting',
      ],
      changeMethods: [
        'set_greeting',
      ],
    })
    await this.updateBalance()
    console.log('this.balance', this.balance)
  }

  async updateBalance(): Promise<void> {
    this.balance = Math.floor(parseFloat(formatNearAmount(await this.contract.balance())) * 100) / 100
  }

  signIn(): void {
    // Allow the current app to make calls to the specified contract on the
    // user's behalf.
    // This works by creating a new access key for the user's account and storing
    // the private key in localStorage.
    this.connection.requestSignIn(nearConfig.contractName)
  }

  isAuthenticated(): boolean {
    return !!this.accountId
  }

  signOut(): void {
    this.connection.signOut()
  }
}
