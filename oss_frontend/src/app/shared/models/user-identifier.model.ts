import { UserIdentifierType } from './user-identifier-type.model';
import {AttributeGroup} from "./attribute-group.model";

export interface UserIdentifier {
  id: number;
  identifier_type: UserIdentifierType;
  value: string;
  plain_password?: string;
  is_enabled: boolean;
  comment: string;
  auth_attribute_group: AttributeGroup | null;
  expiration_date: string | null;
  reject_expired: boolean;
  expired_auth_attribute_group: AttributeGroup | null;
  created_at: string;
  updated_at: string;
  is_expired: boolean;
}
