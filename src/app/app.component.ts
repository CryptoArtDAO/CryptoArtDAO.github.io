import {
  Component,
  OnInit,
  Inject,
} from '@angular/core';
import {
  WalletService,
} from "./service/wallet.service";


@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent implements OnInit {
  title = 'CryptoArt DAO';
  greeting: string = ''
  newGreeting: string = ''
  showNotification = false

  constructor(
    public wallet: WalletService,
  ) {
  }

  sample(): string {
    return JSON.stringify({
      title: 'max 170',
      description: 'max 1k'
    });
  }

  // get accountId(): string {
  //   return this.window.connection.getAccountId()
  // }
  //
  // get signedIn(): boolean {
  //   return this.window.connection.isSignedIn()
  // }
  //
  // get contractId(): string {
  //   return this.window.contract.contractId
  // }

  // get buttonDisabled(): boolean {
  //   const newGreeting = this.newGreeting?.trim()
  //   return !newGreeting || newGreeting === this.greeting
  // }

  // constructor(@Inject(WINDOW) private window: Window) {}

  async ngOnInit(): Promise<void> {
    // await this.fetchGreeting()
    console.log(this.wallet)
  }

  titleFull(): string {
    return `${this.title} (${this.wallet.contractName})`
    //return `${this.title}`
  }
  // signIn(): void {
  //   signIn()
  // }
  //


  //
  // async fetchGreeting(): Promise<void> {
  //   if (this.signedIn) {
  //     const result = await this.window.contract.get_greeting({ account_id: this.accountId })
  //     if (result) {
  //       this.greeting = result
  //       this.newGreeting =  result
  //     }
  //   }
  // }
  //
  // async onSubmit(event: any): Promise<void> {
  //   console.log('event', event)
  //   event.preventDefault()
  //
  //   // get elements from the form using their id attribute
  //   const { fieldset, greeting } = event.target.elements
  //
  //   // disable the form while the value gets updated on-chain
  //   fieldset.disabled = true
  //
  //   try {
  //     // make an update call to the smart contract
  //     await this.window.contract.set_greeting({ message: greeting.value })
  //   } catch (e) {
  //     alert(
  //       'Something went wrong! ' +
  //       'Maybe you need to sign out and back in? ' +
  //       'Check your browser console for more info.'
  //     )
  //     throw e
  //   } finally {
  //     // re-enable the form, whether the call succeeded or failed
  //     fieldset.disabled = false
  //   }
  //
  //   // update local `greeting` variable to match persisted value
  //   this.greeting = this.newGreeting
  //
  //   // show notification
  //   this.showNotification = true
  //
  //   // remove notification again after css animation completes
  //   // this allows it to be shown again next time the form is submitted
  //   setTimeout(() => {
  //     this.showNotification = false
  //   }, 11000)
  // }
}
