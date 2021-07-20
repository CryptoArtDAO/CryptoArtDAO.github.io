import {Injectable,} from '@angular/core';
import getConfig from '../../config'
import {connect, Contract, keyStores, WalletConnection,} from 'near-api-js'
import {formatNearAmount,} from "near-api-js/lib/utils/format";

const nearConfig = getConfig('development') // TODO use process.env.NODE_ENV || 'development'

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

  is_member(param: Object): Promise<boolean>

  can_vote(param: Object): Promise<boolean>

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
  memberList: string[] = []
  proposalList: ProposalOfAccount[] = []
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
        'can_vote',
      ],
      changeMethods: [
        'add_member_proposal',
        'vote_approve',
        'vote_reject',
      ],
    })
    await this.update()
  }

  async update(): Promise<void> {
    this.isMember = await this.updateStatusMember(this.accountId)
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
    // FIXME it need refactor
    const list = await this.contract.proposal_list()
    this.proposalList = await Promise.all(
      list.map(async (proposal) => {
        const out = <ProposalOfAccount>{
          proposal: await proposal
        }
        out.canVote = false
        if (this.accountId) {
          const catVote = await this.canVote(proposal.id, this.accountId)
          out.canVote = catVote && this.isMember
        }
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

  async updateStatusMember(account_id: string): Promise<boolean> {
    return await this.contract.is_member(
      {
        account_id,
      }
    )
  }

  async canVote(proposal_id: number, account_id: string): Promise<boolean> {
    return await this.contract.can_vote(
      {
        proposal_id,
        account_id,
      }
    )
  }

  addMemberProposal(title: string, description: string): Promise<number> {
    return this.contract.add_member_proposal(
      {
        title,
        description,
      },
      '5000000000000', // 5 TGas
      '14000000000000000000000', // 0.014 NEAR
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
