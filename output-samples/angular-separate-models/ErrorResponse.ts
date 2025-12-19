export interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: {
    field?: string;
    message?: string;
  }[];
    requestId?: string;
    timestamp?: string;
  };
}
