import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { MARKETPLACE_ADDRESS, PREFIX, RPC } from "../constants.js";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { SeleneClient } from "../contract/Selene.client.js";
import { GasPrice } from "@cosmjs/stargate";

export async function makeWallet(mnemonic: string) {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: PREFIX,
  });
  const address = (await wallet.getAccounts())[0].address;
  const cosmwasmSigner = await SigningCosmWasmClient.connectWithSigner(
    RPC,
    wallet,
    {
      gasPrice: GasPrice.fromString("900000000000aconst"),
    }
  );

  const balance = await cosmwasmSigner.getBalance(address, "aconst")

  const seleneClient = new SeleneClient(
    cosmwasmSigner,
    address,
    MARKETPLACE_ADDRESS
  );

  return {
    wallet,
    address,
    cosmwasmSigner,
    seleneClient,
    balance
  };
}
