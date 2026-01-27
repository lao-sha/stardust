/**
 * 星尘玄鉴 - 统一错误处理系统
 * 
 * 提供一致的错误处理策略：
 * 1. 错误分类和转换
 * 2. 错误日志记录
 * 3. 用户友好的错误消息
 * 4. 错误上报（可选 Sentry 集成）
 */

import {
  StardustError,
  WalletError,
  AuthenticationError,
  CryptoError,
  APIConnectionError,
  TransactionError,
  NetworkError,
  DivinationError,
} from './errors';

// ==================== 错误严重级别 ====================

export enum ErrorSeverity {
  /** 低级别 - 可恢复的错误，不影响核心功能 */
  Low = 'low',
  /** 中级别 - 影响部分功能，需要用户注意 */
  Medium = 'medium',
  /** 高级别 - 影响核心功能，需要立即处理 */
  High = 'high',
  /** 致命级别 - 应用无法继续运行 */
  Critical = 'critical',
}

// ==================== 错误上下文 ====================

export interface ErrorContext {
  /** 错误发生的模块/服务 */
  module: string;
  /** 错误发生的操作 */
  operation: string;
  /** 用户ID（如果有） */
  userId?: string;
  /** 额外的上下文数据 */
  metadata?: Record<string, unknown>;
}

// ==================== 处理后的错误结果 ====================

export interface HandledError {
  /** 原始错误 */
  original: unknown;
  /** 转换后的 StardustError */
  error: StardustError;
  /** 用户友好的消息 */
  userMessage: string;
  /** 错误严重级别 */
  severity: ErrorSeverity;
  /** 是否应该重试 */
  retryable: boolean;
  /** 错误时间戳 */
  timestamp: number;
}

// ==================== 错误消息映射 ====================

const ERROR_MESSAGES: Record<string, string> = {
  // 钱包相关
  WALLET_ERROR: '钱包操作失败',
  AUTH_ERROR: '密码错误，请重试',
  CRYPTO_ERROR: '加密操作失败',
  
  // 网络相关
  API_CONNECTION_ERROR: '无法连接到服务器，请检查网络',
  NETWORK_ERROR: '网络连接失败，请稍后重试',
  
  // 交易相关
  TRANSACTION_ERROR: '交易失败',
  INSUFFICIENT_BALANCE: '余额不足',
  TRANSACTION_REJECTED: '交易被拒绝',
  
  // 占卜相关
  DIVINATION_ERROR: '占卜服务暂时不可用',
  
  // 通用
  UNKNOWN_ERROR: '发生未知错误，请稍后重试',
  TIMEOUT_ERROR: '操作超时，请重试',
  VALIDATION_ERROR: '输入数据无效',
};

// ==================== 错误上报配置 ====================

interface ErrorReporterConfig {
  /** 是否启用错误上报 */
  enabled: boolean;
  /** Sentry DSN（如果使用 Sentry） */
  sentryDsn?: string;
  /** 环境标识 */
  environment: 'development' | 'staging' | 'production';
  /** 采样率 (0-1) */
  sampleRate: number;
  /** 忽略的错误类型 */
  ignoredErrors: string[];
}

let reporterConfig: ErrorReporterConfig = {
  enabled: false,
  environment: 'development',
  sampleRate: 1.0,
  ignoredErrors: ['AuthenticationError'],
};

// ==================== 错误日志队列 ====================

interface ErrorLogEntry {
  error: HandledError;
  context: ErrorContext;
  reported: boolean;
}

const errorLog: ErrorLogEntry[] = [];
const MAX_LOG_SIZE = 100;

// ==================== 核心错误处理函数 ====================

/**
 * 统一错误处理入口
 * 
 * @param error 原始错误
 * @param context 错误上下文
 * @returns 处理后的错误信息
 */
export function handleError(
  error: unknown,
  context: ErrorContext
): HandledError {
  const timestamp = Date.now();
  
  // 1. 转换为 StardustError
  const stardustError = normalizeError(error);
  
  // 2. 确定严重级别
  const severity = determineSeverity(stardustError, context);
  
  // 3. 获取用户友好消息
  const userMessage = getUserMessage(stardustError);
  
  // 4. 判断是否可重试
  const retryable = isRetryable(stardustError);
  
  // 5. 构建处理结果
  const handled: HandledError = {
    original: error,
    error: stardustError,
    userMessage,
    severity,
    retryable,
    timestamp,
  };
  
  // 6. 记录日志
  logError(handled, context);
  
  // 7. 上报错误（如果启用）
  reportError(handled, context);
  
  return handled;
}

/**
 * 将任意错误转换为 StardustError
 */
export function normalizeError(error: unknown): StardustError {
  // 已经是 StardustError
  if (error instanceof StardustError) {
    return error;
  }
  
  // 标准 Error
  if (error instanceof Error) {
    // 检查是否是特定类型的错误
    const message = error.message.toLowerCase();
    
    if (message.includes('network') || message.includes('fetch')) {
      return new NetworkError(error.message, error);
    }
    
    if (message.includes('timeout')) {
      return new NetworkError('操作超时', error);
    }
    
    if (message.includes('密码') || message.includes('password')) {
      return new AuthenticationError(error.message);
    }
    
    if (message.includes('余额') || message.includes('balance') || message.includes('insufficient')) {
      return new TransactionError('余额不足', error);
    }
    
    if (message.includes('api') || message.includes('connection')) {
      return new APIConnectionError(error.message, error);
    }
    
    // 通用错误
    return new StardustError(error.message, 'UNKNOWN_ERROR', error);
  }
  
  // 字符串错误
  if (typeof error === 'string') {
    return new StardustError(error, 'UNKNOWN_ERROR');
  }
  
  // 其他类型
  return new StardustError('发生未知错误', 'UNKNOWN_ERROR', error);
}

/**
 * 确定错误严重级别
 */
function determineSeverity(
  error: StardustError,
  context: ErrorContext
): ErrorSeverity {
  // 认证错误 - 低级别（用户可自行解决）
  if (error instanceof AuthenticationError) {
    return ErrorSeverity.Low;
  }
  
  // 网络错误 - 中级别（可能是临时的）
  if (error instanceof NetworkError || error instanceof APIConnectionError) {
    return ErrorSeverity.Medium;
  }
  
  // 交易错误 - 高级别（涉及资金）
  if (error instanceof TransactionError) {
    return ErrorSeverity.High;
  }
  
  // 加密错误 - 致命级别（可能导致数据丢失）
  if (error instanceof CryptoError) {
    return ErrorSeverity.Critical;
  }
  
  // 钱包错误 - 高级别
  if (error instanceof WalletError) {
    return ErrorSeverity.High;
  }
  
  // 默认中级别
  return ErrorSeverity.Medium;
}

/**
 * 获取用户友好的错误消息
 */
function getUserMessage(error: StardustError): string {
  // 优先使用错误码对应的消息
  if (error.code) {
    const message = ERROR_MESSAGES[error.code];
    if (message) {
      return message;
    }
  }
  
  // 使用错误消息（如果足够友好）
  if (error.message && !error.message.includes('Error:') && error.message.length < 100) {
    return error.message;
  }
  
  // 默认消息
  return ERROR_MESSAGES.UNKNOWN_ERROR ?? '发生未知错误';
}

/**
 * 判断错误是否可重试
 */
function isRetryable(error: StardustError): boolean {
  // 网络错误通常可重试
  if (error instanceof NetworkError || error instanceof APIConnectionError) {
    return true;
  }
  
  // 认证错误不应自动重试
  if (error instanceof AuthenticationError) {
    return false;
  }
  
  // 加密错误不应重试
  if (error instanceof CryptoError) {
    return false;
  }
  
  // 交易错误需要根据具体情况判断
  if (error instanceof TransactionError) {
    const message = error.message.toLowerCase();
    // 余额不足不应重试
    if (message.includes('余额') || message.includes('insufficient')) {
      return false;
    }
    // 其他交易错误可能可以重试
    return true;
  }
  
  return false;
}

// ==================== 日志记录 ====================

/**
 * 记录错误日志
 */
function logError(handled: HandledError, context: ErrorContext): void {
  const entry: ErrorLogEntry = {
    error: handled,
    context,
    reported: false,
  };
  
  // 添加到日志队列
  errorLog.push(entry);
  
  // 限制日志大小
  if (errorLog.length > MAX_LOG_SIZE) {
    errorLog.shift();
  }
  
  // 控制台输出（开发环境）
  if (__DEV__ || reporterConfig.environment === 'development') {
    const prefix = `[${context.module}:${context.operation}]`;
    
    switch (handled.severity) {
      case ErrorSeverity.Critical:
        console.error(`🔴 ${prefix} CRITICAL:`, handled.error.message, handled.error);
        break;
      case ErrorSeverity.High:
        console.error(`🟠 ${prefix} HIGH:`, handled.error.message);
        break;
      case ErrorSeverity.Medium:
        console.warn(`🟡 ${prefix} MEDIUM:`, handled.error.message);
        break;
      case ErrorSeverity.Low:
        console.log(`🟢 ${prefix} LOW:`, handled.error.message);
        break;
    }
  }
}

/**
 * 获取错误日志
 */
export function getErrorLog(): ErrorLogEntry[] {
  return [...errorLog];
}

/**
 * 清除错误日志
 */
export function clearErrorLog(): void {
  errorLog.length = 0;
}

// ==================== 错误上报 ====================

/**
 * 配置错误上报
 */
export function configureErrorReporter(config: Partial<ErrorReporterConfig>): void {
  reporterConfig = { ...reporterConfig, ...config };
}

/**
 * 上报错误到远程服务
 */
function reportError(handled: HandledError, context: ErrorContext): void {
  // 检查是否启用上报
  if (!reporterConfig.enabled) {
    return;
  }
  
  // 检查是否在忽略列表中
  if (reporterConfig.ignoredErrors.includes(handled.error.name)) {
    return;
  }
  
  // 采样
  if (Math.random() > reporterConfig.sampleRate) {
    return;
  }
  
  // 只上报中级别以上的错误
  if (handled.severity === ErrorSeverity.Low) {
    return;
  }
  
  // TODO: 集成 Sentry 或其他错误上报服务
  // 示例 Sentry 集成：
  // if (reporterConfig.sentryDsn) {
  //   Sentry.captureException(handled.error, {
  //     tags: {
  //       module: context.module,
  //       operation: context.operation,
  //       severity: handled.severity,
  //     },
  //     extra: {
  //       userMessage: handled.userMessage,
  //       retryable: handled.retryable,
  //       metadata: context.metadata,
  //     },
  //   });
  // }
  
  // 标记为已上报
  const entry = errorLog.find(e => e.error === handled);
  if (entry) {
    entry.reported = true;
  }
}

// ==================== 便捷工具函数 ====================

/**
 * 创建带上下文的错误处理器
 */
export function createErrorHandler(module: string) {
  return {
    /**
     * 处理错误并返回结果
     */
    handle: (error: unknown, operation: string, metadata?: Record<string, unknown>) => {
      return handleError(error, { module, operation, metadata });
    },
    
    /**
     * 包装异步函数，自动处理错误
     */
    wrap: <T>(
      operation: string,
      fn: () => Promise<T>,
      options?: {
        fallback?: T;
        rethrow?: boolean;
        metadata?: Record<string, unknown>;
      }
    ): Promise<T | undefined> => {
      return fn().catch((error) => {
        const handled = handleError(error, {
          module,
          operation,
          metadata: options?.metadata,
        });
        
        if (options?.rethrow) {
          throw handled.error;
        }
        
        return options?.fallback as T | undefined;
      });
    },
    
    /**
     * 包装同步函数，自动处理错误
     */
    wrapSync: <T>(
      operation: string,
      fn: () => T,
      options?: {
        fallback?: T;
        rethrow?: boolean;
        metadata?: Record<string, unknown>;
      }
    ): T | undefined => {
      try {
        return fn();
      } catch (error) {
        const handled = handleError(error, {
          module,
          operation,
          metadata: options?.metadata,
        });
        
        if (options?.rethrow) {
          throw handled.error;
        }
        
        return options?.fallback as T | undefined;
      }
    },
  };
}

/**
 * 安全执行异步操作
 */
export async function safeAsync<T>(
  fn: () => Promise<T>,
  context: ErrorContext
): Promise<{ data?: T; error?: HandledError }> {
  try {
    const data = await fn();
    return { data };
  } catch (error) {
    const handled = handleError(error, context);
    return { error: handled };
  }
}

/**
 * 带重试的异步操作
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  context: ErrorContext,
  options: {
    maxRetries?: number;
    delay?: number;
    backoff?: number;
  } = {}
): Promise<T> {
  const { maxRetries = 3, delay = 1000, backoff = 2 } = options;
  
  let lastError: HandledError | undefined;
  let currentDelay = delay;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = handleError(error, {
        ...context,
        metadata: { ...context.metadata, attempt },
      });
      
      // 如果不可重试，立即抛出
      if (!lastError.retryable || attempt === maxRetries) {
        throw lastError.error;
      }
      
      // 等待后重试
      await new Promise(resolve => setTimeout(resolve, currentDelay));
      currentDelay *= backoff;
    }
  }
  
  throw lastError?.error ?? new StardustError('重试失败', 'RETRY_FAILED');
}

// ==================== 全局变量声明 ====================

declare const __DEV__: boolean;
