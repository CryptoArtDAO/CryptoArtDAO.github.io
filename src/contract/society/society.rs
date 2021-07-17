use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::{env, AccountId};
use std::collections::HashMap;

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    MemberList,
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Society {
    member_list: UnorderedSet<AccountId>,
    records: HashMap<String, String>,
    balance: u128,
}

#[near_bindgen]
impl Society {
    #[init]
    pub fn new() -> Self {
        Self {
            member_list: UnorderedSet::new(StorageKey::MemberList),
            records: Default::default(),
            balance: 0,
        }
    }

    pub fn balance(self) -> U128 {
        U128(env::account_balance() - env::account_locked_balance())
    }

    pub fn member_list(self, offset: Option<u64>, limit: Option<u64>) -> Vec<AccountId> {
        let limit = limit.map(|v| v as usize).unwrap_or(usize::MAX);
        assert_ne!(limit, 0, "Cannot provide limit of 0.");
        let start_index: u128 = offset.map(From::from).unwrap_or_default();
        assert!(
            self.member_list.len() as u128 > start_index,
            "Out of bounds, please use a smaller from_index."
        );
        self.member_list
            .iter()
            .skip(start_index as usize)
            .take(limit)
            .collect()
    }

    pub fn set_greeting(&mut self, message: String) {
        let account_id = env::signer_account_id();

        // Use env::log to record logs permanently to the blockchain!
        env::log(format!("Saving greeting '{}' for account '{}'", message, account_id,).as_bytes());

        self.records.insert(account_id, message);
        self.balance = env::account_balance() - env::account_locked_balance()
    }

    // `match` is similar to `switch` in other languages; here we use it to default to "Hello" if
    // self.records.get(&account_id) is not yet defined.
    // Learn more: https://doc.rust-lang.org/book/ch06-02-match.html#matching-with-optiont
    pub fn get_greeting(&self, account_id: String) -> &str {
        match self.records.get(&account_id) {
            Some(greeting) => greeting,
            None => "Hello",
        }
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

    pub fn new_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn set_then_get_greeting() {
        let context = new_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Society::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting(accounts(1).to_string())
        );
    }

    #[test]
    fn get_default_greeting() {
        let mut context = new_context(accounts(0));
        testing_env!(context.is_view(true).build());
        let contract = Society::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }
}
