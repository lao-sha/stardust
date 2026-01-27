/**
 * 星尘玄鉴 - 分页列表 Hook
 * 
 * 提供分页数据加载的完整状态管理：
 * - 首次加载
 * - 下拉刷新
 * - 上拉加载更多
 * - 错误处理
 * - 缓存管理
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import type { PaginationState } from '@/components/VirtualizedList';

// ==================== 类型定义 ====================

export interface UsePaginatedListOptions<T> {
  /** 数据获取函数 */
  fetchData: (page: number, pageSize: number) => Promise<{
    data: T[];
    total?: number;
    hasMore?: boolean;
  }>;
  /** 每页数量 */
  pageSize?: number;
  /** 是否自动加载首页 */
  autoLoad?: boolean;
  /** 唯一键提取器（用于去重） */
  getItemKey?: (item: T) => string | number;
  /** 缓存键（用于持久化） */
  cacheKey?: string;
}

export interface UsePaginatedListReturn<T> {
  /** 列表数据 */
  data: T[];
  /** 分页状态 */
  pagination: PaginationState;
  /** 是否首次加载中 */
  isLoading: boolean;
  /** 是否刷新中 */
  isRefreshing: boolean;
  /** 是否加载更多中 */
  isLoadingMore: boolean;
  /** 错误信息 */
  error: string | null;
  
  /** 加载首页 */
  loadFirstPage: () => Promise<void>;
  /** 刷新（重新加载首页） */
  refresh: () => Promise<void>;
  /** 加载更多 */
  loadMore: () => Promise<void>;
  /** 重试 */
  retry: () => Promise<void>;
  /** 清空数据 */
  clear: () => void;
  /** 更新单个项目 */
  updateItem: (key: string | number, updater: (item: T) => T) => void;
  /** 删除单个项目 */
  removeItem: (key: string | number) => void;
  /** 在头部添加项目 */
  prependItem: (item: T) => void;
  /** 在尾部添加项目 */
  appendItem: (item: T) => void;
}

// ==================== Hook 实现 ====================

export function usePaginatedList<T>(
  options: UsePaginatedListOptions<T>
): UsePaginatedListReturn<T> {
  const {
    fetchData,
    pageSize = 20,
    autoLoad = true,
    getItemKey,
  } = options;

  // 状态
  const [data, setData] = useState<T[]>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    page: 0,
    pageSize,
    hasMore: true,
  });
  const [isLoading, setIsLoading] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 防止重复请求
  const loadingRef = useRef(false);
  const mountedRef = useRef(true);

  // 组件卸载时标记
  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
    };
  }, []);

  // 去重合并数据
  const mergeData = useCallback(
    (existingData: T[], newData: T[], prepend = false): T[] => {
      if (!getItemKey) {
        return prepend ? [...newData, ...existingData] : [...existingData, ...newData];
      }

      const existingKeys = new Set(existingData.map(getItemKey));
      const uniqueNewData = newData.filter(item => !existingKeys.has(getItemKey(item)));

      return prepend
        ? [...uniqueNewData, ...existingData]
        : [...existingData, ...uniqueNewData];
    },
    [getItemKey]
  );

  // 加载首页
  const loadFirstPage = useCallback(async () => {
    if (loadingRef.current) return;
    loadingRef.current = true;
    setIsLoading(true);
    setError(null);

    try {
      const result = await fetchData(1, pageSize);
      
      if (!mountedRef.current) return;

      setData(result.data);
      setPagination({
        page: 1,
        pageSize,
        hasMore: result.hasMore ?? result.data.length >= pageSize,
        total: result.total,
      });
    } catch (err) {
      if (!mountedRef.current) return;
      
      const message = err instanceof Error ? err.message : '加载失败';
      setError(message);
    } finally {
      if (mountedRef.current) {
        setIsLoading(false);
      }
      loadingRef.current = false;
    }
  }, [fetchData, pageSize]);

  // 刷新
  const refresh = useCallback(async () => {
    if (loadingRef.current) return;
    loadingRef.current = true;
    setIsRefreshing(true);
    setError(null);

    try {
      const result = await fetchData(1, pageSize);
      
      if (!mountedRef.current) return;

      setData(result.data);
      setPagination({
        page: 1,
        pageSize,
        hasMore: result.hasMore ?? result.data.length >= pageSize,
        total: result.total,
      });
    } catch (err) {
      if (!mountedRef.current) return;
      
      const message = err instanceof Error ? err.message : '刷新失败';
      setError(message);
    } finally {
      if (mountedRef.current) {
        setIsRefreshing(false);
      }
      loadingRef.current = false;
    }
  }, [fetchData, pageSize]);

  // 加载更多
  const loadMore = useCallback(async () => {
    if (loadingRef.current || !pagination.hasMore) return;
    loadingRef.current = true;
    setIsLoadingMore(true);

    try {
      const nextPage = pagination.page + 1;
      const result = await fetchData(nextPage, pageSize);
      
      if (!mountedRef.current) return;

      setData(prev => mergeData(prev, result.data));
      setPagination(prev => ({
        ...prev,
        page: nextPage,
        hasMore: result.hasMore ?? result.data.length >= pageSize,
        total: result.total ?? prev.total,
      }));
    } catch (err) {
      // 加载更多失败不显示全局错误，只在控制台记录
      console.warn('[usePaginatedList] Load more failed:', err);
    } finally {
      if (mountedRef.current) {
        setIsLoadingMore(false);
      }
      loadingRef.current = false;
    }
  }, [fetchData, pageSize, pagination.page, pagination.hasMore, mergeData]);

  // 重试
  const retry = useCallback(async () => {
    if (data.length === 0) {
      await loadFirstPage();
    } else {
      await refresh();
    }
  }, [data.length, loadFirstPage, refresh]);

  // 清空数据
  const clear = useCallback(() => {
    setData([]);
    setPagination({
      page: 0,
      pageSize,
      hasMore: true,
    });
    setError(null);
  }, [pageSize]);

  // 更新单个项目
  const updateItem = useCallback(
    (key: string | number, updater: (item: T) => T) => {
      if (!getItemKey) {
        console.warn('[usePaginatedList] updateItem requires getItemKey');
        return;
      }

      setData(prev =>
        prev.map(item => (getItemKey(item) === key ? updater(item) : item))
      );
    },
    [getItemKey]
  );

  // 删除单个项目
  const removeItem = useCallback(
    (key: string | number) => {
      if (!getItemKey) {
        console.warn('[usePaginatedList] removeItem requires getItemKey');
        return;
      }

      setData(prev => prev.filter(item => getItemKey(item) !== key));
    },
    [getItemKey]
  );

  // 在头部添加项目
  const prependItem = useCallback(
    (item: T) => {
      setData(prev => mergeData(prev, [item], true));
    },
    [mergeData]
  );

  // 在尾部添加项目
  const appendItem = useCallback(
    (item: T) => {
      setData(prev => mergeData(prev, [item], false));
    },
    [mergeData]
  );

  // 自动加载首页
  useEffect(() => {
    if (autoLoad) {
      loadFirstPage();
    }
  }, [autoLoad, loadFirstPage]);

  return {
    data,
    pagination,
    isLoading,
    isRefreshing,
    isLoadingMore,
    error,
    loadFirstPage,
    refresh,
    loadMore,
    retry,
    clear,
    updateItem,
    removeItem,
    prependItem,
    appendItem,
  };
}

// ==================== 简化版 Hook ====================

/**
 * 简化版分页 Hook，适用于简单场景
 */
export function useSimplePagination<T>(
  fetchFn: (page: number) => Promise<T[]>,
  options?: {
    pageSize?: number;
    autoLoad?: boolean;
  }
) {
  const pageSize = options?.pageSize ?? 20;

  return usePaginatedList<T>({
    fetchData: async (page) => {
      const data = await fetchFn(page);
      return {
        data,
        hasMore: data.length >= pageSize,
      };
    },
    pageSize,
    autoLoad: options?.autoLoad ?? true,
  });
}

export default usePaginatedList;
