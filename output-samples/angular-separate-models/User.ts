export interface User {
  id: string;
  email: string;
  profile: UserProfile;
  preferences?: UserPreferences;
  createdAt?: string;
  updatedAt?: string;
  isActive?: boolean;
  roles?: UserRole[];
}
