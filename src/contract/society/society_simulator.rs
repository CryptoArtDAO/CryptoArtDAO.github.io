#[test]
fn vote() {
    let (contract, list) = create_member_proposal_for_all();
    let actual: Vec<Proposal> = contract
        .view(contract.account_id(), "proposal_list", &args(json!({})))
        .unwrap_json();
    assert_eq!(list.len(), actual.len());
    // 1st member added on deploy contract
    assert_eq!(1, member_total(&contract));

    // 1 exist members for 2nd member need 1 approve
    let proposal_id = 0;
    call_vote_approve(&contract, &contract, proposal_id);
    assert_eq!(2, member_total(&contract));

    // 2 exist members for 3rd member need 2 approve
    let proposal_id = 1;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    assert_eq!(3, member_total(&contract));

    // 3 exist members for 4th member need 2 approve
    let proposal_id = 2;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    assert_eq!(4, member_total(&contract));

    // 4 exist members for 5th member need 3 approve
    let proposal_id = 3;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    assert_eq!(5, member_total(&contract));

    // 5 exist members for 6th member need 3 approve
    let proposal_id = 4;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    assert_eq!(6, member_total(&contract));

    // 6 exist members for 7th member need 4 approve
    let proposal_id = 5;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    assert_eq!(7, member_total(&contract));

    // 7 exist members for 8th member need 4 approve
    let proposal_id = 6;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    assert_eq!(8, member_total(&contract));

    // 8 exist members for 9th member need 5 approve
    let proposal_id = 7;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    call_vote_approve(&contract, &list[3], proposal_id);
    assert_eq!(9, member_total(&contract));

    // 9 exist members for 10th member need 5 approve
    let proposal_id = 8;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    call_vote_approve(&contract, &list[3], proposal_id);
    assert_eq!(10, member_total(&contract));

    // 10 exist members for 11th member need 6 approve
    let proposal_id = 9;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    call_vote_approve(&contract, &list[3], proposal_id);
    call_vote_approve(&contract, &list[4], proposal_id);
    assert_eq!(11, member_total(&contract));

    // 11 exist members for 12th member need 6 approve
    let proposal_id = 10;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    call_vote_approve(&contract, &list[3], proposal_id);
    call_vote_approve(&contract, &list[4], proposal_id);
    assert_eq!(12, member_total(&contract));

    // 12 exist members for 13th member need 7 approve
    let proposal_id = 11;
    call_vote_approve(&contract, &contract, proposal_id);
    call_vote_approve(&contract, &list[0], proposal_id);
    call_vote_approve(&contract, &list[1], proposal_id);
    call_vote_approve(&contract, &list[2], proposal_id);
    call_vote_approve(&contract, &list[3], proposal_id);
    call_vote_approve(&contract, &list[4], proposal_id);
    call_vote_approve(&contract, &list[6], proposal_id);
    assert_eq!(13, member_total(&contract));
}

fn member_total(contract: &UserAccount) -> usize {
    let actual: Vec<AccountId> = contract
        .view(contract.account_id(), "member_list", &args(json!({})))
        .unwrap_json();
    actual.len()
}

fn call_vote_approve(contract: &UserAccount, signer: &UserAccount, proposal_id: u64) {
    let result = call(
        &contract,
        &signer,
        "vote_approve",
        json!({
            "proposal_id": proposal_id,
        }),
        0, // deposit
    );
    assert_eq!(
        0,
        result.promise_errors().len(),
        "got error: {:#?}",
        result.promise_errors()
    );
}

fn create_member_proposal_for_all() -> (UserAccount, Vec<UserAccount>) {
    let (contract, list) = deploy();
    for user in list.iter().as_ref() {
        let result = call(
            &contract,
            user,
            "add_member_proposal",
            json!({
                "title": "a".repeat(170),
                "description": "a".repeat(1000),
            }),
            0, // deposit
        );
        assert_eq!(
            0,
            result.promise_errors().len(),
            "got error: {:#?}",
            result.promise_errors()
        );
    }
    (contract, list)
}

#[test]
fn add_member_proposal() {
    let (contract, list) = deploy();
    let actual: Vec<Proposal> = contract
        .view(contract.account_id(), "proposal_list", &args(json!({})))
        .unwrap_json();
    assert_eq!(0, actual.len());
    let result = call(
        &contract,
        &list[0],
        "add_member_proposal",
        json!({
            "title": "a".repeat(170),
            "description": "a".repeat(1000),
        }),
        0, // deposit
    );
    assert_burnt_gas("add_member_proposal_1", &result, "4", None);
    let actual: Vec<Proposal> = contract
        .view(contract.account_id(), "proposal_list", &args(json!({})))
        .unwrap_json();
    assert_eq!(1, actual.len());
}

use near_sdk::serde_json::json;
use near_sdk::serde_json::Value;
use near_sdk::AccountId;
use near_sdk_sim::init_simulator;
use near_sdk_sim::lazy_static_include;
use near_sdk_sim::to_yocto;
use near_sdk_sim::ExecutionResult;
use near_sdk_sim::UserAccount;
use near_sdk_sim::DEFAULT_GAS;
use near_sdk_sim::STORAGE_AMOUNT;

pub fn account_id(name: &str) -> AccountId {
    name.to_string()
}

pub fn args(data: Value) -> Vec<u8> {
    json!(data).to_string().into_bytes()
}

use crate::Proposal;
use near_sdk::Balance;

pub fn call(
    contract: &UserAccount,
    signer: &UserAccount,
    method: &str,
    data: Value,
    deposit: Balance,
) -> ExecutionResult {
    signer.call(
        contract.account_id.clone(),
        method,
        &args(data),
        DEFAULT_GAS,
        deposit,
    )
}

pub fn deploy() -> (UserAccount, Vec<UserAccount>) {
    let (root, list) = init();
    lazy_static_include::lazy_static_include_bytes! {
        CONTRACT_WASM_BYTES => "../../../build/society-minified.wasm",
    }
    let contract = root.deploy(&CONTRACT_WASM_BYTES, account_id("contract"), STORAGE_AMOUNT);
    let result = contract.call(
        contract.account_id(),
        "init",
        &json!({ "initial_members": vec![contract.account_id()] })
            .to_string()
            .into_bytes(),
        DEFAULT_GAS,
        0,
    );
    assert_eq!(
        0,
        result.promise_errors().len(),
        "got error: {:#?}",
        result.promise_errors()
    );
    (contract, list)
}

pub fn init() -> (UserAccount, Vec<UserAccount>) {
    let root = init_simulator(None);
    let list = vec![
        root.create_user(account_id("alice"), to_yocto("100")),
        root.create_user(account_id("bob"), to_yocto("100")),
        root.create_user(account_id("carol"), to_yocto("100")),
        root.create_user(account_id("chuck"), to_yocto("100")),
        root.create_user(account_id("craig"), to_yocto("100")),
        root.create_user(account_id("dave"), to_yocto("100")),
        root.create_user(account_id("eve"), to_yocto("100")),
        root.create_user(account_id("mallory"), to_yocto("100")),
        root.create_user(account_id("peggy"), to_yocto("100")),
        root.create_user(account_id("trent"), to_yocto("100")),
        root.create_user(account_id("walter"), to_yocto("100")),
        root.create_user(account_id("arthur"), to_yocto("100")),
        root.create_user(account_id("paul"), to_yocto("100")),
    ];
    (root, list)
}

pub fn assert_burnt_gas(
    message: &str,
    result: &ExecutionResult,
    tera_gas: &str,
    has_error: Option<usize>,
) {
    print!("{}: ", message);
    assert_eq!(
        result.promise_errors().len(),
        has_error.unwrap_or_default(),
        "got error: {:#?}",
        result.promise_errors()
    );
    assert!(
        result.gas_burnt() <= to_gas(tera_gas),
        "gas burned {} less than or equal to {} expected",
        result.gas_burnt(),
        to_gas(tera_gas)
    );
    println!(
        "burnt tokens:{:.02} TeraGas diff: {:.04}",
        (result.gas_burnt()) as f64 / 1e12,
        (to_gas(tera_gas) - result.gas_burnt()) as f64 / 1e12,
    );
}

pub fn to_gas(tera_gas: &str) -> u64 {
    let part: Vec<_> = tera_gas.split('.').collect();
    let number = part[0].parse::<u64>().unwrap() * u64::pow(10, 12);
    if part.len() > 1 {
        let power = part[1].len() as u32;
        let mantissa = part[1].parse::<u64>().unwrap() * u64::pow(10, 12 - power);
        number + mantissa
    } else {
        number
    }
}
