/**
 * 星尘玄鉴 - 统一日志服务
 * 
 * 功能：
 * - 日志级别控制 (debug, info, warn, error)
 * - 生产环境自动禁用 debug/info
 * - 模块化日志（带前缀）
 * - 日志格式化和时间戳
 * - 可选的远程日志上报
 */

// ==================== 类型定义 ====================

export enum LogLevel {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
  None = 4,
}

export interface LoggerConfig {
  /** 最小日志级别 */
  minLevel: LogLevel;
  /** 是否显示时间戳 */
  showTimestamp: boolean;
  /** 是否显示日志级别 */
  showLevel: boolean;
  /** 是否启用颜色（仅开发环境） */
  enableColors: boolean;
  /** 远程日志上报函数 */
  remoteLogger?: (level: LogLevel, module: string, message: string, data?: unknown) => void;
}

interface LogEntry {
  timestamp: number;
  level: LogLevel;
  module: string;
  message: string;
  data?: unknown;
}

// ==================== 默认配置 ====================

const DEFAULT_CONFIG: LoggerConfig = {
  minLevel: __DEV__ ? LogLevel.Debug : LogLevel.Warn,
  showTimestamp: __DEV__,
  showLevel: true,
  enableColors: __DEV__,
};

// ==================== 全局状态 ====================

let globalConfig: LoggerConfig = { ...DEFAULT_CONFIG };
const logHistory: LogEntry[] = [];
const MAX_HISTORY_SIZE = 500;

// ==================== 日志级别名称和颜色 ====================

const LEVEL_NAMES: Record<LogLevel, string> = {
  [LogLevel.Debug]: 'DEBUG',
  [LogLevel.Info]: 'INFO',
  [LogLevel.Warn]: 'WARN',
  [LogLevel.Error]: 'ERROR',
  [LogLevel.None]: '',
};

const LEVEL_COLORS: Record<LogLevel, string> = {
  [LogLevel.Debug]: '\x1b[36m', // Cyan
  [LogLevel.Info]: '\x1b[32m',  // Green
  [LogLevel.Warn]: '\x1b[33m',  // Yellow
  [LogLevel.Error]: '\x1b[31m', // Red
  [LogLevel.None]: '',
};

const RESET_COLOR = '\x1b[0m';

// ==================== 核心日志函数 ====================

function shouldLog(level: LogLevel): boolean {
  return level >= globalConfig.minLevel;
}

function formatTimestamp(): string {
  const now = new Date();
  const hours = now.getHours().toString().padStart(2, '0');
  const minutes = now.getMinutes().toString().padStart(2, '0');
  const seconds = now.getSeconds().toString().padStart(2, '0');
  const ms = now.getMilliseconds().toString().padStart(3, '0');
  return `${hours}:${minutes}:${seconds}.${ms}`;
}

function formatMessage(
  level: LogLevel,
  module: string,
  message: string
): string {
  const parts: string[] = [];

  // 时间戳
  if (globalConfig.showTimestamp) {
    parts.push(`[${formatTimestamp()}]`);
  }

  // 日志级别
  if (globalConfig.showLevel) {
    const levelName = LEVEL_NAMES[level];
    if (globalConfig.enableColors) {
      parts.push(`${LEVEL_COLORS[level]}${levelName}${RESET_COLOR}`);
    } else {
      parts.push(levelName);
    }
  }

  // 模块名
  if (module) {
    if (globalConfig.enableColors) {
      parts.push(`\x1b[35m[${module}]${RESET_COLOR}`); // Magenta
    } else {
      parts.push(`[${module}]`);
    }
  }

  // 消息
  parts.push(message);

  return parts.join(' ');
}

function log(
  level: LogLevel,
  module: string,
  message: string,
  ...data: unknown[]
): void {
  if (!shouldLog(level)) return;

  // 记录到历史
  const entry: LogEntry = {
    timestamp: Date.now(),
    level,
    module,
    message,
    data: data.length > 0 ? data : undefined,
  };
  
  logHistory.push(entry);
  if (logHistory.length > MAX_HISTORY_SIZE) {
    logHistory.shift();
  }

  // 格式化消息
  const formattedMessage = formatMessage(level, module, message);

  // 输出到控制台
  switch (level) {
    case LogLevel.Debug:
      if (data.length > 0) {
        console.debug(formattedMessage, ...data);
      } else {
        console.debug(formattedMessage);
      }
      break;
    case LogLevel.Info:
      if (data.length > 0) {
        console.info(formattedMessage, ...data);
      } else {
        console.info(formattedMessage);
      }
      break;
    case LogLevel.Warn:
      if (data.length > 0) {
        console.warn(formattedMessage, ...data);
      } else {
        console.warn(formattedMessage);
      }
      break;
    case LogLevel.Error:
      if (data.length > 0) {
        console.error(formattedMessage, ...data);
      } else {
        console.error(formattedMessage);
      }
      break;
  }

  // 远程上报
  if (globalConfig.remoteLogger && level >= LogLevel.Warn) {
    globalConfig.remoteLogger(level, module, message, data.length > 0 ? data : undefined);
  }
}

// ==================== 配置函数 ====================

/**
 * 配置日志服务
 */
export function configureLogger(config: Partial<LoggerConfig>): void {
  globalConfig = { ...globalConfig, ...config };
}

/**
 * 设置日志级别
 */
export function setLogLevel(level: LogLevel): void {
  globalConfig.minLevel = level;
}

/**
 * 获取当前配置
 */
export function getLoggerConfig(): LoggerConfig {
  return { ...globalConfig };
}

/**
 * 重置为默认配置
 */
export function resetLoggerConfig(): void {
  globalConfig = { ...DEFAULT_CONFIG };
}

// ==================== 日志历史 ====================

/**
 * 获取日志历史
 */
export function getLogHistory(): LogEntry[] {
  return [...logHistory];
}

/**
 * 清除日志历史
 */
export function clearLogHistory(): void {
  logHistory.length = 0;
}

/**
 * 导出日志历史为字符串
 */
export function exportLogHistory(): string {
  return logHistory
    .map(entry => {
      const time = new Date(entry.timestamp).toISOString();
      const level = LEVEL_NAMES[entry.level];
      const data = entry.data ? ` ${JSON.stringify(entry.data)}` : '';
      return `[${time}] ${level} [${entry.module}] ${entry.message}${data}`;
    })
    .join('\n');
}

// ==================== 模块化日志器 ====================

export interface ModuleLogger {
  debug: (message: string, ...data: unknown[]) => void;
  info: (message: string, ...data: unknown[]) => void;
  warn: (message: string, ...data: unknown[]) => void;
  error: (message: string, ...data: unknown[]) => void;
  /** 计时开始 */
  time: (label: string) => void;
  /** 计时结束并输出 */
  timeEnd: (label: string) => void;
}

const timers = new Map<string, number>();

/**
 * 创建模块化日志器
 */
export function createLogger(module: string): ModuleLogger {
  return {
    debug: (message: string, ...data: unknown[]) => {
      log(LogLevel.Debug, module, message, ...data);
    },
    info: (message: string, ...data: unknown[]) => {
      log(LogLevel.Info, module, message, ...data);
    },
    warn: (message: string, ...data: unknown[]) => {
      log(LogLevel.Warn, module, message, ...data);
    },
    error: (message: string, ...data: unknown[]) => {
      log(LogLevel.Error, module, message, ...data);
    },
    time: (label: string) => {
      const key = `${module}:${label}`;
      timers.set(key, performance.now());
    },
    timeEnd: (label: string) => {
      const key = `${module}:${label}`;
      const start = timers.get(key);
      if (start !== undefined) {
        const duration = performance.now() - start;
        timers.delete(key);
        log(LogLevel.Debug, module, `${label}: ${duration.toFixed(2)}ms`);
      }
    },
  };
}

// ==================== 全局日志函数 ====================

/**
 * 全局日志对象（无模块前缀）
 */
export const logger = createLogger('App');

// ==================== 便捷导出 ====================

export const debug = (module: string, message: string, ...data: unknown[]) =>
  log(LogLevel.Debug, module, message, ...data);

export const info = (module: string, message: string, ...data: unknown[]) =>
  log(LogLevel.Info, module, message, ...data);

export const warn = (module: string, message: string, ...data: unknown[]) =>
  log(LogLevel.Warn, module, message, ...data);

export const error = (module: string, message: string, ...data: unknown[]) =>
  log(LogLevel.Error, module, message, ...data);

// ==================== 生产环境禁用 ====================

/**
 * 在生产环境禁用所有 console 输出
 * 调用此函数后，原生 console 方法将被替换为空函数
 */
export function disableConsoleInProduction(): void {
  if (!__DEV__) {
    const noop = () => {};
    console.log = noop;
    console.debug = noop;
    console.info = noop;
    // 保留 warn 和 error 用于关键错误
    // console.warn = noop;
    // console.error = noop;
  }
}

// ==================== 全局变量声明 ====================

declare const __DEV__: boolean;

export default logger;
