import {Timezone} from "./timezone.model";

export interface Vendor {
  id: number;
  name: string;
  description: string;
  vendor_id: number;
  created_at: string;
  updated_at: string;
}

export interface VendorAttribute {
  id: number;
  vendor: number;
  name: string;
  description: string;
  attribute_id: number;
  attribute_type: string;
  created_at: string;
  updated_at: string;
}

export interface NasGroup {
  id: number;
  name: string;
  description: string;
  parent?: NasGroup;
  parent_id: number;
  created_at: string;
  updated_at: string;
  children?: NasGroup[];
  level?: number;
  is_active: boolean;
}

export interface Secret {
  id: number;
  name: string;
  secret: string;
  rad_sec?: boolean;
  description?: string;
  source_subnets: string[];
  created_at: string;
  updated_at: string;
}

export interface Nas {
  id: number;
  name: string;
  description: string;
  ip_address: string;
  coa_enabled: boolean;
  coa_port: number;
  groups: NasGroup[];
  secret_id: number;
  secret?: Secret;
  vendor?: Vendor;
  vendor_id: number;
  timezone_id?: number;
  timezone?: Timezone;
  created_at: string;
  updated_at: string;
}

export interface NasCreate {
  name: string;
  description: string;
  ip_address: string;
  coa_enabled: boolean;
  coa_port: number;
  group_ids: number[];
  secret_id?: number;
  is_active: boolean;
}

export interface NasUpdate {
  name: string;
  description?: string;
  ip_address: string;
  coa_enabled: boolean;
  coa_port?: number;
  group_ids?: number[];
  secret_id?: number;
  vendor_id?: number;
  timezone_id?: number;
}
