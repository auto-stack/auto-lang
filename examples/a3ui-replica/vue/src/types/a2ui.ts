export interface A2UIComponent {
  id: string;
  component: string;
  [key: string]: any;
}

export interface Widget {
  id: string;
  name: string;
  components: A2UIComponent[];
  dataModel: Record<string, any>;
  createdAt: number;
  updatedAt: number;
}

export interface JsonlChunk {
  id: string;
  type: string;
  payload: any;
  timestamp: number;
}

export interface GalleryWidget {
  id: string;
  name: string;
  description: string;
  components: A2UIComponent[];
  dataModel: Record<string, any>;
}

export interface CatalogComponent {
  name: string;
  category: string;
  description: string;
  props: CatalogProp[];
  example: A2UIComponent;
}

export interface CatalogProp {
  name: string;
  description: string;
  type: string;
  default?: string;
  options?: string[];
}

export interface NavItem {
  icon: string;
  label: string;
  route: string;
  subtitle?: string;
  external?: boolean;
}
