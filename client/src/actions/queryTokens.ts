import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import {
  FAUCET_MNEMONIC,
  HEUR_ADDRESS,
  HUSD_ADDRESS,
  PREFIX,
  RPC,
} from "../constants.js";
import { GasPrice } from "@cosmjs/stargate";

// Faucet is a faucet for the cw20 tokens
export async function sendMeTokens(targetAddress: string) {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(FAUCET_MNEMONIC, {
    prefix: PREFIX,
  });
  const senderAddress = (await wallet.getAccounts())[0].address;
  // console.log(senderAddress);
  const cosmwasmSigner = await SigningCosmWasmClient.connectWithSigner(
    RPC,
    wallet,
    {
      gasPrice: GasPrice.fromString("900000000000aconst"),
    }
  );

  return cosmwasmSigner.executeMultiple(
    senderAddress,
    [
      {
        contractAddress: HEUR_ADDRESS,
        msg: {
          mint: {
            recipient: targetAddress,
            amount: (1_000 * Math.pow(10, 6)).toString(),
          },
        },
      },
      {
        contractAddress: HUSD_ADDRESS,
        msg: {
          mint: {
            recipient: targetAddress,
            amount: (1_000 * Math.pow(10, 6)).toString(),
          },
        },
      },
    ],
    "auto"
  );
}
