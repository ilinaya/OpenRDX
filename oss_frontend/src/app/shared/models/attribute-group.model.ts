export interface AttributeGroup {
  id: number;
  name: string;
  description?: string;
  is_system: boolean;
  created_at: string;
  updated_at: string;
}

export interface AttributeGroupCreate {
  name: string;
  description: string;
}

export interface AttributeGroupUpdate {
  name?: string;
  description?: string;
} 