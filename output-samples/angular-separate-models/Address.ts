export interface Address {
  street: string;
  street2?: string | null;
  city: string;
  state?: string | null;
  country: string;
  postalCode: string;
}
