/**
 * 星尘玄鉴 - 虚拟滚动列表组件
 * 
 * 基于 FlatList 实现高性能列表渲染：
 * - 虚拟化渲染（只渲染可见项）
 * - 分页加载
 * - 下拉刷新
 * - 上拉加载更多
 * - 空状态和错误状态
 * 
 * 注意：如需更高性能，可安装 @shopify/flash-list
 * npm install @shopify/flash-list
 */

import React, { useCallback, useState, useRef, useMemo } from 'react';
import {
  FlatList,
  View,
  Text,
  StyleSheet,
  RefreshControl,
  ActivityIndicator,
  TouchableOpacity,
  FlatListProps,
  ListRenderItem,
  ViewStyle,
  NativeSyntheticEvent,
  NativeScrollEvent,
} from 'react-native';

// ==================== 类型定义 ====================

export interface PaginationState {
  /** 当前页码 */
  page: number;
  /** 每页数量 */
  pageSize: number;
  /** 是否有更多数据 */
  hasMore: boolean;
  /** 总数量（可选） */
  total?: number;
}

export interface VirtualizedListProps<T> extends Omit<FlatListProps<T>, 'data' | 'renderItem'> {
  /** 列表数据 */
  data: T[];
  /** 渲染列表项 */
  renderItem: ListRenderItem<T>;
  /** 唯一键提取器 */
  keyExtractor: (item: T, index: number) => string;
  
  // 分页相关
  /** 分页状态 */
  pagination?: PaginationState;
  /** 加载更多回调 */
  onLoadMore?: () => Promise<void>;
  /** 是否正在加载更多 */
  isLoadingMore?: boolean;
  
  // 刷新相关
  /** 下拉刷新回调 */
  onRefresh?: () => Promise<void>;
  /** 是否正在刷新 */
  isRefreshing?: boolean;
  
  // 状态相关
  /** 是否正在加载（首次） */
  isLoading?: boolean;
  /** 错误信息 */
  error?: string | null;
  /** 重试回调 */
  onRetry?: () => void;
  
  // 自定义渲染
  /** 空状态组件 */
  emptyComponent?: React.ReactNode;
  /** 空状态文本 */
  emptyText?: string;
  /** 空状态图标 */
  emptyIcon?: string;
  /** 加载更多组件 */
  loadMoreComponent?: React.ReactNode;
  /** 列表头部 */
  headerComponent?: React.ReactNode;
  /** 列表尾部（在加载更多之前） */
  footerComponent?: React.ReactNode;
  
  // 性能优化
  /** 预估项目高度（用于优化） */
  estimatedItemSize?: number;
  /** 初始渲染数量 */
  initialNumToRender?: number;
  /** 窗口大小（渲染屏幕数） */
  windowSize?: number;
  /** 最大渲染数量 */
  maxToRenderPerBatch?: number;
  /** 移除不可见项的阈值 */
  removeClippedSubviews?: boolean;
  
  // 样式
  /** 容器样式 */
  containerStyle?: ViewStyle;
  /** 内容容器样式 */
  contentContainerStyle?: ViewStyle;
}

// ==================== 子组件 ====================

/** 加载中状态 */
function LoadingState(): React.ReactElement {
  return (
    <View style={styles.centerContainer}>
      <ActivityIndicator size="large" color="#e94560" />
      <Text style={styles.loadingText}>加载中...</Text>
    </View>
  );
}

/** 错误状态 */
function ErrorState({
  error,
  onRetry,
}: {
  error: string;
  onRetry?: () => void;
}): React.ReactElement {
  return (
    <View style={styles.centerContainer}>
      <Text style={styles.errorIcon}>⚠️</Text>
      <Text style={styles.errorText}>{error}</Text>
      {onRetry && (
        <TouchableOpacity style={styles.retryButton} onPress={onRetry}>
          <Text style={styles.retryButtonText}>重试</Text>
        </TouchableOpacity>
      )}
    </View>
  );
}

/** 空状态 */
function EmptyState({
  text = '暂无数据',
  icon = '📭',
}: {
  text?: string;
  icon?: string;
}): React.ReactElement {
  return (
    <View style={styles.centerContainer}>
      <Text style={styles.emptyIcon}>{icon}</Text>
      <Text style={styles.emptyText}>{text}</Text>
    </View>
  );
}

/** 加载更多指示器 */
function LoadMoreIndicator({
  isLoading,
  hasMore,
}: {
  isLoading: boolean;
  hasMore: boolean;
}): React.ReactElement | null {
  if (!hasMore) {
    return (
      <View style={styles.loadMoreContainer}>
        <Text style={styles.noMoreText}>— 没有更多了 —</Text>
      </View>
    );
  }

  if (isLoading) {
    return (
      <View style={styles.loadMoreContainer}>
        <ActivityIndicator size="small" color="#e94560" />
        <Text style={styles.loadMoreText}>加载中...</Text>
      </View>
    );
  }

  return null;
}

// ==================== 主组件 ====================

function VirtualizedListInner<T>(
  props: VirtualizedListProps<T>,
  ref: React.ForwardedRef<FlatList<T>>
): React.ReactElement {
  const {
    data,
    renderItem,
    keyExtractor,
    
    // 分页
    pagination,
    onLoadMore,
    isLoadingMore = false,
    
    // 刷新
    onRefresh,
    isRefreshing = false,
    
    // 状态
    isLoading = false,
    error,
    onRetry,
    
    // 自定义渲染
    emptyComponent,
    emptyText,
    emptyIcon,
    loadMoreComponent,
    headerComponent,
    footerComponent,
    
    // 性能优化
    estimatedItemSize = 80,
    initialNumToRender = 10,
    windowSize = 5,
    maxToRenderPerBatch = 10,
    removeClippedSubviews = true,
    
    // 样式
    containerStyle,
    contentContainerStyle,
    
    // 其他 FlatList props
    ...flatListProps
  } = props;

  const [isEndReached, setIsEndReached] = useState(false);
  const loadMoreLock = useRef(false);

  // 处理加载更多
  const handleEndReached = useCallback(async () => {
    if (
      !onLoadMore ||
      isLoadingMore ||
      loadMoreLock.current ||
      !pagination?.hasMore
    ) {
      return;
    }

    loadMoreLock.current = true;
    setIsEndReached(true);

    try {
      await onLoadMore();
    } finally {
      loadMoreLock.current = false;
      setIsEndReached(false);
    }
  }, [onLoadMore, isLoadingMore, pagination?.hasMore]);

  // 处理下拉刷新
  const handleRefresh = useCallback(async () => {
    if (!onRefresh || isRefreshing) return;
    await onRefresh();
  }, [onRefresh, isRefreshing]);

  // 渲染列表尾部
  const renderFooter = useCallback(() => {
    return (
      <View>
        {footerComponent}
        {loadMoreComponent ?? (
          <LoadMoreIndicator
            isLoading={isLoadingMore || isEndReached}
            hasMore={pagination?.hasMore ?? true}
          />
        )}
      </View>
    );
  }, [footerComponent, loadMoreComponent, isLoadingMore, isEndReached, pagination?.hasMore]);

  // 渲染空状态
  const renderEmpty = useCallback(() => {
    if (isLoading) return null;
    
    return emptyComponent ?? (
      <EmptyState text={emptyText} icon={emptyIcon} />
    );
  }, [isLoading, emptyComponent, emptyText, emptyIcon]);

  // 刷新控制
  const refreshControl = useMemo(() => {
    if (!onRefresh) return undefined;
    
    return (
      <RefreshControl
        refreshing={isRefreshing}
        onRefresh={handleRefresh}
        colors={['#e94560']}
        tintColor="#e94560"
        title="下拉刷新"
        titleColor="#666666"
      />
    );
  }, [onRefresh, isRefreshing, handleRefresh]);

  // 首次加载状态
  if (isLoading && data.length === 0) {
    return (
      <View style={[styles.container, containerStyle]}>
        <LoadingState />
      </View>
    );
  }

  // 错误状态
  if (error && data.length === 0) {
    return (
      <View style={[styles.container, containerStyle]}>
        <ErrorState error={error} onRetry={onRetry} />
      </View>
    );
  }

  return (
    <View style={[styles.container, containerStyle]}>
      <FlatList
        ref={ref}
        data={data}
        renderItem={renderItem}
        keyExtractor={keyExtractor}
        
        // 头部和尾部
        ListHeaderComponent={headerComponent as React.ComponentType | undefined}
        ListFooterComponent={renderFooter}
        ListEmptyComponent={renderEmpty}
        
        // 刷新
        refreshControl={refreshControl}
        
        // 加载更多
        onEndReached={handleEndReached}
        onEndReachedThreshold={0.3}
        
        // 性能优化
        initialNumToRender={initialNumToRender}
        windowSize={windowSize}
        maxToRenderPerBatch={maxToRenderPerBatch}
        removeClippedSubviews={removeClippedSubviews}
        getItemLayout={
          estimatedItemSize
            ? (_, index) => ({
                length: estimatedItemSize,
                offset: estimatedItemSize * index,
                index,
              })
            : undefined
        }
        
        // 样式
        contentContainerStyle={[
          styles.contentContainer,
          data.length === 0 && styles.emptyContentContainer,
          contentContainerStyle,
        ]}
        
        // 其他属性
        showsVerticalScrollIndicator={false}
        {...flatListProps}
      />
    </View>
  );
}

// 使用 forwardRef 支持 ref
export const VirtualizedList = React.forwardRef(VirtualizedListInner) as <T>(
  props: VirtualizedListProps<T> & { ref?: React.ForwardedRef<FlatList<T>> }
) => React.ReactElement;

// ==================== 样式 ====================

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#1a1a2e',
  },
  contentContainer: {
    flexGrow: 1,
  },
  emptyContentContainer: {
    flex: 1,
    justifyContent: 'center',
  },
  centerContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 40,
  },
  
  // 加载状态
  loadingText: {
    marginTop: 12,
    fontSize: 14,
    color: '#666666',
  },
  
  // 错误状态
  errorIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  errorText: {
    fontSize: 16,
    color: '#ff6b6b',
    textAlign: 'center',
    marginBottom: 20,
  },
  retryButton: {
    backgroundColor: '#e94560',
    paddingHorizontal: 24,
    paddingVertical: 10,
    borderRadius: 8,
  },
  retryButtonText: {
    color: '#ffffff',
    fontSize: 14,
    fontWeight: '600',
  },
  
  // 空状态
  emptyIcon: {
    fontSize: 48,
    marginBottom: 16,
  },
  emptyText: {
    fontSize: 16,
    color: '#666666',
    textAlign: 'center',
  },
  
  // 加载更多
  loadMoreContainer: {
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    paddingVertical: 16,
    gap: 8,
  },
  loadMoreText: {
    fontSize: 14,
    color: '#666666',
  },
  noMoreText: {
    fontSize: 12,
    color: '#444444',
  },
});

export default VirtualizedList;
