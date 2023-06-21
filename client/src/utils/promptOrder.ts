import * as p from "@clack/prompts";
import { bold, cyan, grey, yellow } from "kleur/colors";
import { FullBalance } from "../types.js";

export async function OrderCreationPrompt(
  supportedTokens: FullBalance[],
  side: "buy" | "sell"
) {
  const selectedToken = (await p.select({
    options: supportedTokens.map((tok) => {
      return {
        label: tok.symbol,
        value: tok,
      };
    }),
    message: `Select a token to ${side}`,
  })) as FullBalance;

  const remainingTokens = supportedTokens.filter(
    (tok) => tok.address !== selectedToken.address
  );

  const against = (await p.select({
    options: remainingTokens.map((tok) => {
      return {
        label: tok.symbol,
        value: tok,
      };
    }),
    message: `${side === "buy" ? "Buy against:" : "Sell for:"}`,
  })) as FullBalance;

  const amount = (await p.text({
    message: `How Many`,
    validate: (value) => {
      const regex = new RegExp("[0-9]+");
      if (!value) {
        return "Amount is required!";
      }
      if (!regex.test(value)) {
        return "Only numbers are allowed!";
      }
    },
  })) as string;

  const shouldContinue = await p.confirm({
    message: `Do you want to ${side} ${parseInt(amount)} ${
      selectedToken.symbol
    }`,
  });
}
