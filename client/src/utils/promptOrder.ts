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

  // There's only one market, so we're only asking for good measure
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
      const regex = new RegExp(/^\d+(\.\d{1,2})?$/);
      //const regex = new RegExp("[0-9]+");
      if (!value) {
        return "Amount is required!";
      }
      if (!regex.test(value)) {
        //return value
        return "Only numbers are allowed, try again!";
      }
    },
  })) as string;

  const price = await p.text({
    message: "Set a price",
    validate: (value) => {
      // const regex = new RegExp("/^\d*\.?\d*$/")
      const regex = new RegExp(/^\d+(\.\d{1,2})?$/);

      if (!value) {
        return "Price is required!";
      }
      if (!regex.test(value)) {
        // return value;
        return "Only numbers are allowed!";
      }
    },
  });

  const shouldContinue = await p.confirm({
    message: `Do you want to ${side} ${parseInt(amount)} ${
      selectedToken.symbol
    }`,
  });

  // amount has to take decimals into account
  if (shouldContinue) {
    const amountIncludingDecimals = (
      parseInt(amount) * Math.pow(10, selectedToken.decimals)
    ).toString();
    return {
      token: selectedToken.address,
      amount: amountIncludingDecimals,
      price: price as string,
    };
  } else {
    return null;
  }
}

export async function NewOrderCreationPrompt(
  supportedTokens: FullBalance[],
  side: "buy" | "sell"
) {
  let tokenToSend: FullBalance;
  let tokenAgainst: FullBalance;
  if (side === "buy") {
    tokenToSend = supportedTokens.find((t) => t.symbol === "HUSD")!;
    tokenAgainst = supportedTokens.find((t) => t.symbol === "HEUR")!;
  } else {
    tokenToSend = supportedTokens.find((t) => t.symbol === "HEUR")!;
    tokenAgainst = supportedTokens.find((t) => t.symbol === "HUSD")!;
  }

  const amount = (await p.text({
    message: `How Many ${tokenToSend.symbol} do you want to exchange?`,
    validate: (value) => {
      const regex = new RegExp(/^\d+(\.\d{1,2})?$/);
      //const regex = new RegExp("[0-9]+");
      if (!value) {
        return "Amount is required!";
      }
      if (!regex.test(value)) {
        //return value
        return "Only numbers are allowed, try again!";
      }
    },
  })) as string;

  const price = (await p.text({
    message: "Set a price",
    validate: (value) => {
      // const regex = new RegExp("/^\d*\.?\d*$/")
      const regex = new RegExp(/^\d+(\.\d{1,2})?$/);

      if (!value) {
        return "Price is required!";
      }
      if (!regex.test(value)) {
        // return value;
        return "Only numbers are allowed!";
      }
    },
  })) as string;

  const messageContinue = (() => {
    if (side === "buy") {
      return `Buy ${tokenAgainst.symbol} for ${price}/unit (${amount} ${tokenToSend.symbol} in total)`;
    } else {
      return `Sell ${amount} ${tokenToSend.symbol} for ${price}/unit`;
    }
  })();
  const shouldContinue = await p.confirm({
    message: messageContinue,
  });

  // amount has to take decimals into account
  if (shouldContinue) {
    const amountIncludingDecimals = (
      parseInt(amount) * Math.pow(10, tokenToSend.decimals)
    ).toString();
    return {
      token: tokenToSend.address,
      amount: amountIncludingDecimals,
      price: price as string,
    };
  } else {
    return null;
  }
}


export async function NewMarketOrderCreationPrompt(
  supportedTokens: FullBalance[],
  side: "buy" | "sell"
) {
  let tokenToSend: FullBalance;
  let tokenAgainst: FullBalance;
  if (side === "buy") {
    tokenToSend = supportedTokens.find((t) => t.symbol === "HUSD")!;
    tokenAgainst = supportedTokens.find((t) => t.symbol === "HEUR")!;
  } else {
    tokenToSend = supportedTokens.find((t) => t.symbol === "HEUR")!;
    tokenAgainst = supportedTokens.find((t) => t.symbol === "HUSD")!;
  }

  const amount = (await p.text({
    message: `How Many ${tokenToSend.symbol} do you want to exchange?`,
    validate: (value) => {
      const regex = new RegExp(/^\d+(\.\d{1,2})?$/);
      //const regex = new RegExp("[0-9]+");
      if (!value) {
        return "Amount is required!";
      }
      if (!regex.test(value)) {
        //return value
        return "Only numbers are allowed, try again!";
      }
    },
  })) as string;

  const messageContinue = (() => {
    if (side === "buy") {
      return `Send a market Buy with ${amount} ${tokenToSend.symbol}`;
    } else {
      return `Send a market Sell with ${amount} ${tokenToSend.symbol}`;
    }
  })();
  const shouldContinue = await p.confirm({
    message: messageContinue,
  });

  // amount has to take decimals into account
  if (shouldContinue) {
    const amountIncludingDecimals = (
      parseInt(amount) * Math.pow(10, tokenToSend.decimals)
    ).toString();
    return {
      token: tokenToSend.address,
      amount: amountIncludingDecimals,
    };
  } else {
    return null;
  }
}