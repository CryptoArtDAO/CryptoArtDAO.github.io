#[test]
fn vote() {
    let (contract, list) = deploy();
    let result = call(
        &contract,
        &list[0],
        "add_member_proposal",
        json!({
            "title": "a".repeat(170),
            "description": "a".repeat(1000),
        }),
        to_yocto("0.01254"), // deposit
    );
    assert_burnt_gas("deploy_1", &result, "7.5", None);
    let actual: Vec<AccountId> = contract
        .view(contract.account_id(), "member_list", &args(json!({})))
        .unwrap_json();
    assert_eq!(1, actual.len());
    // TODO simulate voting flow
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
use near_sdk::serde::de::DeserializeOwned;
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
    contract.call(
        contract.account_id(),
        "init",
        &json!({}).to_string().into_bytes(),
        DEFAULT_GAS,
        0,
    );
    (contract, list)
}

pub fn init() -> (UserAccount, Vec<UserAccount>) {
    let root = init_simulator(None);
    let mut list = vec![
        root.create_user(account_id("alice"), to_yocto("100")),
        root.create_user(account_id("bob"), to_yocto("100")),
        root.create_user(account_id("carol"), to_yocto("100")),
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
