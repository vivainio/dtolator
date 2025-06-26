// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

import {
  AccessResponseSchema,
  CompanyInformationSchema,
  UserInformationSchema,
} from "./schema";
import { z } from "zod";

export interface AccessRequest {
  accessId: string;
}
export interface PostMessageDto {
  topicId: string;
  content: string;
}
export interface OnBoardDto {
  companyInfo: CompanyInformation;
  userInfo: UserInformation;
}

export type AccessResponse = z.infer<typeof AccessResponseSchema>;
export type CompanyInformation = z.infer<typeof CompanyInformationSchema>;
export type UserInformation = z.infer<typeof UserInformationSchema>;

export { AccessResponseSchema };
export { CompanyInformationSchema };
export { UserInformationSchema };

