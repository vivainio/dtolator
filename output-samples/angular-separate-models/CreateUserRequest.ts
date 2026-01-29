export interface CreateUserRequest {
  email: string;
  password: string;
  profile: UserProfile;
  preferences?: UserPreferences;
}
