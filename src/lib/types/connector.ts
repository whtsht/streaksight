export type ConnectorConfigItem = {
  name: string;
  type: 'string' | 'boolean' | 'file';
  default?: any;
  nullable?: boolean;
};

export type ConnectorConfig = ConnectorConfigItem[];

export type SchemaType = 'string' | 'number' | 'boolean' | 'date';

export type Schema = {
  columns: Array<{
    name: string;
    type: SchemaType;
  }>;
};

export type ConnectorType = 'LocalFileCSV' | 'LocalFileJSON';

export const CONNECTOR_TYPES: Record<ConnectorType, string> = {
  LocalFileCSV: 'Local CSV/TSV File',
  LocalFileJSON: 'Local JSON File'
};
