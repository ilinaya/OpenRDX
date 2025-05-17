export interface UserGroup {
  id: number;
  name: string;
  description?: string;
  parent?: UserGroup;
  parent_id?: number;
  created_at: string;
  updated_at: string;
}

export interface UserGroupCreate {
  name: string;
  description: string;
  parent: number | null;
}

export interface UserGroupUpdate {
  name?: string;
  description?: string;
  parent?: number | null;
} 