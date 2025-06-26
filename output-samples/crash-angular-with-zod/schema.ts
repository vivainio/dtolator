import { z } from 'zod';

export const AccessResponseSchema = z.object({
  redirect: z.string(),
  aid: z.string(),
  cid: z.string(),
  requiresOnboarding: z.boolean()
});

export type AccessResponse = z.infer<typeof AccessResponseSchema>;

export const CompanyInformationSchema = z.object({
  companyName: z.string(),
  streetAddress: z.string(),
  city: z.string(),
  zipCode: z.string(),
  country: z.string()
});

export type CompanyInformation = z.infer<typeof CompanyInformationSchema>;

export const UserInformationSchema = z.object({
  firstName: z.string(),
  lastName: z.string()
});

export type UserInformation = z.infer<typeof UserInformationSchema>;

