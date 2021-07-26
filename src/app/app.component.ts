import {Component,} from '@angular/core';
import {WalletService,} from "./service/wallet.service";


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
  proposalFundTitle = ''
  proposalFundDescription = ''
  proposalFundValue = 0
  showConsoleHelp = false
  showFormProposalFund = false
  showHelp = false
  showArchive = false
  voteProcess = false
  proposalProcess = false

  constructor(
    public wallet: WalletService,
  ) {
  }

  sampleMemberProposal(): string {
    return JSON.stringify({
      title: 'optional max 170',
      description: 'optional max 1k'
    });
  }

  sampleFundProposal(): string {
    return JSON.stringify({
      title: 'require max 170',
      description: 'require max 1k',
      script: JSON.stringify({
        fund: '10000000000000000000000000',
      }),
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

  toggleProposalForBecomeMember(): void {
    this.formProposalForBecomeMember = !this.formProposalForBecomeMember
  }

  toggleFormProposalFund(): void {
    this.showFormProposalFund = !this.showFormProposalFund
  }

  toggleConsoleHelp(): void {
    this.showConsoleHelp = !this.showConsoleHelp
  }

  toggleHelp(): void {
    this.showHelp = !this.showHelp
  }

  toggleArchive(): void {
    this.showArchive = !this.showArchive
  }

  voteApprove(proposal_id: number): void {
    this.voteProcess = true
    this.wallet.voteApprove(proposal_id).then(async () => {
      await this.wallet.update()
    })
  }

  voteReject(proposal_id: number): void {
    this.voteProcess = true
    this.wallet.voteReject(proposal_id).then(async () => {
      await this.wallet.update()
    })
  }

  sendProposalForBecomeMember(): void {
    this.proposalProcess = true
    this.wallet.addMemberProposal(this.proposalForBecomeMemberTitle, this.proposalForBecomeMemberDescription).then(async () => {
      await this.wallet.update()
      this.formProposalForBecomeMember = false
      this.proposalProcess = false
    })
  }

  sendProposalFund(): void {
    this.proposalProcess = true
    this.wallet.addFundProposal(
      this.proposalFundTitle,
      this.proposalFundDescription,
      this.proposalFundValue.toString(),
    ).then(async () => {
      await this.wallet.update()
      this.showFormProposalFund = false
      this.proposalProcess = false
    })
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
