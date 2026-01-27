/**
 * 错误处理器测试
 */

import {
  handleError,
  normalizeError,
  createErrorHandler,
  safeAsync,
  withRetry,
  ErrorSeverity,
} from '../error-handler';
import {
  StardustError,
  WalletError,
  AuthenticationError,
  NetworkError,
  TransactionError,
} from '../errors';

describe('Error Handler', () => {
  describe('normalizeError', () => {
    it('should keep StardustError as is', () => {
      const error = new WalletError('Test error');
      const normalized = normalizeError(error);
      expect(normalized).toBe(error);
      expect(normalized instanceof WalletError).toBe(true);
    });

    it('should convert Error to StardustError', () => {
      const error = new Error('Test error');
      const normalized = normalizeError(error);
      expect(normalized instanceof StardustError).toBe(true);
      expect(normalized.message).toBe('Test error');
    });

    it('should detect network errors', () => {
      const error = new Error('Network request failed');
      const normalized = normalizeError(error);
      expect(normalized instanceof NetworkError).toBe(true);
    });

    it('should detect authentication errors', () => {
      const error = new Error('密码错误');
      const normalized = normalizeError(error);
      expect(normalized instanceof AuthenticationError).toBe(true);
    });

    it('should detect balance errors', () => {
      const error = new Error('余额不足');
      const normalized = normalizeError(error);
      expect(normalized instanceof TransactionError).toBe(true);
    });

    it('should convert string to StardustError', () => {
      const normalized = normalizeError('String error');
      expect(normalized instanceof StardustError).toBe(true);
      expect(normalized.message).toBe('String error');
    });

    it('should handle unknown types', () => {
      const normalized = normalizeError({ unknown: 'object' });
      expect(normalized instanceof StardustError).toBe(true);
      expect(normalized.message).toBe('发生未知错误');
    });
  });

  describe('handleError', () => {
    it('should handle error with context', () => {
      const error = new Error('Test error');
      const handled = handleError(error, {
        module: 'TestModule',
        operation: 'testOperation',
      });

      expect(handled.error instanceof StardustError).toBe(true);
      expect(handled.userMessage).toBeTruthy();
      expect(handled.severity).toBeDefined();
      expect(handled.retryable).toBeDefined();
      expect(handled.timestamp).toBeGreaterThan(0);
    });

    it('should determine correct severity', () => {
      const authError = new AuthenticationError('密码错误');
      const handled = handleError(authError, {
        module: 'Auth',
        operation: 'login',
      });
      expect(handled.severity).toBe(ErrorSeverity.Low);

      const networkError = new NetworkError('网络错误');
      const handledNetwork = handleError(networkError, {
        module: 'Network',
        operation: 'fetch',
      });
      expect(handledNetwork.severity).toBe(ErrorSeverity.Medium);
    });

    it('should determine retryability', () => {
      const networkError = new NetworkError('网络错误');
      const handled = handleError(networkError, {
        module: 'Network',
        operation: 'fetch',
      });
      expect(handled.retryable).toBe(true);

      const authError = new AuthenticationError('密码错误');
      const handledAuth = handleError(authError, {
        module: 'Auth',
        operation: 'login',
      });
      expect(handledAuth.retryable).toBe(false);
    });
  });

  describe('createErrorHandler', () => {
    it('should create module-specific error handler', () => {
      const handler = createErrorHandler('TestModule');
      expect(handler.handle).toBeDefined();
      expect(handler.wrap).toBeDefined();
      expect(handler.wrapSync).toBeDefined();
    });

    it('should handle errors with module context', () => {
      const handler = createErrorHandler('TestModule');
      const error = new Error('Test error');
      const handled = handler.handle(error, 'testOperation');

      expect(handled.error instanceof StardustError).toBe(true);
    });

    it('should wrap async functions', async () => {
      const handler = createErrorHandler('TestModule');
      const successFn = async () => 'success';
      const result = await handler.wrap('test', successFn);
      expect(result).toBe('success');
    });

    it('should handle async function errors', async () => {
      const handler = createErrorHandler('TestModule');
      const errorFn = async () => {
        throw new Error('Test error');
      };
      const result = await handler.wrap('test', errorFn, { fallback: 'fallback' });
      expect(result).toBe('fallback');
    });

    it('should wrap sync functions', () => {
      const handler = createErrorHandler('TestModule');
      const successFn = () => 'success';
      const result = handler.wrapSync('test', successFn);
      expect(result).toBe('success');
    });

    it('should handle sync function errors', () => {
      const handler = createErrorHandler('TestModule');
      const errorFn = () => {
        throw new Error('Test error');
      };
      const result = handler.wrapSync('test', errorFn, { fallback: 'fallback' });
      expect(result).toBe('fallback');
    });
  });

  describe('safeAsync', () => {
    it('should return data on success', async () => {
      const fn = async () => 'success';
      const result = await safeAsync(fn, {
        module: 'Test',
        operation: 'test',
      });

      expect(result.data).toBe('success');
      expect(result.error).toBeUndefined();
    });

    it('should return error on failure', async () => {
      const fn = async () => {
        throw new Error('Test error');
      };
      const result = await safeAsync(fn, {
        module: 'Test',
        operation: 'test',
      });

      expect(result.data).toBeUndefined();
      expect(result.error).toBeDefined();
      expect(result.error?.error instanceof StardustError).toBe(true);
    });
  });

  describe('withRetry', () => {
    it('should succeed on first try', async () => {
      const fn = jest.fn(async () => 'success');
      const result = await withRetry(fn, {
        module: 'Test',
        operation: 'test',
      });

      expect(result).toBe('success');
      expect(fn).toHaveBeenCalledTimes(1);
    });

    it('should retry on failure', async () => {
      let attempts = 0;
      const fn = jest.fn(async () => {
        attempts++;
        if (attempts < 3) {
          throw new NetworkError('Network error');
        }
        return 'success';
      });

      const result = await withRetry(
        fn,
        { module: 'Test', operation: 'test' },
        { maxRetries: 3, delay: 10 }
      );

      expect(result).toBe('success');
      expect(fn).toHaveBeenCalledTimes(3);
    });

    it('should throw after max retries', async () => {
      const fn = jest.fn(async () => {
        throw new NetworkError('Network error');
      });

      await expect(
        withRetry(
          fn,
          { module: 'Test', operation: 'test' },
          { maxRetries: 2, delay: 10 }
        )
      ).rejects.toThrow();

      expect(fn).toHaveBeenCalledTimes(3); // initial + 2 retries
    });

    it('should not retry non-retryable errors', async () => {
      const fn = jest.fn(async () => {
        throw new AuthenticationError('密码错误');
      });

      await expect(
        withRetry(
          fn,
          { module: 'Test', operation: 'test' },
          { maxRetries: 3, delay: 10 }
        )
      ).rejects.toThrow();

      expect(fn).toHaveBeenCalledTimes(1); // no retries
    });
  });
});
