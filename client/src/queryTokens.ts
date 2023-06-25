import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { FAUCET_MNEMONIC, PREFIX, RPC } from "./constants.js";

// Faucet is a faucet for the cw20 tokens
export async function sendMeTokens(me: string) {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(FAUCET_MNEMONIC, {
    prefix: PREFIX,
  });
  const address = (await wallet.getAccounts())[0].address;
  console.log(address);
  const cosmwasmSigner = await SigningCosmWasmClient.connectWithSigner(
    RPC,
    wallet
  );

  //cosmwasmSigner

  // mint both tokens for the user
  me;
  cosmwasmSigner;
}
