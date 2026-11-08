/**
 * Structured error classes for Kamino Adapter.
 */

export class AdapterError extends Error {
  public readonly statusCode: number;
  public readonly code: string;
  public readonly details?: unknown;

  constructor(message: string, statusCode = 500, code = "INTERNAL_ERROR", details?: unknown) {
    super(message);
    this.name = this.constructor.name;
    this.statusCode = statusCode;
    this.code = code;
    this.details = details;
  }
}

export class NotFoundError extends AdapterError {
  constructor(message: string, code = "NOT_FOUND", details?: unknown) {
    super(message, 404, code, details);
  }
}

export class ValidationError extends AdapterError {
  constructor(message: string, details?: unknown) {
    super(message, 400, "VALIDATION_ERROR", details);
  }
}

export class RpcError extends AdapterError {
  constructor(message: string, details?: unknown) {
    super(message, 502, "RPC_ERROR", details);
  }
}

export class TimeoutError extends AdapterError {
  constructor(message: string, details?: unknown) {
    super(message, 504, "TIMEOUT_ERROR", details);
  }
}

export class InconsistentSlotError extends AdapterError {
  constructor(message: string, details?: unknown) {
    super(message, 409, "SLOT_DRIFT_EXCEEDED", details);
  }
}

