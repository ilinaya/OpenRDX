import { UserGroup } from './user-group.model';
import { UserIdentifier } from './user-identifier.model';

export interface User {
  id: number;
  email: string;
  first_name: string;
  last_name: string;
  phone_number: string;
  is_active: boolean;
  groups: UserGroup[];
  identifiers: UserIdentifier[];
  created_at: string;
  updated_at: string;
  last_login: string | null;
  full_name: string;
} 