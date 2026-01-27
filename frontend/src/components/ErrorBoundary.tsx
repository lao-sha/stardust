/**
 * 星尘玄鉴 - 全局错误边界组件
 * 
 * 捕获 React 组件树中的 JavaScript 错误，
 * 显示友好的错误界面，并上报错误信息。
 */

import React, { Component, ErrorInfo, ReactNode } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  ScrollView,
} from 'react-native';
import { handleError, ErrorSeverity } from '@/lib/error-handler';
import { StardustError } from '@/lib/errors';

// ==================== 类型定义 ====================

interface ErrorBoundaryProps {
  /** 子组件 */
  children: ReactNode;
  /** 自定义错误渲染 */
  fallback?: ReactNode | ((error: Error, reset: () => void) => ReactNode);
  /** 错误发生时的回调 */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  /** 模块名称（用于错误上报） */
  module?: string;
  /** 是否显示错误详情（开发模式） */
  showDetails?: boolean;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

// ==================== 错误边界组件 ====================

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    this.setState({ errorInfo });

    // 使用统一错误处理
    handleError(error, {
      module: this.props.module ?? 'ErrorBoundary',
      operation: 'componentDidCatch',
      metadata: {
        componentStack: errorInfo.componentStack,
      },
    });

    // 调用自定义回调
    this.props.onError?.(error, errorInfo);
  }

  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render(): ReactNode {
    const { hasError, error, errorInfo } = this.state;
    const { children, fallback, showDetails } = this.props;

    if (hasError && error) {
      // 使用自定义 fallback
      if (fallback) {
        if (typeof fallback === 'function') {
          return fallback(error, this.handleReset);
        }
        return fallback;
      }

      // 默认错误界面
      return (
        <DefaultErrorFallback
          error={error}
          errorInfo={errorInfo}
          onReset={this.handleReset}
          showDetails={showDetails ?? __DEV__}
        />
      );
    }

    return children;
  }
}

// ==================== 默认错误界面 ====================

interface DefaultErrorFallbackProps {
  error: Error;
  errorInfo: ErrorInfo | null;
  onReset: () => void;
  showDetails: boolean;
}

function DefaultErrorFallback({
  error,
  errorInfo,
  onReset,
  showDetails,
}: DefaultErrorFallbackProps): React.ReactElement {
  const isStardustError = error instanceof StardustError;
  const errorCode = isStardustError ? (error as StardustError).code : undefined;

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        {/* 错误图标 */}
        <Text style={styles.icon}>⚠️</Text>

        {/* 错误标题 */}
        <Text style={styles.title}>出错了</Text>

        {/* 错误消息 */}
        <Text style={styles.message}>
          {getUserFriendlyMessage(error)}
        </Text>

        {/* 错误代码 */}
        {errorCode && (
          <Text style={styles.errorCode}>错误代码: {errorCode}</Text>
        )}

        {/* 操作按钮 */}
        <View style={styles.buttonContainer}>
          <TouchableOpacity
            style={styles.primaryButton}
            onPress={onReset}
            accessibilityRole="button"
            accessibilityLabel="重试"
          >
            <Text style={styles.primaryButtonText}>重试</Text>
          </TouchableOpacity>
        </View>

        {/* 开发模式显示详情 */}
        {showDetails && (
          <ScrollView style={styles.detailsContainer}>
            <Text style={styles.detailsTitle}>错误详情（仅开发模式可见）</Text>
            <Text style={styles.detailsText}>
              {error.name}: {error.message}
            </Text>
            {error.stack && (
              <Text style={styles.stackTrace}>{error.stack}</Text>
            )}
            {errorInfo?.componentStack && (
              <>
                <Text style={styles.detailsTitle}>组件堆栈</Text>
                <Text style={styles.stackTrace}>
                  {errorInfo.componentStack}
                </Text>
              </>
            )}
          </ScrollView>
        )}
      </View>
    </View>
  );
}

// ==================== 辅助函数 ====================

function getUserFriendlyMessage(error: Error): string {
  if (error instanceof StardustError) {
    const code = error.code;
    
    switch (code) {
      case 'WALLET_ERROR':
        return '钱包操作失败，请稍后重试';
      case 'AUTH_ERROR':
        return '认证失败，请检查密码';
      case 'NETWORK_ERROR':
      case 'API_CONNECTION_ERROR':
        return '网络连接失败，请检查网络设置';
      case 'TRANSACTION_ERROR':
        return '交易处理失败，请稍后重试';
      default:
        return error.message || '发生未知错误';
    }
  }

  // 通用错误消息
  const message = error.message?.toLowerCase() ?? '';
  
  if (message.includes('network') || message.includes('fetch')) {
    return '网络连接失败，请检查网络设置';
  }
  
  if (message.includes('timeout')) {
    return '操作超时，请稍后重试';
  }

  return '应用遇到问题，请重试或联系支持';
}

// ==================== 样式 ====================

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#1a1a2e',
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  content: {
    backgroundColor: '#16213e',
    borderRadius: 16,
    padding: 24,
    width: '100%',
    maxWidth: 400,
    alignItems: 'center',
  },
  icon: {
    fontSize: 48,
    marginBottom: 16,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#ffffff',
    marginBottom: 12,
  },
  message: {
    fontSize: 16,
    color: '#a0a0a0',
    textAlign: 'center',
    marginBottom: 8,
    lineHeight: 24,
  },
  errorCode: {
    fontSize: 12,
    color: '#666666',
    marginBottom: 24,
  },
  buttonContainer: {
    flexDirection: 'row',
    gap: 12,
    marginTop: 8,
  },
  primaryButton: {
    backgroundColor: '#e94560',
    paddingHorizontal: 32,
    paddingVertical: 12,
    borderRadius: 8,
  },
  primaryButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
  detailsContainer: {
    marginTop: 24,
    maxHeight: 200,
    width: '100%',
    backgroundColor: '#0f0f23',
    borderRadius: 8,
    padding: 12,
  },
  detailsTitle: {
    fontSize: 12,
    fontWeight: 'bold',
    color: '#e94560',
    marginBottom: 8,
    marginTop: 8,
  },
  detailsText: {
    fontSize: 12,
    color: '#ff6b6b',
    fontFamily: 'monospace',
  },
  stackTrace: {
    fontSize: 10,
    color: '#666666',
    fontFamily: 'monospace',
    marginTop: 4,
  },
});

// ==================== 高阶组件 ====================

/**
 * 使用错误边界包装组件的高阶组件
 */
export function withErrorBoundary<P extends object>(
  WrappedComponent: React.ComponentType<P>,
  options?: Omit<ErrorBoundaryProps, 'children'>
): React.FC<P> {
  const displayName = WrappedComponent.displayName || WrappedComponent.name || 'Component';

  const WithErrorBoundary: React.FC<P> = (props) => (
    <ErrorBoundary module={displayName} {...options}>
      <WrappedComponent {...props} />
    </ErrorBoundary>
  );

  WithErrorBoundary.displayName = `withErrorBoundary(${displayName})`;

  return WithErrorBoundary;
}

// ==================== 全局变量声明 ====================

declare const __DEV__: boolean;

export default ErrorBoundary;
