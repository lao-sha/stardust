# Promise 处理风格指南

> 日期: 2026-01-27
> 目的: 统一项目中的 Promise 处理风格

## 推荐风格：async/await

### 为什么选择 async/await？

1. **可读性更好** - 代码看起来像同步代码
2. **错误处理统一** - 使用 try/catch
3. **调试更容易** - 堆栈跟踪更清晰
4. **避免回调地狱** - 扁平化代码结构

---

## 标准模式

### 1. 基本异步操作

```typescript
// ✅ 推荐：async/await
async function fetchData() {
  try {
    const result = await apiCall();
    return result;
  } catch (error) {
    console.error('Error:', error);
    throw error;
  }
}

// ❌ 不推荐：.then/.catch
function fetchData() {
  return apiCall()
    .then(result => result)
    .catch(error => {
      console.error('Error:', error);
      throw error;
    });
}
```

### 2. 多个并行请求

```typescript
// ✅ 推荐：Promise.all + async/await
async function fetchMultiple() {
  try {
    const [users, posts, comments] = await Promise.all([
      fetchUsers(),
      fetchPosts(),
      fetchComments(),
    ]);
    return { users, posts, comments };
  } catch (error) {
    console.error('Error:', error);
    throw error;
  }
}

// ❌ 不推荐：嵌套 .then
function fetchMultiple() {
  return Promise.all([
    fetchUsers(),
    fetchPosts(),
    fetchComments(),
  ]).then(([users, posts, comments]) => {
    return { users, posts, comments };
  }).catch(error => {
    console.error('Error:', error);
    throw error;
  });
}
```

### 3. 顺序执行

```typescript
// ✅ 推荐：async/await
async function processSequentially() {
  try {
    const user = await fetchUser();
    const profile = await fetchProfile(user.id);
    const settings = await fetchSettings(profile.id);
    return { user, profile, settings };
  } catch (error) {
    console.error('Error:', error);
    throw error;
  }
}

// ❌ 不推荐：链式 .then
function processSequentially() {
  return fetchUser()
    .then(user => fetchProfile(user.id)
      .then(profile => fetchSettings(profile.id)
        .then(settings => ({ user, profile, settings }))
      )
    )
    .catch(error => {
      console.error('Error:', error);
      throw error;
    });
}
```

---

## 特殊场景

### 1. Fire-and-forget（不关心结果）

```typescript
// ✅ 可以使用 .catch() 捕获错误
Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium).catch(err => {
  console.warn('[Haptic] Error:', err);
});

// 或者使用 async IIFE
(async () => {
  try {
    await Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
  } catch (err) {
    console.warn('[Haptic] Error:', err);
  }
})();
```

### 2. 事件处理器中的异步操作

```typescript
// ✅ 推荐：async 事件处理器
const handleSubmit = async () => {
  try {
    setLoading(true);
    await submitForm(data);
    showSuccess();
  } catch (error) {
    showError(error);
  } finally {
    setLoading(false);
  }
};

// ❌ 不推荐：.then/.catch
const handleSubmit = () => {
  setLoading(true);
  submitForm(data)
    .then(() => showSuccess())
    .catch(error => showError(error))
    .finally(() => setLoading(false));
};
```

### 3. useEffect 中的异步操作

```typescript
// ✅ 推荐：内部 async 函数
useEffect(() => {
  const loadData = async () => {
    try {
      const data = await fetchData();
      setData(data);
    } catch (error) {
      setError(error);
    }
  };
  
  loadData();
}, []);

// ❌ 不推荐：直接 .then
useEffect(() => {
  fetchData()
    .then(data => setData(data))
    .catch(error => setError(error));
}, []);
```

### 4. 订阅模式

```typescript
// ✅ 推荐：async/await + 清理
useEffect(() => {
  let unsubscribe: (() => void) | null = null;
  let isCancelled = false;
  
  const subscribe = async () => {
    try {
      const unsub = await service.subscribe(callback);
      if (!isCancelled) {
        unsubscribe = unsub;
      } else {
        unsub();
      }
    } catch (error) {
      console.error('Subscribe error:', error);
    }
  };
  
  subscribe();
  
  return () => {
    isCancelled = true;
    if (unsubscribe) unsubscribe();
  };
}, []);

// ❌ 不推荐：.then/.catch
useEffect(() => {
  let unsubscribe: (() => void) | null = null;
  
  service.subscribe(callback)
    .then(unsub => {
      unsubscribe = unsub;
    })
    .catch(error => {
      console.error('Subscribe error:', error);
    });
  
  return () => {
    if (unsubscribe) unsubscribe();
  };
}, []);
```

---

## 错误处理最佳实践

### 1. 统一错误处理

```typescript
// ✅ 使用 error-handler
import { handleError } from '@/lib/error-handler';

async function fetchData() {
  try {
    return await apiCall();
  } catch (error) {
    const handled = handleError(error, {
      module: 'DataService',
      operation: 'fetchData',
    });
    throw handled.error;
  }
}
```

### 2. Store 中的错误处理

```typescript
// ✅ 统一模式
fetchData: async () => {
  try {
    set({ loading: true, error: null });
    const data = await service.fetchData();
    set({ data, loading: false });
  } catch (error) {
    const message = error instanceof Error ? error.message : '获取数据失败';
    set({ error: message, loading: false });
    throw error;
  }
}
```

### 3. 组件中的错误处理

```typescript
// ✅ 使用 useErrorHandler hook
const { handleError } = useErrorHandler();

const loadData = async () => {
  try {
    const data = await fetchData();
    setData(data);
  } catch (error) {
    handleError(error, '加载数据失败');
  }
};
```

---

## 迁移指南

### 需要重构的文件

1. **src/hooks/useShake.ts**
   ```typescript
   // 当前
   Haptics.impactAsync(...).catch(err => {...});
   
   // 建议保持（fire-and-forget 场景）
   ```

2. **src/stores/trading.store.ts**
   ```typescript
   // 已修复 ✅
   ```

3. **src/services/chat.service.ts**
   ```typescript
   // 当前：混合使用 .then/.catch 和 async/await
   // 建议：统一为 async/await
   ```

### 重构优先级

- **P0**: Store 中的 Promise 处理（已完成）
- **P1**: Service 层的 Promise 处理
- **P2**: 组件中的 Promise 处理
- **P3**: Hook 中的 Promise 处理

---

## 代码审查检查项

在代码审查时，检查：

- [ ] 是否使用 async/await 而非 .then/.catch
- [ ] 是否有 try/catch 错误处理
- [ ] 是否使用统一的错误处理器
- [ ] useEffect 中的异步操作是否正确
- [ ] 是否有内存泄漏风险
- [ ] 错误消息是否用户友好

---

## 总结

**统一使用 async/await** 可以：
- 提高代码可读性
- 简化错误处理
- 减少 bug
- 提升维护性

**例外情况：**
- Fire-and-forget 操作可以使用 `.catch()`
- 但必须处理错误，不能忽略

**迁移策略：**
- 新代码必须使用 async/await
- 旧代码逐步重构
- 优先重构核心模块
