import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

export async function transferCW20WithMessage(sender: string, tokenAddress: string, signer: SigningCosmWasmClient){
    signer.execute(sender, tokenAddress, {
        
    })
}