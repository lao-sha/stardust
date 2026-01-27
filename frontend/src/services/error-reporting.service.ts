/**
 * 星尘玄鉴 - 错误上报服务
 * 
 * 提供统一的错误上报接口，支持：
 * - Sentry 集成（生产环境）
 * - 本地日志（开发环境）
 * - 错误采样和过滤
 * - 用户上下文关联
 */

import { LogLevel } from '@/lib/logger';
import { ErrorSeverity } from '@/lib/error-handler';

// ==================== 类型定义 ====================

export interface ErrorReportingConfig {
  /** 是否启用错误上报 */
  enabled: boolean;
  /** Sentry DSN */
  dsn?: string;
  /** 环境标识 */
  environment: 'development' | 'staging' | 'production';
  /** 采样率 (0-1) */
  sampleRate: number;
  /** 追踪采样率 (0-1) */
  tracesSampleRate: number;
  /** 是否启用性能监控 */
  enablePerformance: boolean;
  /** 忽略的错误类型 */
  ignoredErrors: string[];
  /** 忽略的错误消息模式 */
  ignoredMessages: RegExp[];
  /** 发布版本 */
  release?: string;
}

export interface UserContext {
  id?: string;
  address?: string;
  email?: string;
  username?: string;
}

export interface ErrorTags {
  module?: string;
  operation?: string;
  severity?: ErrorSeverity;
  [key: string]: string | undefined;
}

export interface ErrorExtra {
  [key: string]: unknown;
}

// ==================== 默认配置 ====================

const DEFAULT_CONFIG: ErrorReportingConfig = {
  enabled: !__DEV__,
  environment: __DEV__ ? 'development' : 'production',
  sampleRate: 1.0,
  tracesSampleRate: 0.2,
  enablePerformance: true,
  ignoredErrors: [
    'AuthenticationError',
    'NetworkError', // 网络错误太常见，不上报
  ],
  ignoredMessages: [
    /密码错误/,
    /Network request failed/,
    /timeout/i,
  ],
};

// ==================== 全局状态 ====================

let config: ErrorReportingConfig = { ...DEFAULT_CONFIG };
let userContext: UserContext | null = null;
let isInitialized = false;

// Sentry 实例（延迟加载）
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let Sentry: any = null;

// ==================== 初始化 ====================

/**
 * 初始化错误上报服务
 * 
 * 注意：需要先安装 @sentry/react-native
 * npm install @sentry/react-native
 */
export async function initErrorReporting(
  customConfig?: Partial<ErrorReportingConfig>
): Promise<void> {
  if (isInitialized) {
    console.warn('[ErrorReporting] Already initialized');
    return;
  }

  config = { ...DEFAULT_CONFIG, ...customConfig };

  if (!config.enabled) {
    console.log('[ErrorReporting] Disabled in current environment');
    isInitialized = true;
    return;
  }

  if (!config.dsn) {
    console.warn('[ErrorReporting] No DSN provided, using local logging only');
    isInitialized = true;
    return;
  }

  try {
    // 动态导入 Sentry（避免在未安装时报错）
    // @ts-ignore - Sentry 是可选依赖
    Sentry = await import('@sentry/react-native');

    Sentry.init({
      dsn: config.dsn,
      environment: config.environment,
      sampleRate: config.sampleRate,
      tracesSampleRate: config.tracesSampleRate,
      release: config.release,
      enableAutoSessionTracking: true,
      attachStacktrace: true,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      beforeSend: (event: any) => {
        // 过滤忽略的错误
        if (shouldIgnoreError(event)) {
          return null;
        }
        return event;
      },
    });

    console.log('[ErrorReporting] Sentry initialized successfully');
    isInitialized = true;
  } catch (error) {
    // Sentry 未安装，使用本地日志
    console.warn('[ErrorReporting] Sentry not available, using local logging');
    Sentry = null;
    isInitialized = true;
  }
}

/**
 * 检查是否应该忽略错误
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function shouldIgnoreError(event: any): boolean {
  const exception = event?.exception?.values?.[0];
  
  if (!exception) return false;

  // 检查错误类型
  if (exception.type && config.ignoredErrors.includes(exception.type)) {
    return true;
  }

  // 检查错误消息
  if (exception.value) {
    for (const pattern of config.ignoredMessages) {
      if (pattern.test(exception.value)) {
        return true;
      }
    }
  }

  return false;
}

// ==================== 用户上下文 ====================

/**
 * 设置用户上下文
 */
export function setUser(user: UserContext | null): void {
  userContext = user;

  if (Sentry && user) {
    Sentry.setUser({
      id: user.id || user.address,
      email: user.email,
      username: user.username,
    });
  } else if (Sentry) {
    Sentry.setUser(null);
  }
}

/**
 * 获取当前用户上下文
 */
export function getUser(): UserContext | null {
  return userContext;
}

// ==================== 错误上报 ====================

/**
 * 上报错误
 */
export function captureException(
  error: Error | unknown,
  tags?: ErrorTags,
  extra?: ErrorExtra
): string | undefined {
  if (!config.enabled) {
    return undefined;
  }

  // 采样检查
  if (Math.random() > config.sampleRate) {
    return undefined;
  }

  // 本地日志
  console.error('[ErrorReporting] Captured exception:', error);

  if (Sentry) {
    return Sentry.captureException(error, {
      tags: tags as Record<string, string>,
      extra,
    });
  }

  // 无 Sentry 时返回本地 ID
  return `local-${Date.now()}`;
}

/**
 * 上报消息
 */
export function captureMessage(
  message: string,
  level: 'fatal' | 'error' | 'warning' | 'info' | 'debug' = 'info',
  tags?: ErrorTags,
  extra?: ErrorExtra
): string | undefined {
  if (!config.enabled) {
    return undefined;
  }

  // 本地日志
  const logFn = level === 'error' || level === 'fatal' 
    ? console.error 
    : level === 'warning' 
      ? console.warn 
      : console.log;
  logFn(`[ErrorReporting] ${level.toUpperCase()}: ${message}`);

  if (Sentry) {
    return Sentry.captureMessage(message, {
      level,
      tags: tags as Record<string, string>,
      extra,
    });
  }

  return `local-${Date.now()}`;
}

// ==================== 面包屑 ====================

/**
 * 添加面包屑（用于追踪用户操作路径）
 */
export function addBreadcrumb(
  message: string,
  category: string,
  level: 'fatal' | 'error' | 'warning' | 'info' | 'debug' = 'info',
  data?: Record<string, unknown>
): void {
  if (!config.enabled) return;

  if (Sentry) {
    Sentry.addBreadcrumb({
      message,
      category,
      level,
      data,
      timestamp: Date.now() / 1000,
    });
  }
}

// ==================== 性能监控 ====================

/**
 * 开始性能事务
 */
export function startTransaction(
  name: string,
  op: string
): { finish: () => void } | null {
  if (!config.enabled || !config.enablePerformance) {
    return null;
  }

  if (Sentry) {
    const transaction = Sentry.startTransaction({ name, op });
    return {
      finish: () => transaction.finish(),
    };
  }

  // 本地性能追踪
  const startTime = performance.now();
  return {
    finish: () => {
      const duration = performance.now() - startTime;
      console.log(`[Performance] ${name} (${op}): ${duration.toFixed(2)}ms`);
    },
  };
}

/**
 * 测量异步操作性能
 */
export async function measureAsync<T>(
  name: string,
  op: string,
  fn: () => Promise<T>
): Promise<T> {
  const transaction = startTransaction(name, op);
  try {
    return await fn();
  } finally {
    transaction?.finish();
  }
}

// ==================== 标签和上下文 ====================

/**
 * 设置全局标签
 */
export function setTag(key: string, value: string): void {
  if (Sentry) {
    Sentry.setTag(key, value);
  }
}

/**
 * 设置全局上下文
 */
export function setContext(name: string, context: Record<string, unknown>): void {
  if (Sentry) {
    Sentry.setContext(name, context);
  }
}

// ==================== 与错误处理系统集成 ====================

/**
 * 创建远程日志函数（用于 logger 配置）
 */
export function createRemoteLogger(): (
  level: LogLevel,
  module: string,
  message: string,
  data?: unknown
) => void {
  return (level, module, message, data) => {
    // 只上报 warn 和 error 级别
    if (level < LogLevel.Warn) return;

    const sentryLevel = level === LogLevel.Error ? 'error' : 'warning';
    
    captureMessage(message, sentryLevel, { module }, { data });
  };
}

/**
 * 创建错误上报函数（用于 error-handler 配置）
 */
export function createErrorReporter(): (
  error: Error,
  context: { module: string; operation: string; metadata?: Record<string, unknown> }
) => void {
  return (error, context) => {
    captureException(error, {
      module: context.module,
      operation: context.operation,
    }, context.metadata);
  };
}

// ==================== 工具函数 ====================

/**
 * 获取当前配置
 */
export function getConfig(): ErrorReportingConfig {
  return { ...config };
}

/**
 * 检查是否已初始化
 */
export function isReady(): boolean {
  return isInitialized;
}

/**
 * 检查 Sentry 是否可用
 */
export function isSentryAvailable(): boolean {
  return Sentry !== null;
}

// ==================== 全局变量声明 ====================

declare const __DEV__: boolean;
