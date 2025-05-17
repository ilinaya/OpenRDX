export type AttributeType = 'string' | 'integer' | 'ipaddr' | 'date' | 'octets';

export interface RadiusAttribute {
  id: number;
  group: number;  // AttributeGroup ID
  vendor_id: number;
  attribute_id: number;
  attribute_name: string;
  attribute_type: AttributeType;
  attribute_value: string;
  created_at: string;
  updated_at: string;
}

export interface RadiusAttributeCreate {
  group: number;
  vendor_id: number;
  attribute_id: number;
  attribute_name: string;
  attribute_type: AttributeType;
  attribute_value: string;
}

export interface RadiusAttributeUpdate {
  vendor_id?: number;
  attribute_id?: number;
  attribute_name?: string;
  attribute_type?: AttributeType;
  attribute_value?: string;
} 