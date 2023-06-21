
export interface FullBalance {
  name: string;
  symbol: string;
  decimals: number;
  balance: string;
  humanBalance: number;
  address: string;
}

/*
LimitOrder {
        market_id: u64,
        price: Decimal,
        order_side: OrderSide,
    },
    MarketOrder {
        market_id: u64,
        order_side: OrderSide,
    },*/

/**
 * Use send so we trigger the receiver interface
 */
export interface CW20SendMessage {
  contract: string,
  amount: string,
  msg: Record<string, unknown>
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
