import Ajv from 'ajv';
import schema from '../../../contracts/health-v1.schema.json';

export type Service = 'control-api' | 'host' | 'supervisor';
export interface HealthV1 {
  schema_version: 1;
  service: Service;
  status: 'scaffold';
  execution_available: false;
}
const validator = new Ajv({ strict: true, allErrors: true }).compile<HealthV1>(schema);
export function parseHealth(value: unknown): HealthV1 {
  if (!validator(value)) throw new Error('Invalid or unsupported health envelope');
  return value;
}
export function scaffoldHealth(service: Service): HealthV1 {
  return parseHealth({schema_version: 1, service, status: 'scaffold', execution_available: false});
}

export { JsonInputError, parseJsonInput, type JsonInputErrorKind, type JsonInputLimits, type JsonInputValue } from './json-input';
