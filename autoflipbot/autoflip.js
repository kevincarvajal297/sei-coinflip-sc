import { StargateClient, setupGovExtension, QueryClient } from "@cosmjs/stargate"
import { HttpClient, Tendermint34Client } from "@cosmjs/tendermint-rpc"
import * as fs from "fs"
import { toBase64, toUtf8 } from '@cosmjs/encoding';
import axios from 'axios';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { GasPrice } from '@cosmjs/stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import moment from "moment"
import { exec } from 'child_process';
import { XMLHttpRequest } from 'xmlhttprequest';
import { coin } from '@cosmjs/launchpad'
// import { UTF8 } from 'utf-8';

import pkg from 'utf8';
const { encode, decode } = pkg;

// const utf8 = import('utf8');

// const RPC = "https://rpc-juno.itastakers.com:443";
const RPC = "http://localhost:26657";

const config = {
    endpoint: RPC,
    bech32prefix: 'juno',
    feeDenom: 'ujunox',
    gasPrice: GasPrice.fromString('0.01ujunox'),
    mnemonic: 'clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose',
};

async function setup() {
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(config.mnemonic, {
        prefix: config.bech32prefix,
    });
    const { address } = (await wallet.getAccounts())[0];
    const options = {
        prefix: config.bech32prefix,
        gasPrice: config.gasPrice,
    };
    const client = await SigningCosmWasmClient.connectWithSigner(
        config.endpoint,
        wallet,
        options
    );

    // now ensure there is a balance
    console.log(`Querying balance of ${address}`);
    const {denom, amount} = await client.getBalance(address, config.feeDenom);
    console.log(`Got ${amount} ${denom}`);
    if (!amount || amount === "0") {
        console.log('Please add tokens to your account before uploading')
    }
  
    return { address, client };
}

const contractAddr = "juno14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9skjuwg8"

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

const { address, client } = await setup();

async function main() {
    
    for (let i = 0; i < 200; i ++) {
        console.log(i)
        while (true) {
            try {
                await client.execute(
                    address,
                    contractAddr,
                    { 
                        flip: {
                            level: 1
                        }
                    },
                    'auto',
                    '',
                    [coin(1000000, 'ujunox')]
                );
                break;
            } catch (error) {
                continue;
            }
        }
    }
    
    
    
    
}


await main()

