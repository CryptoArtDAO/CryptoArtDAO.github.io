# CryptoArt DAO

## Dev deploy
```shell
yarn contract-qa
yarn contract-build
yarn contract-dev-deploy && contractId=$(cat neardev/dev-account) && near --accountId $contractId call $contractId init
near delete foo.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete bar.$(cat neardev/dev-account) $NEAR_DEV_ACCOUNT
near delete $(cat neardev/dev-account) $NEAR_DEV_ACCOUNT && rm -fr neardev
yarn start
```

## Contract interact
```shell
npx near login
contractId=$(cat neardev/dev-account)
contractId=cryptoartdao.testnet
near state $contractId
near view $contractId balance
near view $contractId member_list
near view $contractId proposal_list
near view $contractId can_vote '{"proposal_id":0,"account_id": "dev-1626575883917-97357653463081"}'
near --masterAccount $contractId create-account "foo.$contractId" --initialBalance 10
near --accountId "foo.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}' --deposit 0.006
near --accountId $contractId call $contractId vote_approve '{"proposal_id":0}'
near --masterAccount $contractId create-account "bar.$contractId" --initialBalance 10
near --accountId "bar.$contractId" call $contractId add_member_proposal '{"title":"foo", "description": "bar"}' --deposit 0.006
near --accountId $contractId call $contractId vote_approve '{"proposal_id":1}'
near --accountId "foo.$contractId" call $contractId vote_approve '{"proposal_id":1}'
near --accountId $contractId call $contractId vote_reject '{"proposal_id":0}'
```

## Deploy
```shell
yarn contract-qa
yarn contract-build
near deploy cryptoartdao.testnet build/society-minified.wasm init '{}'
echo 'export default "cryptoartdao.testnet"' > src/contract-name.ts
yarn build --output-path dist --base-href / --deploy-url https://cryptoartdao.github.io
cd dist
git init && git add . && git commit -m '1.0.0-beta.0' 
git remote add origin git@github.com:CryptoArtDAO/CryptoArtDAO.github.io.git
git branch -m gh-pages && git push origin +gh-pages
mkdir -p dist && cd $_
yarn build --output-path public --base-href / --deploy-url https://cryptoartdao.gitlab.io
git remote add origin git@gitlab.com:cryptoartdao/cryptoartdao.gitlab.io.git
git branch -m main && git push origin +main
```
