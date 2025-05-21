export interface NasDevice {
  id: number;
  name: string;
  ip_address: string;
  nas_type: {
    id: number;
    name: string;
  };
} 