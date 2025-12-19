export interface Price {
  amount: number;
  currency: "USD" | "EUR" | "GBP" | "JPY";
  originalAmount?: number | null;
}
