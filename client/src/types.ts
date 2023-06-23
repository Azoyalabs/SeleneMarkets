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
        order_side: OrderSide;
        price: Decimal;
      };
    }
  | {
      market_order: {
        market_id: number;
        order_side: OrderSide;
      };
    };
export type OrderSide = "buy" | "sell";
export type Decimal = string;

/**
 * Use send so we trigger the receiver interface
 */
export interface CW20SendMessage {
  contract: string;
  amount: string;
  msg: SeleneCw20Msg;
}

const ACTIONS = [
  "set-sell",
  "set-buy",
  "remove",
  "get-market",
  "get-bids",
  "get-asks",
] as const;
export type ACTION = (typeof ACTIONS)[number];
