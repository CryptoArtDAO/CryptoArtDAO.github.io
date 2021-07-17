use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::collections::Vector;
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;
use near_sdk::Promise;
use serde::Serialize;

near_sdk::setup_alloc!();

const MINT_STORAGE_COST: u128 = 5000000000000000000000; // 0.005

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

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, Serialize)]
pub enum ProposalKind {
    MemberRequest,
}

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey, Serialize)]
pub enum ProposalStatus {
    Draft,
    Vote,
    Accepted,
    Rejected,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize)]
pub struct Proposal {
    title: String,
    kind: ProposalKind,
    status: ProposalStatus,
    description: String,
    author: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    MemberList,
    ProposalList,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Society {
    member_list: UnorderedSet<AccountId>,
    proposal_list: Vector<Proposal>,
}

#[near_bindgen]
impl Society {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Self {
            member_list: UnorderedSet::new(StorageKey::MemberList),
            proposal_list: Vector::new(StorageKey::ProposalList),
        };
        contract.add_member(env::signer_account_id());
        contract
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

    pub fn is_member(self, account_id: AccountId) -> bool {
        self.member_list.contains(&account_id)
    }

    fn add_member(&mut self, account_id: AccountId) -> bool {
        if self.member_list.contains(&account_id) {
            env::panic(b"Member exist")
        }
        self.member_list.insert(&account_id)
    }

    // pub fn add_member_some(&mut self, title: Option<String>, description: Option<String>) {
    //     let signer_account_id = env::signer_account_id();
    //     assert!(self.is_member(signer_account_id), "Only for members");
    // }

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
        if self.member_list.contains(&signer_account_id) {
            env::panic(b"Member exist")
        }
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
        let proposal = Proposal {
            title,
            kind,
            status,
            description,
            author,
        };
        self.proposal_list.push(&proposal);
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
        self.proposal_list
            .iter()
            .skip(start_index as usize)
            .take(limit as usize)
            .collect()
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
    fn balance() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::new();
        assert_eq!(U128(96926860000000000000000000), contract.balance());
    }

    #[test]
    fn member_list() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::new();
        assert_eq!(
            vec![accounts(1).into()] as Vec<AccountId>,
            contract.member_list(None, None)
        );
    }

    #[test]
    fn is_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::new();
        assert!(contract.is_member(accounts(1).into()));
    }

    #[test]
    fn is_not_member() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::new();
        assert!(!contract.is_member(accounts(2).into()));
    }

    #[test]
    fn proposal_list_empty() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let contract = Society::new();
        assert_eq!(0, contract.proposal_list(None, None).len());
    }

    #[test]
    #[should_panic(expected = "Member exist")]
    fn add_member_proposal_for_exist() {
        let mut context = new_context(accounts(1));
        testing_env!(context.attached_deposit(MINT_STORAGE_COST).build());
        let mut contract = Society::new();
        contract.add_member_proposal(None, None);
    }

    #[test]
    fn add_member_proposal() {
        let mut context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Society::new();
        testing_env!(context
            .signer_account_id(ValidAccountId::try_from("a".repeat(64)).unwrap())
            .attached_deposit(MINT_STORAGE_COST)
            .build());
        assert_eq!(
            0,
            contract.add_member_proposal(Some("a".repeat(170)), Some("a".repeat(170)),)
        );
        assert_eq!(1, contract.proposal_list(None, None).len());
    }

    #[test]
    #[should_panic(expected = "Need attach minimum 0.04 NEAR for cover storage")]
    fn add_member_proposal_whthout_cover_storage() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Society::new();
        contract.add_member_proposal(None, None);
    }
}
