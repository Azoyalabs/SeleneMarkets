import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { CW20SendMessage, SeleneCw20Msg } from "../types.js";
import { MARKETPLACE_ADDRESS } from "../constants.js";

export async function transferCW20WithMessage(
  sender: string,
  amount: string,
  tokenAddress: string,
  message: SeleneCw20Msg,
  signer: SigningCosmWasmClient
) {
  const tokenSendMessage: CW20SendMessage = {
    contract: MARKETPLACE_ADDRESS,
    amount: amount,
    msg: message,
  };
  return signer.execute(sender, tokenAddress, tokenSendMessage, "auto");
}
