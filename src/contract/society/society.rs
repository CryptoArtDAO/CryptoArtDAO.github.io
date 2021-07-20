use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::Vector;
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
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

near_sdk::setup_alloc!();

const VOTE_TARGET: f64 = 0.50; // 50%
const MINT_STORAGE_COST: u128 = 14000000000000000000000; // 0.014 NEAR

fn consensus(max: u64, quorum: u64) -> bool {
    let target = (max as f64 * VOTE_TARGET).floor() as u64 + 1;
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

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    MemberRequest,
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
    title: String,
    kind: ProposalKind,
    status: ProposalStatus,
    description: String,
    author: AccountId,
    vote: ProposalVote,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalState {
    title: String,
    kind: ProposalKind,
    status: ProposalStatus,
    description: String,
    author: AccountId,
    vote: ProposalVote,
}

impl ProposalState {
    pub fn new(
        title: String,
        description: String,
        author: AccountId,
        kind: ProposalKind,
        status: ProposalStatus,
    ) -> Self {
        Self {
            title,
            kind,
            status,
            description,
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
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Society {
    member_list: UnorderedSet<AccountId>,
    proposal_list: Vector<ProposalState>,
    vote_list: LookupMap<u64, UnorderedSet<AccountId>>,
}

#[near_bindgen]
impl Society {
    #[init]
    pub fn init() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Self::new();
        contract.setup();
        contract
    }

    #[init(ignore_state)]
    pub fn migrate() -> Self {
        assert!(Self::is_self(), "Private function");
        assert!(env::state_exists(), "State doesn't exist");
        let mut state: Society = env::state_read().expect("State doesn't exist");
        env::log(b"clearing");
        env::log(format!("current.proposal_list.len()={}", state.proposal_list.len()).as_bytes());
        for i in 0..state.proposal_list.len() {
            env::log(format!("vote_list.remove({})", &i).as_bytes());
            state.vote_list.remove(&i);
        }
        env::log(format!("proposal_list.clear() - {}", state.proposal_list.len()).as_bytes());
        state.proposal_list.clear();
        env::log(format!("member_list.clear() - {}", state.member_list.len()).as_bytes());
        state.member_list.clear();
        let mut contract = Self::new();
        contract.setup();
        contract
    }

    fn is_self() -> bool {
        env::predecessor_account_id() == env::current_account_id()
    }

    fn setup(&mut self) {
        self.add_member(env::signer_account_id());
    }

    fn new() -> Self {
        Self {
            member_list: UnorderedSet::new(StorageKey::MemberList),
            proposal_list: Vector::new(StorageKey::ProposalList),
            vote_list: LookupMap::new(StorageKey::VoteList),
        }
    }

    fn account_locked_for_storage(self) -> u128 {
        env::storage_byte_cost() * Balance::from(env::storage_usage())
    }

    pub fn balance(self) -> U128 {
        U128(
            env::account_balance()
                - env::account_locked_balance()
                - self.account_locked_for_storage(),
        )
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
            })
        } else {
            vote_list.insert(&signer_account_id);
        }
        self.vote_list.insert(&proposal_id, &vote_list);
        if proposal.is_accepted() {
            self.add_member(proposal.author.clone());
        }
        self.proposal_list.replace(proposal_id, &proposal);
    }

    #[payable]
    pub fn add_member_proposal(
        &mut self,
        title: Option<String>,
        description: Option<String>,
    ) -> u64 {
        if env::attached_deposit() < MINT_STORAGE_COST {
            env::panic(b"Need attach minimum 0.04 NEAR for cover storage")
        }
        let initial_storage_usage = env::storage_usage();
        let signer_account_id = env::signer_account_id();
        self.assert_is_member(signer_account_id.clone());
        let proposal_id = self.add_proposal(
            signer_account_id,
            ProposalKind::MemberRequest,
            ProposalStatus::Vote,
            title,
            description,
        );
        refund_deposit(env::storage_usage() - initial_storage_usage);
        proposal_id
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
    ) -> u64 {
        let title = title.unwrap_or_default();
        if title.len() > 170 {
            env::panic(b"Field title mus be less 70 lenght")
        }
        let description = description.unwrap_or_default();
        if description.len() > 1000 {
            env::panic(b"Field description mus be less 1000 lenght")
        }
        self.proposal_list.push(&ProposalState::new(
            title,
            description,
            author,
            kind,
            status,
        ));
        self.proposal_list.len() - 1
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
                title: state.title,
                kind: state.kind,
                status: state.status,
                description: state.description,
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
        let contract = Society::init();
        assert_eq!(U128(96926860000000000000000000), contract.balance());
    }

    #[test]
    fn member_list() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::init();
        assert_eq!(
            vec![accounts(1).into()] as Vec<AccountId>,
            contract.member_list(None, None)
        );
    }

    #[test]
    fn is_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::init();
        assert!(contract.is_member(accounts(1).into()));
    }

    #[test]
    fn is_not_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::init();
        assert!(!contract.is_member(accounts(2).into()));
    }

    #[test]
    fn proposal_list_empty() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::init();
        assert_eq!(0, contract.proposal_list(None, None).len());
    }

    #[test]
    #[should_panic(expected = "Account bob already is member")]
    fn add_member_proposal_for_exist() {
        let mut context = new_context(accounts(1));
        testing_env!(context.attached_deposit(MINT_STORAGE_COST).build());
        let mut contract = Society::init();
        contract.add_member_proposal(None, None);
    }

    #[test]
    fn add_member_proposal() {
        let mut context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Society::init();
        testing_env!(context
            .signer_account_id(ValidAccountId::try_from("a".repeat(64)).unwrap())
            .attached_deposit(MINT_STORAGE_COST)
            .build());
        assert_eq!(
            0,
            contract.add_member_proposal(Some("a".repeat(170)), Some("a".repeat(1000)))
        );
        assert_eq!(1, contract.proposal_list(None, None).len());
    }

    #[test]
    #[should_panic(expected = "Need attach minimum 0.04 NEAR for cover storage")]
    fn add_member_proposal_without_cover_storage() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Society::init();
        contract.add_member_proposal(None, None);
    }
}
#[cfg(test)]
mod society_simulator;
