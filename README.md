# CryptoArt DAO

## Dev deploy
```shell
yarn contract-qa
yarn contract-build
yarn contract-dev-deploy && contractId=$(cat neardev/dev-account) && near --accountId $contractId call $contractId init "{\"initial_members\": [\"$contractId\"]}"
near delete foo.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete bar.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete $(cat neardev/dev-account) $NEAR_DEV_ACCOUNT && rm -fr neardev
echo "export default '$contractId'" > src/contract-name.ts
yarn start
```

## Contract interact
```shell
npx near login
contractId=$(cat neardev/dev-account)
near state $contractId
near view $contractId balance
near view $contractId member_list
near view $contractId proposal_list
near view $contractId can_vote "{\"proposal_id\":0,\"account_id\": \"$contractId\"}"
near --accountId $contractId call $contractId vote_approve '{"proposal_id":0}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":0}'
near --masterAccount $contractId create-account "foo.$contractId" --initialBalance 10
near --accountId "foo.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}'
near --accountId "foo.$contractId" call $contractId add_member_proposal '{"title":"foo2", "description": "bar2"}'
near --accountId $contractId call $contractId vote_approve '{"proposal_id":1}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":1}'
near --accountId "foo.$contractId" call $contractId add_fund_proposal '{"title":"foo", "description": "bar", "script":"{\"fund\":\"10000000000000000000000000\"}"}'

near --accountId "bar.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}'
near --accountId "bar.$contractId" call $contractId add_member_proposal '{"title":"foo2", "description": "bar2"}'
near --accountId $contractId call $contractId vote_approve '{"proposal_id":2}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":2}'
near --accountId "bar.$contractId" call $contractId add_fund_proposal '{"title":"foo", "description": "bar", "script":"{\"fund\":\"10000000000000000000000000\"}"}'

near --accountId $contractId call $contractId vote_approve '{"proposal_id":1}'
near --masterAccount $contractId create-account "bar.$contractId" --initialBalance 10
near --accountId "bar.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}'
near --accountId "foo.$contractId" call $contractId vote_approve '{"proposal_id":1}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":0}'

near --masterAccount $contractId create-account "quz.$contractId" --initialBalance 10
near --accountId "quz.$contractId" call $contractId add_member_proposal
near --accountId $contractId call $contractId vote_approve '{"proposal_id":3}'
near --accountId $contractId call $contractId vote_approve '{"proposal_id":4}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":3}'
near --accountId "quz.$contractId" call $contractId add_fund_proposal '{"title":"foo", "description": "bar", "script":"{\"fund\":\"10000000000000000000000000\"}"}'
near --accountId "bar.$contractId" call $contractId add_fund_proposal '{"title":"foo", "description": "bar", "script":"{\"fund\":\"10000000000000000000000000\"}"}'
```

## Deploy
```shell
contractId=cryptoartdao.testnet
near state $contractId
# QA
yarn contract-qa
# Send found if need
near send $NEAR_DEV_ACCOUNT $contractId 1000
# Deploy contract
yarn contract-build
near deploy $contractId build/society-minified.wasm init '{"initial_members": ["%near_account_id%"]}'
# Deploy app
echo "export default '$contractId'" > src/contract-name.ts
yarn deploy:app
```
