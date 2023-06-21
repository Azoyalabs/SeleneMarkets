
export interface FullBalance {
  name: string;
  symbol: string;
  decimals: number;
  balance: string;
  humanBalance: number;
  address: string;
}

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
