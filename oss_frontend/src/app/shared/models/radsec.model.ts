export interface RadSecSource {
  id: number;
  name: string;
  description?: string;
  source_subnets: string[];
  created_at: string;
  updated_at: string;
  tls_key: string;
  tls_cert: string;
}

export interface RadSecSourceCreate {
  name: string;
  secret: string;
  description?: string;
  source_subnets?: string[];
  tls_key: string;
  tls_cert: string;
}

export interface RadSecSourceUpdate {
  name?: string;
  secret?: string;
  description?: string;
  source_subnets?: string[];
  tls_key: string;
  tls_cert: string;
}
