/**
 * 星尘玄鉴 - 应用根布局
 * 主题色：金棕色 #B2955D
 */

import { Slot } from 'expo-router';
import { View, StyleSheet, Text } from 'react-native';
import { StatusBar } from 'expo-status-bar';
import React, { useEffect } from 'react';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { disableConsoleInProduction, createLogger, configureLogger } from '@/lib/logger';
import { configureErrorReporter } from '@/lib/error-handler';
import { 
  initErrorReporting, 
  createRemoteLogger,
  setTag,
} from '@/services/error-reporting.service';
import { initWebSecurity } from '@/lib/security/init';

// 生产环境禁用 console.log/debug/info
disableConsoleInProduction();

const log = createLogger('RootLayout');

// 主题色
const THEME_BG = '#F5F5F7';
const DARK_BG = '#0a0a0f';

export default function RootLayout() {
  log.debug('Rendering...');
  
  // 初始化服务
  useEffect(() => {
    const initServices = async () => {
      try {
        // 1. 初始化 Web 安全功能（P0 修复）
        await initWebSecurity();
        
        // 2. 初始化错误上报（生产环境）
        await initErrorReporting({
          // DSN 从环境变量获取，未配置时使用本地日志
          dsn: process.env.EXPO_PUBLIC_SENTRY_DSN,
          environment: __DEV__ ? 'development' : 'production',
          release: '1.0.0',
        });
        
        // 3. 配置日志服务使用远程上报
        configureLogger({
          remoteLogger: createRemoteLogger(),
        });
        
        // 4. 设置应用标签
        setTag('app', 'stardust-mobile');
        
        log.info('Services initialized');
      } catch (error) {
        log.warn('Failed to initialize services:', error);
      }
    };
    
    initServices();
  }, []);
  
  return (
    <ErrorBoundary module="RootLayout">
      <View style={styles.container}>
        <StatusBar style="light" />
        <View style={styles.content}>
          <Slot />
        </View>
      </View>
    </ErrorBoundary>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: DARK_BG,
  },
  content: {
    flex: 1,
    backgroundColor: THEME_BG,
  },
});

declare const __DEV__: boolean;
