export interface ApiKey {
  id: number;
  name: string;
  key: string;
  expires_at: string;
  created_by?: number;
  created_by_email?: string;
  is_active: boolean;
  last_used_at?: string;
  created_at: string;
  updated_at: string;
  is_expired: boolean;
  days_until_expiry: number;
}

export interface ApiKeyCreate {
  name: string;
  validity_days: number;  // 1 to 3650 (1 day to 10 years)
}

