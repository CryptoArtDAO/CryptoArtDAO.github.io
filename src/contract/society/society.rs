use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
use near_sdk::json_types::ValidAccountId;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::serde::Deserialize;
use near_sdk::serde::Serialize;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::CryptoHash;
use near_sdk::PanicOnDefault;
use near_sdk::Promise;
use std::option::Option;

near_sdk::setup_alloc!();

// Param of Protocol contract
const PARAM_VOTE_TARGET: f64 = 0.50; // 50%
const PARAM_TIME_LOCK: u64 = 10 * 60 * 1_000_000_000; // 10m in nanoseconds
const PARAM_FUND_RESERVE: Balance = 10_000_000_000_000_000_000_000_000; // reserve is 10NEAR

fn consensus(max: u64, quorum: u64) -> bool {
    let target = (max as f64 * PARAM_VOTE_TARGET).floor() as u64 + 1;
    quorum >= target
}

pub fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Must attach {} yoctoNEAR to cover storage",
        required_cost,
    );

    let refund = attached_deposit - required_cost;
    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

pub fn hash(data: String) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(data.as_bytes()));
    hash
}

#[derive(Deserialize)]
struct FundScript {
    fund: U128,
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    MemberRequest,
    FundRequest,
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    Draft,
    Vote,
    Accepted,
    Rejected,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalVote {
    approve: u64,
    reject: u64,
}

impl ProposalVote {
    pub fn is_approve(&self) -> bool {
        self.approve > self.reject
    }

    pub fn is_parte(&self) -> bool {
        self.approve == self.reject
    }

    pub fn is_reject(&self) -> bool {
        self.approve < self.reject
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    id: u64,
    timestamp: u64,
    title: String,
    kind: ProposalKind,
    status: ProposalStatus,
    description: String,
    script: Option<String>,
    author: AccountId,
    vote: ProposalVote,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalState {
    timestamp: u64,
    title: String,
    kind: ProposalKind,
    status: ProposalStatus,
    description: String,
    script: Option<String>,
    author: AccountId,
    vote: ProposalVote,
}

impl ProposalState {
    pub fn new(
        title: Option<String>,
        description: Option<String>,
        author: AccountId,
        kind: ProposalKind,
        status: ProposalStatus,
        script: Option<String>,
    ) -> Self {
        let title = title.unwrap_or_default();
        if title.len() > 170 {
            env::panic(b"Field title mus be less 70 lenght")
        }
        let description = description.unwrap_or_default();
        if description.len() > 1000 {
            env::panic(b"Field description mus be less 1000 lenght")
        }
        Self {
            timestamp: env::block_timestamp(),
            title,
            kind,
            status,
            description,
            script,
            author,
            vote: ProposalVote {
                approve: 0,
                reject: 0,
            },
        }
    }

    fn is_draft(&self) -> bool {
        self.status == ProposalStatus::Draft
    }

    fn is_accepted(&self) -> bool {
        self.status == ProposalStatus::Accepted
    }

    fn vote(&mut self, resolve: bool, max: u64) {
        if resolve {
            self.vote.approve += 1;
        } else {
            self.vote.reject += 1;
        }
        self.calc(max);
    }

    fn calc(&mut self, total: u64) {
        if !self.consensus(total) {
            return;
        }
        if self.vote.is_approve() {
            self.status = ProposalStatus::Accepted;
        } else if self.vote.is_reject() {
            self.status = ProposalStatus::Rejected;
        } else if self.quorum() == total {
            self.status = ProposalStatus::Draft;
            self.vote.reject = 0;
            self.vote.approve = 0;
        }
    }

    fn consensus(&self, max: u64) -> bool {
        consensus(max, self.quorum())
    }

    fn quorum(&self) -> u64 {
        self.vote.approve + self.vote.reject
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    MemberList,
    ProposalList,
    ProposalVote { hash: CryptoHash },
    VoteList,
    ActiveProposal,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Society {
    member_list: UnorderedSet<AccountId>,
    proposal_list: Vector<ProposalState>,
    vote_list: LookupMap<u64, UnorderedSet<AccountId>>,
    active_proposal: LookupMap<AccountId, u64>,
    fund_proposal: Balance,
}

#[near_bindgen]
impl Society {
    #[init]
    pub fn init(initial_members: Vec<ValidAccountId>) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        assert!(
            !initial_members.is_empty(),
            "Need minimum one initial member"
        );
        let mut contract = Self::new();
        contract.setup(initial_members);
        contract
    }

    fn new() -> Self {
        Self {
            member_list: UnorderedSet::new(StorageKey::MemberList),
            proposal_list: Vector::new(StorageKey::ProposalList),
            vote_list: LookupMap::new(StorageKey::VoteList),
            active_proposal: LookupMap::new(StorageKey::ActiveProposal),
            fund_proposal: 0,
        }
    }

    fn setup(&mut self, initial_members: Vec<ValidAccountId>) {
        for member in initial_members {
            self.add_member(member.into());
        }
    }

    fn account_locked_for_storage(&self) -> u128 {
        env::storage_byte_cost() * Balance::from(env::storage_usage())
    }

    fn fund(&self) -> Balance {
        env::account_balance()
            - env::account_locked_balance()
            - self.account_locked_for_storage()
            - PARAM_FUND_RESERVE
            - self.fund_proposal
    }

    pub fn balance(&self) -> U128 {
        U128(self.fund())
    }

    pub fn is_member(&self, account_id: AccountId) -> bool {
        self.member_list.contains(&account_id)
    }

    fn add_member(&mut self, account_id: AccountId) -> bool {
        self.assert_is_member(account_id.clone());
        self.member_list.insert(&account_id)
    }

    fn vote_total(&self) -> u64 {
        self.member_list.len()
    }

    pub fn can_vote(self, proposal_id: u64, account_id: AccountId) -> bool {
        match self.vote_list.get(&proposal_id) {
            Some(vote_list) => !vote_list.contains(&account_id),
            None => true,
        }
    }

    pub fn vote_approve(&mut self, proposal_id: u64) {
        self.vote(proposal_id, true)
    }

    pub fn vote_reject(&mut self, proposal_id: u64) {
        self.vote(proposal_id, false)
    }

    fn vote(&mut self, proposal_id: u64, resolve: bool) {
        let signer_account_id = env::signer_account_id();
        assert!(
            self.is_member(signer_account_id.clone()),
            "Only for members"
        );
        let mut proposal = match self.proposal_list.get(proposal_id) {
            Some(proposal) => proposal,
            None => env::panic(b"Proposal not found"),
        };
        let mut vote_list = match self.vote_list.get(&proposal_id) {
            Some(vote_list) => vote_list,
            None => UnorderedSet::new(StorageKey::ProposalVote {
                hash: hash(format!("{}{}", proposal_id, proposal.author)),
            }),
        };
        if vote_list.contains(&signer_account_id) {
            env::panic(b"You are already voted")
        }
        proposal.vote(resolve, self.vote_total());
        if proposal.is_draft() {
            vote_list = UnorderedSet::new(StorageKey::ProposalVote {
                hash: hash(format!("{}{}", proposal_id, proposal.author)),
            });
            if proposal.kind == ProposalKind::FundRequest {
                let script = proposal.script.clone();
                let fund_script: FundScript =
                    serde_json::from_str(&script.unwrap_or_else(|| "{}".to_string())).unwrap();
                let request_fund = u128::from(fund_script.fund);
                self.fund_proposal -= request_fund;
            };
        } else {
            vote_list.insert(&signer_account_id);
        }
        self.vote_list.insert(&proposal_id, &vote_list);
        if proposal.is_accepted() {
            self.active_proposal.remove(&proposal.author);
            if proposal.kind == ProposalKind::MemberRequest {
                self.add_member(proposal.author.clone());
            };
            if proposal.kind == ProposalKind::FundRequest {
                let script = proposal.script.clone();
                let fund_script: FundScript =
                    serde_json::from_str(&script.unwrap_or_else(|| "{}".to_string())).unwrap();
                let request_fund = u128::from(fund_script.fund);
                self.fund_proposal -= request_fund;
                Promise::new(proposal.author.clone()).transfer(request_fund);
            };
        };
        self.proposal_list.replace(proposal_id, &proposal);
    }

    pub fn add_member_proposal(
        &mut self,
        title: Option<String>,
        description: Option<String>,
    ) -> u64 {
        let signer_account_id = env::signer_account_id();
        self.assert_is_member(signer_account_id.clone());
        self.add_proposal(
            signer_account_id,
            ProposalKind::MemberRequest,
            ProposalStatus::Vote,
            title,
            description,
            None,
        )
    }

    pub fn add_fund_proposal(&mut self, title: String, description: String, script: String) -> u64 {
        let signer_account_id = env::signer_account_id();
        assert!(
            self.is_member(signer_account_id.clone()),
            "Only for members"
        );
        let fund_script: FundScript = serde_json::from_str(&script).unwrap();
        let request_fund = u128::from(fund_script.fund);
        // TODO move into method
        let curren_fund = env::account_balance()
            - env::account_locked_balance()
            - env::storage_byte_cost() * Balance::from(env::storage_usage())
            - PARAM_FUND_RESERVE
            - self.fund_proposal;
        if request_fund >= curren_fund {
            env::panic(b"The fund does not have so many resources")
        };
        self.fund_proposal += request_fund;
        self.add_proposal(
            signer_account_id,
            ProposalKind::FundRequest,
            ProposalStatus::Vote,
            Some(title),
            Some(description),
            Some(script),
        )
    }

    fn assert_is_member(&mut self, account_id: AccountId) {
        if self.member_list.contains(&account_id) {
            env::panic(format!("Account {} already is member", account_id).as_bytes())
        }
    }

    fn add_proposal(
        &mut self,
        author: AccountId,
        kind: ProposalKind,
        status: ProposalStatus,
        title: Option<String>,
        description: Option<String>,
        script: Option<String>,
    ) -> u64 {
        match self.active_proposal.get(&author) {
            Some(proposal_id) => {
                let proposal = match self.proposal_list.get(proposal_id) {
                    Some(proposal) => proposal,
                    None => env::panic(format!("Proposal {} not active", proposal_id).as_bytes()),
                };
                if proposal.timestamp + PARAM_TIME_LOCK >= env::block_timestamp() {
                    env::panic(format!("Proposal {} is locked try late", proposal_id).as_bytes())
                }
                if proposal.status != ProposalStatus::Draft {
                    env::panic(
                        format!(
                            "You can update the proposal {} only in the draft status",
                            proposal_id,
                        )
                        .as_bytes(),
                    )
                }
                self.proposal_list.replace(
                    proposal_id,
                    &ProposalState::new(title, description, author, kind, status, script),
                );
                proposal_id
            }
            None => {
                let proposal_id = self.proposal_list.len();
                self.active_proposal.insert(&author, &proposal_id);
                self.proposal_list.push(&ProposalState::new(
                    title,
                    description,
                    author,
                    kind,
                    status,
                    script,
                ));
                proposal_id
            }
        }
    }

    pub fn member_list(self, offset: Option<u64>, limit: Option<u64>) -> Vec<AccountId> {
        let limit = limit.unwrap_or(100);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u64 = offset.unwrap_or(0);
        assert!(
            self.member_list.len() > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        self.member_list
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .collect()
    }

    pub fn proposal_list(self, offset: Option<u64>, limit: Option<u64>) -> Vec<Proposal> {
        let limit = limit.unwrap_or(100);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u64 = offset.unwrap_or(0);
        assert!(
            self.member_list.len() > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        let mut id = start_index;
        let mut result = vec![];
        for state in self.proposal_list.iter().skip(start_index as usize) {
            result.push(Proposal {
                id,
                timestamp: state.timestamp,
                title: state.title,
                kind: state.kind,
                status: state.status,
                description: state.description,
                script: state.script,
                author: state.author,
                vote: state.vote,
            });
            id += 1
        }
        result
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use near_sdk::json_types::ValidAccountId;
    use near_sdk::test_utils::accounts;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;
    use std::convert::TryFrom;

    pub fn new_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    pub fn new_contract() -> Society {
        Society::init(vec![accounts(1)])
    }

    #[test]
    fn consensus_cases() {
        assert!(consensus(1, 1));
        assert_eq!(!consensus(2, 1), consensus(2, 2));
        assert_eq!(!consensus(3, 1), consensus(3, 2));
        assert_eq!(!consensus(4, 2), consensus(4, 3));
        assert_eq!(!consensus(5, 2), consensus(5, 3));
        assert_eq!(!consensus(6, 3), consensus(6, 4));
        assert_eq!(!consensus(7, 3), consensus(7, 4));
        assert_eq!(!consensus(8, 4), consensus(8, 5));
        assert_eq!(!consensus(9, 4), consensus(9, 5));
        assert_eq!(!consensus(10, 5), consensus(10, 6));
        assert_eq!(!consensus(11, 5), consensus(11, 6));
        assert_eq!(!consensus(12, 6), consensus(12, 7));
        assert_eq!(!consensus(13, 6), consensus(13, 7));
    }

    #[test]
    fn balance() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = new_contract();
        assert_eq!(U128(86926860000000000000000000), contract.balance());
    }

    #[test]
    fn member_list() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = new_contract();
        assert_eq!(
            vec![accounts(1).into()] as Vec<AccountId>,
            contract.member_list(None, None)
        );
    }

    #[test]
    fn is_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = new_contract();
        assert!(contract.is_member(accounts(1).into()));
    }

    #[test]
    fn add_fund_proposal() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = new_contract();
        let proposal_id = contract.add_fund_proposal(
            "a".to_string(),
            "b".to_string(),
            "{\"fund\": \"1000000000000000000000000\"}".to_string(),
        );
        assert_eq!(0, proposal_id);
    }

    #[test]
    fn is_not_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = new_contract();
        assert!(!contract.is_member(accounts(2).into()));
    }

    #[test]
    fn proposal_list_empty() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = new_contract();
        assert_eq!(0, contract.proposal_list(None, None).len());
    }

    #[test]
    #[should_panic(expected = "Account bob already is member")]
    fn add_member_proposal_for_exist() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = new_contract();
        contract.add_member_proposal(None, None);
    }

    #[test]
    fn add_member_proposal() {
        let mut context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = new_contract();
        testing_env!(context
            .signer_account_id(ValidAccountId::try_from("a".repeat(64)).unwrap())
            .build());
        assert_eq!(
            0,
            contract.add_member_proposal(Some("a".repeat(170)), Some("a".repeat(1000)))
        );
        assert_eq!(1, contract.proposal_list(None, None).len());
    }
}
#[cfg(test)]
mod society_simulator;
