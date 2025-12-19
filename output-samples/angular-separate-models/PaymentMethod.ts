export interface PaymentMethod {
  type: "credit_card" | "debit_card" | "paypal" | "bank_transfer" | "crypto";
  last4?: string;
  brand?: "visa" | "mastercard" | "amex" | "discover";
}
