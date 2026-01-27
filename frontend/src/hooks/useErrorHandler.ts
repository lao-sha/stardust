/**
 * 星尘玄鉴 - 错误处理 Hook
 * 
 * 在函数组件中使用统一的错误处理策略
 */

import { useState, useCallback, useRef } from 'react';
import {
  handleError,
  HandledError,
  ErrorContext,
  ErrorSeverity,
  safeAsync,
  withRetry,
} from '@/lib/error-handler';
import { StardustError } from '@/lib/errors';

// ==================== 类型定义 ====================

interface UseErrorHandlerOptions {
  /** 模块名称 */
  module: string;
  /** 错误发生时的回调 */
  onError?: (error: HandledError) => void;
  /** 是否自动清除错误 */
  autoClear?: boolean;
  /** 自动清除延迟（毫秒） */
  autoClearDelay?: number;
}

interface ErrorState {
  /** 是否有错误 */
  hasError: boolean;
  /** 处理后的错误 */
  error: HandledError | null;
  /** 用户友好的消息 */
  message: string | null;
  /** 是否可重试 */
  retryable: boolean;
}

interface UseErrorHandlerReturn {
  /** 错误状态 */
  errorState: ErrorState;
  /** 处理错误 */
  handleError: (error: unknown, operation: string, metadata?: Record<string, unknown>) => HandledError;
  /** 清除错误 */
  clearError: () => void;
  /** 安全执行异步操作 */
  safeExecute: <T>(
    operation: string,
    fn: () => Promise<T>,
    options?: { fallback?: T; metadata?: Record<string, unknown> }
  ) => Promise<T | undefined>;
  /** 带重试的异步操作 */
  executeWithRetry: <T>(
    operation: string,
    fn: () => Promise<T>,
    options?: { maxRetries?: number; delay?: number; metadata?: Record<string, unknown> }
  ) => Promise<T>;
  /** 设置错误（用于外部错误） */
  setError: (error: HandledError) => void;
}

// ==================== Hook 实现 ====================

export function useErrorHandler(options: UseErrorHandlerOptions): UseErrorHandlerReturn {
  const { module, onError, autoClear = false, autoClearDelay = 5000 } = options;
  
  const [errorState, setErrorState] = useState<ErrorState>({
    hasError: false,
    error: null,
    message: null,
    retryable: false,
  });
  
  const clearTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  
  // 清除错误
  const clearError = useCallback(() => {
    if (clearTimeoutRef.current) {
      clearTimeout(clearTimeoutRef.current);
      clearTimeoutRef.current = null;
    }
    setErrorState({
      hasError: false,
      error: null,
      message: null,
      retryable: false,
    });
  }, []);
  
  // 设置错误状态
  const setError = useCallback((handled: HandledError) => {
    setErrorState({
      hasError: true,
      error: handled,
      message: handled.userMessage,
      retryable: handled.retryable,
    });
    
    // 调用回调
    onError?.(handled);
    
    // 自动清除
    if (autoClear) {
      if (clearTimeoutRef.current) {
        clearTimeout(clearTimeoutRef.current);
      }
      clearTimeoutRef.current = setTimeout(clearError, autoClearDelay);
    }
  }, [onError, autoClear, autoClearDelay, clearError]);
  
  // 处理错误
  const handleErrorFn = useCallback((
    error: unknown,
    operation: string,
    metadata?: Record<string, unknown>
  ): HandledError => {
    const context: ErrorContext = { module, operation, metadata };
    const handled = handleError(error, context);
    setError(handled);
    return handled;
  }, [module, setError]);
  
  // 安全执行异步操作
  const safeExecute = useCallback(async <T>(
    operation: string,
    fn: () => Promise<T>,
    execOptions?: { fallback?: T; metadata?: Record<string, unknown> }
  ): Promise<T | undefined> => {
    clearError();
    
    const context: ErrorContext = {
      module,
      operation,
      metadata: execOptions?.metadata,
    };
    
    const result = await safeAsync(fn, context);
    
    if (result.error) {
      setError(result.error);
      return execOptions?.fallback;
    }
    
    return result.data;
  }, [module, clearError, setError]);
  
  // 带重试的异步操作
  const executeWithRetry = useCallback(async <T>(
    operation: string,
    fn: () => Promise<T>,
    retryOptions?: { maxRetries?: number; delay?: number; metadata?: Record<string, unknown> }
  ): Promise<T> => {
    clearError();
    
    const context: ErrorContext = {
      module,
      operation,
      metadata: retryOptions?.metadata,
    };
    
    try {
      return await withRetry(fn, context, {
        maxRetries: retryOptions?.maxRetries,
        delay: retryOptions?.delay,
      });
    } catch (error) {
      if (error instanceof StardustError) {
        const handled = handleError(error, context);
        setError(handled);
      }
      throw error;
    }
  }, [module, clearError, setError]);
  
  return {
    errorState,
    handleError: handleErrorFn,
    clearError,
    safeExecute,
    executeWithRetry,
    setError,
  };
}

// ==================== 简化版 Hook ====================

/**
 * 简化版错误处理 Hook
 * 只提供基本的错误状态管理
 */
export function useSimpleError() {
  const [error, setError] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  
  const showError = useCallback((message: string) => {
    setError(message);
    setIsError(true);
  }, []);
  
  const clearError = useCallback(() => {
    setError(null);
    setIsError(false);
  }, []);
  
  const handleError = useCallback((err: unknown) => {
    if (err instanceof Error) {
      showError(err.message);
    } else if (typeof err === 'string') {
      showError(err);
    } else {
      showError('发生未知错误');
    }
  }, [showError]);
  
  return {
    error,
    isError,
    showError,
    clearError,
    handleError,
  };
}

// ==================== 错误提示 Hook ====================

/**
 * 错误提示 Hook
 * 用于显示临时错误提示
 */
export function useErrorToast(duration: number = 3000) {
  const [toast, setToast] = useState<{
    visible: boolean;
    message: string;
    severity: ErrorSeverity;
  }>({
    visible: false,
    message: '',
    severity: ErrorSeverity.Medium,
  });
  
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  
  const showToast = useCallback((
    message: string,
    severity: ErrorSeverity = ErrorSeverity.Medium
  ) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    
    setToast({ visible: true, message, severity });
    
    timeoutRef.current = setTimeout(() => {
      setToast(prev => ({ ...prev, visible: false }));
    }, duration);
  }, [duration]);
  
  const hideToast = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setToast(prev => ({ ...prev, visible: false }));
  }, []);
  
  const showErrorToast = useCallback((error: HandledError) => {
    showToast(error.userMessage, error.severity);
  }, [showToast]);
  
  return {
    toast,
    showToast,
    hideToast,
    showErrorToast,
  };
}

export default useErrorHandler;
