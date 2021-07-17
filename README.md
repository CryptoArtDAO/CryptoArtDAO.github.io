# CryptoArt DAO

```shell
yarn contract-qa
yarn contract-dev-deploy && contractId=$(cat neardev/dev-account) && near --accountId $contractId call $contractId new
contractId=$(cat neardev/dev-account)
near --accountId $contractId deploy $contractId build/society.wasm
near state $contractId
near view $contractId balance
near view $contractId member_list
near --accountId $contractId call $contractId set_greeting '{"message": "12345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890"}'
near --accountId $contractId call $contractId balance
npx near --accountId ilyar.testnet call dev-1626538660125-68644668973822 add_member_proposal '{"title":"all 42"}' --deposit 0.004

near delete $(cat neardev/dev-account) ilyar.testnet && rm -fr neardev
yarn contract-build
yarn start
```
