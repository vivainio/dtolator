// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

import {
  UserSchema,
  UserProfileSchema,
  AddressSchema,
  ApiResponseSchema,
} from "./schema";
import { z } from "zod";

export interface CreateUserRequest {
  email: string;
  name: string;
  age?: number | null;
  profile: UserProfile;
  address?: Address;
}

export type User = z.infer<typeof UserSchema>;
export type UserProfile = z.infer<typeof UserProfileSchema>;
export type Address = z.infer<typeof AddressSchema>;
export type ApiResponse = z.infer<typeof ApiResponseSchema>;

export { UserSchema };
export { UserProfileSchema };
export { AddressSchema };
export { ApiResponseSchema };

