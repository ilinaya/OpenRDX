export interface Secret {
  id: number;
  name: string;
  secret: string;
  rad_sec: boolean;
  description?: string;
  source_subnets: string[];
  created_at: string;
  updated_at: string;
}

export interface SecretCreate {
  name: string;
  secret: string;
  rad_sec?: boolean;
  description?: string;
  source_subnets?: string[];
}

export interface SecretUpdate {
  name?: string;
  secret?: string;
  rad_sec?: boolean;
  description?: string;
  source_subnets?: string[];
} 