import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { HEUR_ADDRESS, HUSD_ADDRESS } from "../constants.js";

export async function queryTokenBalances(
  address: string,
  signingClient: SigningCosmWasmClient
) {
  const tokenAddresses = [HEUR_ADDRESS, HUSD_ADDRESS];
  const allTokenInfo = await Promise.all(
    tokenAddresses.map(async (add) => {
      const { balance }: { balance: string } =
        await signingClient.queryContractSmart(add, {
          balance: {
            address,
          },
        });

      const {
        name,
        symbol,
        decimals,
        total_supply,
      }: {
        name: string;
        symbol: string;
        decimals: number;
        total_supply: string;
      } = await signingClient.queryContractSmart(add, {
        token_info: {},
      });

      return {
        balance,
        humanBalance: parseInt(balance) * Math.pow(10, -decimals),
        name,
        symbol,
        decimals,
        address: add,
      };
    })
  );

  return allTokenInfo;
}
