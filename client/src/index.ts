import { sendMeTokens } from "./queryTokens.js";
import * as p from "@clack/prompts";
import { bold, cyan, gray, grey, inverse, red, yellow } from "kleur/colors";
import { ALICE_MNEMONIC, BOB_MNEMONIC, EXPLORER_TX_LINK, MARKET_ID } from "./constants.js";
import { makeWallet } from "./utils/makeWallet.js";
import { queryTokenBalances } from "./actions/balance.js";
import { OrderCreationPrompt } from "./utils/promptOrder.js";
import { ACTION, SeleneCw20Msg } from "./types.js";
import { transferCW20WithMessage } from "./utils/transferCW20.js";
import { stringify } from "querystring";
import { UserOrderRecord } from "./contract/Selene.types.js";

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
      if (sellPrompt) {
        const limitSellMsg: SeleneCw20Msg = {
          limit_order: {
            market_id: MARKET_ID,
            price: sellPrompt.price,
          },
        };

        try {
          const s = p.spinner();
          s.start(
            `Sending sell order: ${red(
              `${sellPrompt.amount} ${sellPrompt.token} at ${sellPrompt.price}`
            )}`
          );
          const putSellOrder = await transferCW20WithMessage(
            wallet.address,
            sellPrompt.amount,
            sellPrompt.token,
            limitSellMsg,
            wallet.cosmwasmSigner
          );
          s.stop(`transaction successful: ${EXPLORER_TX_LINK(putSellOrder.transactionHash)}`);
        } catch (error) {
          console.error(error);
        }
      } else {
        p.outro("Order cancelled");
      }
      break;

    case "set-buy":
      const buyPrompt = await OrderCreationPrompt(balances, "buy");
      if (buyPrompt) {
        const limitSellMsg: SeleneCw20Msg = {
          limit_order: {
            market_id: MARKET_ID,
            price: buyPrompt.price,
          },
        };

        try {
          const s = p.spinner();
          s.start(
            `Sending buy order: ${red(
              `${buyPrompt.amount} ${buyPrompt.token} at ${buyPrompt.price}`
            )}`
          );
          const buyOrderTx = await transferCW20WithMessage(
            wallet.address,
            buyPrompt.amount,
            buyPrompt.token,
            limitSellMsg,
            wallet.cosmwasmSigner
          );
          s.stop(`transaction successful: ${EXPLORER_TX_LINK(buyOrderTx.transactionHash)}`);
        } catch (error) {
          console.error(error);
        }
      } else {
        p.outro("Order cancelled");
      }
      break;


    case "get-bids":
      try {
        const { orders: bidOrders } = await wallet.seleneClient.getUserBids({
          targetMarket: MARKET_ID,
          userAddress: wallet.address,
        });
        console.log(`${gray(`Current open bid orders`)}`);
        console.table(
          bidOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
        );
      } catch (error) {
        console.log(yellow(`No bid orders found for ${wallet.address}`));
      }

      break;

    case "get-asks":
      try {
        const { orders: askOrders } = await wallet.seleneClient.getUserAsks({
          targetMarket: MARKET_ID,
          userAddress: wallet.address,
        });
        console.log(gray(`Current open sell orders`));
        console.table(
          askOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
        );
      } catch (error) {
        console.log(yellow(`No sell orders found for ${wallet.address}`));
      }

      break;

    case "get-market":
      try {
        const { bids: bidOrders, asks: askOrders } = await wallet.seleneClient.getMarketBook({
          marketId: MARKET_ID,
          nbLevels: 10
        });
        console.log(gray(`Current open buy orders`));
        console.table(
          bidOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
        );
        console.log(gray(`Current open sell orders`));
        console.table(
          askOrders.map((o) => ({ price: o.price, quantity: o.quantity }))
        );
      } catch (error) {
        console.log(yellow(`No orders on this market`));
      }
      break;

    case "remove":
      try {
        const res = await wallet.seleneClient.getUserOrders({
          targetMarket: 0,
          userAddress: wallet.address
        });

        // orders to options
        const order_opts = res.orders.map((o) => ({
          label: JSON.stringify(o),
          value: o
        }));
        
        // res.orders
        const chosen_order = (await p.select({
          message: "Which order do you want to cancel",
          options: order_opts,
        })) as UserOrderRecord;

        
        const s = p.spinner();
        s.start(
          `Cancelling order`
        );
        const removeTx = await wallet.seleneClient.removeLimitOrder({marketId: chosen_order.market_id, price: chosen_order.price});
        s.stop(`transaction successful: ${EXPLORER_TX_LINK(removeTx.transactionHash)}`);

      } catch (error) {
        console.log(yellow(`No orders on this market`));
      }
      break;
      //wallet.seleneClient.
      return
      // TODO: need to get current orders to prompt which one to remove
      wallet.seleneClient.removeLimitOrder({
        marketId: MARKET_ID,
        price: "1.1"
      })

      break;
    default:
      break;
  }

  //console.dir(action);
  //await sendMeTokens("me");
}

main().finally(() => process.exit(0));
