export interface FullBalance {
  name: string;
  symbol: string;
  decimals: number;
  balance: string;
  humanBalance: number;
  address: string;
}

export type SeleneCw20Msg =
  | {
      limit_order: {
        market_id: number;
        price: Decimal;
      };
    }
  | {
      market_order: {
        market_id: number;
      };
    };
export type Decimal = string;

/**
 * Use send so we trigger the receiver interface
 */
export interface CW20SendMessage {
  contract: string;
  amount: string;
  msg: string; // base64 encoded SeleneCW20Msg
}

const ACTIONS = [
  "set-sell",
  "set-buy",
  "market-sell",
  "market-buy",
  "remove",
  "get-market",
  "get-bids",
  "get-asks",
  "faucet-tokens"
] as const;
export type ACTION = (typeof ACTIONS)[number];
