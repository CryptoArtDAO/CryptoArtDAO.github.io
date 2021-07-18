import {Injectable,} from '@angular/core';
import getConfig from '../../config'
import {connect, Contract, keyStores, WalletConnection,} from 'near-api-js'
import {formatNearAmount,} from "near-api-js/lib/utils/format";

const nearConfig = getConfig(process.env.NODE_ENV || 'development')

interface ProposalVote {
  approve: number
  reject: number
}
interface Proposal {
  id: number
  title: string
  description: string
  kind: string
  status: string
  author: string
  vote: ProposalVote
}

interface ProposalOfAccount {
  proposal: Proposal
  canVote: boolean
}

interface Society extends Contract {
  balance(): Promise<string>
  member_list(): Promise<string[]>
  proposal_list(): Promise<Proposal[]>
  is_member(account_id: string): Promise<boolean>
  cat_vote(proposal_id: number, account_id: string): Promise<boolean>
  add_member_proposal(param: Object, gas: string, amount: string): Promise<number>
  vote_reject(param: Object, gas: string, amount: string): Promise<void>
  vote_approve(param: Object, gas: string, amount: string): Promise<void>
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
  memberList: string[]  = []
  proposalList: Proposal[] = []
  proposalOfAccountList: ProposalOfAccount[] = []
  isMember: boolean = false

  // constructor(@Inject(WINDOW) private window: Window) {
  constructor() {
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
        'member_list',
        'proposal_list',
        'is_member',
        'cat_vote',
      ],
      changeMethods: [
        'add_member_proposal',
        'vote_approve',
        'vote_reject',
      ],
    })
    //this.isMember = await this.contract.is_member(this.accountId)
    await this.update()
  }

  async update(): Promise<void> {
    await this.updateBalance()
    await this.updateMemberList()
    await this.updateProposalList()
  }

  async updateBalance(): Promise<void> {
    this.balance = Math.floor(parseFloat(formatNearAmount(await this.contract.balance())) * 100) / 100
  }

  async updateMemberList(): Promise<void> {
    this.memberList = await this.contract.member_list()
  }

  async updateProposalList(): Promise<void> {
    this.proposalList = await this.contract.proposal_list()
    const list = await this.contract.proposal_list()
    this.proposalOfAccountList = await Promise.all(
      list.map(async (proposal) => {
        const out = <ProposalOfAccount>{
          proposal
        }
        out.canVote = this.accountId ? await this.contract.cat_vote(proposal.id, this.accountId) : false
        return out
      })
    )
  }

  async voteReject(proposal_id: number): Promise<void> {
    await this.contract.vote_reject(
      {
        proposal_id,
      },
      '300000000000000',
      '0',
    )
  }

  async voteApprove(proposal_id: number): Promise<void> {
    await this.contract.vote_approve(
      {
        proposal_id,
      },
      '300000000000000',
      '0',
    )
  }

  addMemberProposal(title: string, description: string): Promise<number> {
    return this.contract.add_member_proposal(
      {
        title,
        description,
      },
      '300000000000000',
      '6000000000000000000000',
    )
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
