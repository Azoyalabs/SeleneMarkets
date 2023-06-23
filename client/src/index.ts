import { sendMeTokens } from "./queryTokens.js";
import * as p from "@clack/prompts";
import { bold, cyan, gray, grey, inverse, red, yellow } from "kleur/colors";
import { ALICE_MNEMONIC, BOB_MNEMONIC } from "./constants.js";
import { makeWallet } from "./utils/makeWallet.js";
import { queryTokenBalances } from "./actions/balance.js";
import { OrderCreationPrompt } from "./utils/promptOrder.js";
import { ACTION, SeleneCw20Msg } from "./types.js";
import { transferCW20WithMessage } from "./utils/transferCW20.js";

p.intro(inverse(cyan("Selene Markets - Archway Hackathon edition")));

async function main() {
  const mnemonic = (await p.select({
    message: "Select a user",
    options: [
      {
        label: "Bob",
        value: BOB_MNEMONIC,
      },
      {
        label: "Alice",
        value: ALICE_MNEMONIC,
      },
    ],
  })) as string;
  const wallet = await makeWallet(mnemonic);
  console.log(`${cyan(`Connected as ${wallet.address}`)}`);

  const balances = await queryTokenBalances(
    wallet.address,
    wallet.cosmwasmSigner
  );
  console.log(`${cyan(`Your available tokens are:`)}`);
  console.table(balances);

  const action = (await p.select({
    message: "What do you want to do?",
    options: [
      {
        label: "Place a sell order",
        value: "set-sell",
      },
      {
        label: "Place a buy order",
        value: "set-buy",
      },
      {
        label: "Remove an order",
        value: "remove",
      },
      {
        label: "Get topmost orders from the market",
        value: "get-market",
      },
      {
        label: "Get my currently placed bid orders",
        value: "get-bids",
      },
      {
        label: "Get my currently placed ask orders",
        value: "get-asks",
      },
    ],
  })) as ACTION;

  switch (action) {
    case "set-sell":
      const sellPrompt = await OrderCreationPrompt(balances, "sell");
      // prompt.amount
      const limitSellMsg: SeleneCw20Msg = {
        limit_order: {
          market_id: 1,
          order_side: "sell",
          price: "price",
        },
      };

      const putSellOrder = await transferCW20WithMessage(
        wallet.address,
        "amount",
        "tokenAddress",
        limitSellMsg,
        wallet.cosmwasmSigner
      );
      break;

    case "set-buy":
      const buyPrompt = await OrderCreationPrompt(balances, "buy");
      break;

    case "remove":
      // wallet.seleneClient.
      console.log(red("remove is not implemented yet"));
      break;

    case "get-bids":
      const { orders: bidOrders } = await wallet.seleneClient.getUserBids({
        targetMarket: 1,
      });
      console.log(gray(`Current open bid orders`));
      console.table(
        bidOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
      );

      break;

    case "get-asks":
      const { orders: askOrders } = await wallet.seleneClient.getUserAsks({
        targetMarket: 1,
      });
      console.log(gray(`Current open sell orders`));
      console.table(
        askOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
      );

      break;

    case "get-market":
      console.log(red("remove is not implemented yet"));
      break;
    default:
      break;
  }

  console.dir(action);
  await sendMeTokens("me");
}

main().finally(() => process.exit(0));
