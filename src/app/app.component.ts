import {
  Component,
} from '@angular/core';
import {
  WalletService,
} from "./service/wallet.service";


@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.sass']
})
export class AppComponent {
  title = 'CryptoArt DAO';
  formProposalForBecomeMember = false
  proposalForBecomeMemberTitle = ''
  proposalForBecomeMemberDescription = ''
  showConsoleHelp = false

  constructor(
    public wallet: WalletService,
  ) {
  }

  sampleMemberProposal(): string {
    return JSON.stringify({
      title: 'max 170',
      description: 'max 1k'
    });
  }

  sampleVote(): string {
    return JSON.stringify({
      proposal_id: 0,
    });
  }

  catAddProposalForBecomeMember(): boolean {
    return !this.wallet.isAuthenticated() || this.hasAddProposalForBecomeMember()
  }

  addProposalForBecomeMember(): void {
    this.formProposalForBecomeMember = !this.formProposalForBecomeMember
  }

  toggleConsoleHelp(): void {
    this.showConsoleHelp = !this.showConsoleHelp
  }

  async sendProposalForBecomeMember() {
    await this.wallet.addMemberProposal(this.proposalForBecomeMemberTitle, this.proposalForBecomeMemberDescription)
    this.formProposalForBecomeMember = false
  }

  hasAddProposalForBecomeMember(): boolean {
    return this.wallet.proposalList.filter((it) => {
      console.log(it)
      return it.proposal.author === this.wallet.accountId
    }).length > 0
  }

  titleFull(): string {
    return `${this.title} (${this.wallet.contractName})`
  }
}
