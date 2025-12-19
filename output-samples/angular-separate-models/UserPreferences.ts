export interface UserPreferences {
  language?: "en" | "es" | "fr" | "de" | "it";
  currency?: "USD" | "EUR" | "GBP" | "JPY";
  notifications?: NotificationSettings;
  theme?: "light" | "dark" | "auto";
}
