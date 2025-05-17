export interface AdminUser {
  id: number;
  email: string;
  first_name: string;
  last_name: string;
  position?: string;
  phone_number?: string;
  is_active: boolean;
  is_staff: boolean;
  is_superuser: boolean;
  groups: AdminGroup[];
  created_at: string;
  updated_at: string;
}

export interface AdminUserCreate {
  username: string;
  email: string;
  first_name: string;
  last_name: string;
  phone_number?: string;
  position?: string;
  is_active: boolean;
  is_staff: boolean;
  is_superuser: boolean;
  password: string;
}

export interface AdminUserUpdate {
  username?: string;
  email?: string;
  first_name?: string;
  last_name?: string;
  phone_number?: string;
  position?: string;
  is_active?: boolean;
  is_staff?: boolean;
  is_superuser?: boolean;
  password?: string;
}

export interface AdminGroup {
  id: number;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface AdminGroupCreate {
  name: string;
  description: string;
}

export interface AdminGroupUpdate {
  name?: string;
  description?: string;
}