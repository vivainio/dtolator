// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

export interface AccessRequest {
  accessId: string;
}

export interface AccessResponse {
  redirect: string;
  aid: string;
  cid: string;
  requiresOnboarding: boolean;
}

export interface CompanyInformation {
  companyName: string;
  streetAddress: string;
  city: string;
  zipCode: string;
  country: string;
}

export interface PostMessageDto {
  topicId: string;
  content: string;
}

export interface UserInformation {
  firstName: string;
  lastName: string;
}

export interface OnBoardDto {
  companyInfo: CompanyInformation;
  userInfo: UserInformation;
}

