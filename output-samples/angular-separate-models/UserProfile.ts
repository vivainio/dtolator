export interface UserProfile {
  firstName: string;
  lastName: string;
  dateOfBirth?: string | null;
  phoneNumber?: string | null;
  avatar?: ImageUrl;
  address?: Address;
}
