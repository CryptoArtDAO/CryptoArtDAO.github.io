# CryptoArt DAO

```shell
yarn contract-qa
yarn contract-build
yarn contract-dev-deploy && contractId=$(cat neardev/dev-account) && near --accountId $contractId call $contractId init
near delete foo.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete bar.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete $(cat neardev/dev-account) $NEAR_DEV_ACCOUNT && rm -fr neardev
yarn start
```
export NEAR_DEV_ACCOUNT=cryptoartdao.testnet
```shell
npx near login
contractId=$(cat neardev/dev-account)
near state $contractId
near view $contractId balance
near view $contractId member_list
near view $contractId proposal_list
near view $contractId can_vote '{"proposal_id":0,"account_id": "dev-1626564038073-42420096352339"}'
near --masterAccount $contractId create-account "foo.$contractId" --initialBalance 10
near --accountId "foo.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}' --deposit 0.006
near --accountId $contractId call $contractId vote '{"proposal_id":1, "resolve":true}'
near --accountId $contractId call $contractId vote_approve '{"proposal_id":0}'
near --masterAccount $contractId create-account "bar.$contractId" --initialBalance 10
near --accountId "bar.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}' --deposit 0.006
near --accountId $contractId call $contractId vote_reject '{"proposal_id":0}'
```
