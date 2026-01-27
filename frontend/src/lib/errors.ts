/**
 * 星尘玄鉴 - 错误类定义
 * 提供统一的错误处理机制
 */

/**
 * 基础错误类
 */
export class StardustError extends Error {
  constructor(
    message: string,
    public code?: string,
    public cause?: unknown
  ) {
    super(message);
    this.name = this.constructor.name;
    
    // 确保原型链正确（TypeScript 编译目标为 ES5 时需要）
    Object.setPrototypeOf(this, new.target.prototype);
  }

  /**
   * 转换为用户友好的消息
   */
  toUserMessage(): string {
    return this.message;
  }

  /**
   * 转换为 JSON 格式（用于日志和上报）
   */
  toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      stack: this.stack,
    };
  }
}

/**
 * 钱包相关错误
 */
export class WalletError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'WALLET_ERROR', cause);
  }

  toUserMessage(): string {
    return '钱包操作失败，请稍后重试';
  }
}

/**
 * 认证错误（密码错误）
 */
export class AuthenticationError extends StardustError {
  constructor(message: string = '密码错误') {
    super(message, 'AUTH_ERROR');
  }

  toUserMessage(): string {
    return '密码错误，请重试';
  }
}

/**
 * 加密错误
 */
export class CryptoError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'CRYPTO_ERROR', cause);
  }

  toUserMessage(): string {
    return '加密操作失败';
  }
}

/**
 * 链连接错误
 */
export class APIConnectionError extends StardustError {
  constructor(message: string = '无法连接到区块链节点', cause?: unknown) {
    super(message, 'API_CONNECTION_ERROR', cause);
  }

  toUserMessage(): string {
    return '无法连接到服务器，请检查网络';
  }
}

/**
 * 交易错误
 */
export class TransactionError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'TRANSACTION_ERROR', cause);
  }

  toUserMessage(): string {
    if (this.message.includes('余额') || this.message.toLowerCase().includes('insufficient')) {
      return '余额不足';
    }
    return '交易失败，请稍后重试';
  }
}

/**
 * 网络错误
 */
export class NetworkError extends StardustError {
  constructor(message: string = '网络连接失败', cause?: unknown) {
    super(message, 'NETWORK_ERROR', cause);
  }

  toUserMessage(): string {
    return '网络连接失败，请检查网络设置';
  }
}

/**
 * 占卜错误
 */
export class DivinationError extends StardustError {
  constructor(message: string, cause?: unknown) {
    super(message, 'DIVINATION_ERROR', cause);
  }

  toUserMessage(): string {
    return '占卜服务暂时不可用';
  }
}

/**
 * 验证错误
 */
export class ValidationError extends StardustError {
  constructor(message: string, public field?: string) {
    super(message, 'VALIDATION_ERROR');
  }

  toUserMessage(): string {
    return this.message;
  }
}

/**
 * 超时错误
 */
export class TimeoutError extends StardustError {
  constructor(message: string = '操作超时', cause?: unknown) {
    super(message, 'TIMEOUT_ERROR', cause);
  }

  toUserMessage(): string {
    return '操作超时，请重试';
  }
}

/**
 * 权限错误
 */
export class PermissionError extends StardustError {
  constructor(message: string = '没有权限执行此操作') {
    super(message, 'PERMISSION_ERROR');
  }

  toUserMessage(): string {
    return '没有权限执行此操作';
  }
}

/**
 * 资源未找到错误
 */
export class NotFoundError extends StardustError {
  constructor(resource: string) {
    super(`${resource}不存在`, 'NOT_FOUND_ERROR');
  }

  toUserMessage(): string {
    return this.message;
  }
}

/**
 * 业务逻辑错误
 */
export class BusinessError extends StardustError {
  constructor(message: string, code?: string) {
    super(message, code ?? 'BUSINESS_ERROR');
  }

  toUserMessage(): string {
    return this.message;
  }
}

// ==================== 错误工厂函数 ====================

/**
 * 从未知错误创建 StardustError
 */
export function fromUnknown(error: unknown): StardustError {
  if (error instanceof StardustError) {
    return error;
  }

  if (error instanceof Error) {
    return new StardustError(error.message, 'UNKNOWN_ERROR', error);
  }

  if (typeof error === 'string') {
    return new StardustError(error, 'UNKNOWN_ERROR');
  }

  return new StardustError('发生未知错误', 'UNKNOWN_ERROR', error);
}

/**
 * 检查是否为特定类型的错误
 */
export function isErrorType<T extends StardustError>(
  error: unknown,
  ErrorClass: new (...args: never[]) => T
): error is T {
  return error instanceof ErrorClass;
}

/**
 * 检查是否为可重试的错误
 */
export function isRetryableError(error: unknown): boolean {
  if (error instanceof NetworkError || error instanceof APIConnectionError) {
    return true;
  }
  if (error instanceof TimeoutError) {
    return true;
  }
  if (error instanceof AuthenticationError || error instanceof ValidationError) {
    return false;
  }
  return false;
}
