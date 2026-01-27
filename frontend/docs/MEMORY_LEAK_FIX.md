# 内存泄漏修复报告

> 日期: 2026-01-27
> 状态: 已修复

## 修复的问题

### 1. trading.store.ts - 订阅清理 ✅

**问题：** `subscribeToOrder` 返回的取消函数可能在异步订阅完成前被调用

**修复：**
```typescript
subscribeToOrder: (orderId: number) => {
  let unsubscribe: (() => void) | null = null;
  let isCancelled = false; // 添加取消标志

  tradingService.subscribeToOrder(orderId, (order) => {
    if (!isCancelled) { // 检查是否已取消
      set({ currentOrder: order });
    }
  }).then((unsub) => {
    if (!isCancelled) {
      unsubscribe = unsub;
    } else {
      unsub(); // 如果已取消，立即清理
    }
  });

  return () => {
    isCancelled = true;
    if (unsubscribe) unsubscribe();
  };
}
```

### 2. secure-storage.service.ts - 定时器清理 ✅

**状态：** 已有 `destroy()` 方法正确清理

```typescript
destroy(): void {
  this.clearCachedKey();
  if (this.autoLockTimer) {
    clearInterval(this.autoLockTimer);
    this.autoLockTimer = null;
  }
}
```

**使用建议：** 在应用退出或用户登出时调用 `destroy()`

### 3. signer.native.ts - 定时器清理 ✅

**状态：** `lock()` 方法已正确清理

```typescript
lock(): void {
  // ... 清理密钥对
  
  // 清除定时器
  if (this.autoLockTimer) {
    clearTimeout(this.autoLockTimer);
    this.autoLockTimer = null;
  }
}
```

### 4. CountdownTimer.tsx - 定时器清理 ✅

**状态：** useEffect 已正确返回清理函数

```typescript
useEffect(() => {
  const interval = setInterval(updateTimer, 1000);
  return () => clearInterval(interval); // ✅ 正确清理
}, [expireAt, onExpire, isExpired]);
```

### 5. ReleaseTimeoutAlert.tsx - 定时器和动画清理 ✅

**状态：** 两个 useEffect 都正确清理

```typescript
// 定时器清理
useEffect(() => {
  const interval = setInterval(updateElapsed, 1000);
  return () => clearInterval(interval); // ✅
}, [paidAt]);

// 动画清理
useEffect(() => {
  if (alertLevel === 'timeout') {
    const pulse = Animated.loop(...);
    pulse.start();
    return () => pulse.stop(); // ✅
  }
}, [alertLevel, pulseAnim]);
```

---

## 需要注意的使用模式

### 1. Store 订阅使用

```typescript
// ✅ 正确使用
useEffect(() => {
  const unsubscribe = useTradingStore.getState().subscribeToOrder(orderId);
  return () => unsubscribe(); // 组件卸载时清理
}, [orderId]);

// ❌ 错误使用
useEffect(() => {
  useTradingStore.getState().subscribeToOrder(orderId);
  // 缺少清理
}, [orderId]);
```

### 2. 服务清理

```typescript
// 应用退出时
const cleanup = () => {
  secureStorageService.destroy();
  mobileSigner.lock();
  // 其他服务清理...
};

// 在 App.tsx 中
useEffect(() => {
  return () => cleanup();
}, []);
```

### 3. 定时器使用规范

```typescript
// ✅ 正确模式
useEffect(() => {
  const timer = setInterval(() => {
    // ...
  }, 1000);
  
  return () => clearInterval(timer);
}, [dependencies]);

// ❌ 错误模式
useEffect(() => {
  setInterval(() => {
    // ...
  }, 1000);
  // 缺少清理
}, [dependencies]);
```

---

## 检查清单

在添加新功能时，检查以下项：

- [ ] 所有 `setInterval` 都有对应的 `clearInterval`
- [ ] 所有 `setTimeout` 都有对应的 `clearTimeout`
- [ ] 所有订阅都有取消订阅函数
- [ ] 所有动画都有 `stop()` 调用
- [ ] 所有事件监听器都有移除函数
- [ ] useEffect 返回清理函数
- [ ] 组件卸载时清理资源

---

## 测试建议

### 1. 内存泄漏测试

```typescript
// 使用 React DevTools Profiler
// 1. 打开组件
// 2. 关闭组件
// 3. 重复多次
// 4. 检查内存是否持续增长
```

### 2. 订阅测试

```typescript
// 测试订阅清理
it('should cleanup subscription on unmount', () => {
  const unsubscribe = jest.fn();
  const { unmount } = render(<Component />);
  
  unmount();
  
  expect(unsubscribe).toHaveBeenCalled();
});
```

---

## 总结

所有已知的内存泄漏风险已修复或确认安全。主要改进：

1. ✅ trading.store.ts 订阅清理逻辑增强
2. ✅ 所有定时器都有清理函数
3. ✅ 所有动画都有停止逻辑
4. ✅ 服务层提供 destroy() 方法

**建议：** 在代码审查时重点检查新增的定时器、订阅和事件监听器是否正确清理。
