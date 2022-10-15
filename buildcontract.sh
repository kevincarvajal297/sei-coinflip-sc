#!/bin/bash

#Build Flag
NETWORK=mainnet
FUNCTION=$1
CATEGORY=$2
PARAM_1=$3
PARAM_2=$4
PARAM_3=$5

ADDR_VATE="juno18m24ayphdgpp39k8cgsksfsur53dggv7znv6jm"
ADDR_LOCAL="juno19380pt5d828stn7d9u4w53rz0zf55cl9xcfue0"

case $NETWORK in
  devnet)
    NODE="http://localhost:26657"
    DENOM=ujunox
    CHAIN_ID=testing
    LP_TOKEN_CODE_ID=1
    WALLET="--from local"
    ADDR_ADMIN=$ADDR_LOCAL
    GAS=0.01
    ;;
  testnet)
    NODE="https://rpc.juno.giansalex.dev:443"
    DENOM=ujunox
    CHAIN_ID=uni-3
    LP_TOKEN_CODE_ID=123
    WALLET="--from vate"
    ADDR_ADMIN=$ADDR_VATE
    GAS=0.01
    ;;
  mainnet)
    NODE="https://rpc-juno.itastakers.com:443"
    DENOM=ujuno
    CHAIN_ID=juno-1
    LP_TOKEN_CODE_ID=1
    WALLET="--from vate"
    ADDR_ADMIN=$ADDR_VATE
    GAS=0.001
    ;;
esac

##########################################################################################
#not depends
NODECHAIN=" --node $NODE --chain-id $CHAIN_ID"
TXFLAG=" $NODECHAIN --gas-prices $GAS$DENOM --gas auto --gas-adjustment 1.3"


RELEASE_DIR="release/"
INFO_DIR="info/"
INFONET_DIR=$INFO_DIR$NETWORK"/"
CODE_DIR=$INFONET_DIR"code/"
ADDRESS_DIR=$INFONET_DIR"address/"
CONTRACT_DIR="contracts/"

FC_COINFLIP=$CODE_DIR"coinflip.txt"

FILE_UPLOADHASH=$INFO_DIR"uploadtx.txt"

FA_COINFLIP=$ADDRESS_DIR"coinflip.txt"

[ ! -d $RELEASE_DIR ] && mkdir $RELEASE_DIR
[ ! -d $INFO_DIR ] &&mkdir $INFO_DIR
[ ! -d $INFONET_DIR ] &&mkdir $INFONET_DIR
[ ! -d $CODE_DIR ] &&mkdir $CODE_DIR
[ ! -d $ADDRESS_DIR ] &&mkdir $ADDRESS_DIR

###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################
#Environment Functions
CreateEnv() {
    sudo apt-get update && sudo apt upgrade -y
    sudo apt-get install make build-essential gcc git jq chrony -y
    wget https://golang.org/dl/go1.17.3.linux-amd64.tar.gz
    sudo tar -C /usr/local -xzf go1.17.3.linux-amd64.tar.gz
    rm -rf go1.17.3.linux-amd64.tar.gz

    export GOROOT=/usr/local/go
    export GOPATH=$HOME/go
    export GO111MODULE=on
    export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin
    
    rustup default stable
    rustup target add wasm32-unknown-unknown

    git clone https://github.com/CosmosContracts/juno
    cd juno
    git fetch
    git checkout v6.0.0
    make install
    cd ../
    rm -rf juno
}

RustBuild() {
    echo "================================================="
    echo "Rust Optimize Build Start"
    
    RUSTFLAGS='-C link-arg=-s' cargo wasm
    cp -f target/wasm32-unknown-unknown/release/*.wasm $RELEASE_DIR

    cargo schema
}

Upload() {
    echo "================================================="
    echo "Upload $CATEGORY"
    UPLOADTX=$(junod tx wasm store "$RELEASE_DIR$CATEGORY.wasm" $WALLET $TXFLAG --output json -y | jq -r '.txhash')
    
    echo "Upload txHash:"$UPLOADTX
    
    echo "================================================="
    echo "GetCode"
	CODE_ID=""
    while [[ $CODE_ID == "" ]]
    do 
        sleep 3
        CODE_ID=$(junod query tx $UPLOADTX $NODECHAIN --output json | jq -r '.logs[0].events[-1].attributes[0].value')
    done
    echo "Contract Code_id:"$CODE_ID

    #save to FILE_CODE_ID
    echo $CODE_ID > $CODE_DIR$CATEGORY".txt"
}

Instantiate() { 
    echo "================================================="
    echo "Instantiate Contract "$CATEGORY
    #read from FILE_CODE_ID
    CODE_ID=$(cat $CODE_DIR$CATEGORY".txt")
    echo "Code id: " $CODE_ID
if [[ $CATEGORY == "coinflip" ]]; then
    MSG='{"denom":{"native":"'$DENOM'"}}'
    LABEL=COINFLIP

fi
    # echo junod tx wasm instantiate $CODE_ID $MSG --label $LABEL --admin $ADDR_ADMIN $WALLET $TXFLAG -y --output json
    TXHASH=$(junod tx wasm instantiate $CODE_ID $MSG --label $LABEL --admin $ADDR_ADMIN $WALLET $TXFLAG -y --output json | jq -r '.txhash')
    echo $TXHASH
    CONTRACT_ADDR=""
    while [[ $CONTRACT_ADDR == "" ]]
    do
        sleep 2
        CONTRACT_ADDR=$(junod query tx $TXHASH $NODECHAIN --output json | jq -r '.logs[0].events[0].attributes[0].value')
    done
    echo $CONTRACT_ADDR
    echo $CONTRACT_ADDR > $ADDRESS_DIR$CATEGORY".txt"
}

###################################################################################################
###################################################################################################
###################################################################################################
###################################################################################################

Flip() {
    echo "================================================="
    echo "Do Flip"
    junod tx wasm execute $(cat $FA_COINFLIP) '{"flip":{"level":1}}' --amount 1000000$DENOM $WALLET $TXFLAG -y
}

RemoveTreasury() {
    echo "================================================="
    echo "Remove Treasury"
    junod tx wasm execute $(cat $FA_COINFLIP) '{"remove_treasury":{"amount":"1880000"}}' $WALLET $TXFLAG -y
}

Config() {
    junod query wasm contract-state smart $(cat $FA_COINFLIP) '{"config":{}}' $NODECHAIN
}

History() {
    # junod query wasm contract-state smart $(cat $FA_COINFLIP) '{"history":{"start_after":1}}' $NODECHAIN
    junod query wasm contract-state smart $(cat $FA_COINFLIP) '{"history":{}}' $NODECHAIN
}
#################################################################################
Balance() {
    echo "native balance"
    echo "========================================="
    junod query bank balances $ADDR_ADMIN $NODECHAIN
    junod query bank balances $(cat $FA_COINFLIP) $NODECHAIN
}

#################################### End of Function ###################################################
if [[ $FUNCTION == "" ]]; then
    RustBuild
    CATEGORY=coinflip
    Upload
    Instantiate
# sleep 3
#     Flip
# sleep 3
#     Config
else
    $FUNCTION
fi