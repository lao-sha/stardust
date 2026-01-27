# 测试指南

> 日期: 2026-01-27
> 目标: 提升测试覆盖率到 60%+

## 当前状态

| 模块 | 测试文件 | 状态 |
|------|---------|------|
| lib/signer | ✅ | 已有测试 |
| lib/type-guards | ✅ | 新增测试 |
| lib/error-handler | ✅ | 新增测试 |
| stores/wallet | ✅ | 新增测试 |
| services/* | ❌ | 待添加 |
| components/* | ❌ | 待添加 |
| hooks/* | ❌ | 待添加 |

**当前覆盖率**: ~10%
**目标覆盖率**: 60%+

---

## 测试框架

### 使用的工具

- **Jest**: 测试运行器
- **@testing-library/react-hooks**: Hook 测试
- **@testing-library/react-native**: 组件测试

### 配置

```json
// package.json
{
  "scripts": {
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage"
  }
}
```

---

## 测试优先级

### P0 - 核心功能（必须测试）

1. **lib/** - 工具函数
   - ✅ type-guards.ts
   - ✅ error-handler.ts
   - ✅ signer.ts
   - ⏳ crypto.ts
   - ⏳ keystore.ts

2. **stores/** - 状态管理
   - ✅ wallet.store.ts
   - ⏳ trading.store.ts
   - ⏳ chat.store.ts

3. **services/** - 核心服务
   - ⏳ trading.service.ts
   - ⏳ divination-market.service.ts
   - ⏳ chat.service.ts

### P1 - 重要功能

4. **hooks/** - 自定义 Hooks
   - ⏳ useWallet.ts
   - ⏳ useErrorHandler.ts
   - ⏳ usePaginatedList.ts

5. **components/common/** - 通用组件
   - ⏳ Button.tsx
   - ⏳ Input.tsx
   - ⏳ Card.tsx

### P2 - 次要功能

6. **features/** - 功能模块
   - ⏳ wallet/
   - ⏳ trading/
   - ⏳ chat/

---

## 测试模式

### 1. 工具函数测试

```typescript
// src/lib/__tests__/utils.test.ts
describe('Utils', () => {
  describe('formatBalance', () => {
    it('should format balance correctly', () => {
      expect(formatBalance(1000000000000)).toBe('1.0000');
    });

    it('should handle zero', () => {
      expect(formatBalance(0)).toBe('0.0000');
    });
  });
});
```

### 2. Store 测试

```typescript
// src/stores/__tests__/example.store.test.ts
import { renderHook, act } from '@testing-library/react-hooks';
import { useExampleStore } from '../example.store';

describe('Example Store', () => {
  beforeEach(() => {
    // 重置 store
    useExampleStore.setState(initialState);
  });

  it('should update state', () => {
    const { result } = renderHook(() => useExampleStore());

    act(() => {
      result.current.updateValue('new value');
    });

    expect(result.current.value).toBe('new value');
  });
});
```

### 3. Service 测试

```typescript
// src/services/__tests__/example.service.test.ts
import { ExampleService } from '../example.service';

// Mock API
jest.mock('@/lib/api', () => ({
  getApi: jest.fn().mockReturnValue({
    query: {
      example: {
        data: jest.fn().mockResolvedValue({ value: 'test' }),
      },
    },
  }),
}));

describe('Example Service', () => {
  let service: ExampleService;

  beforeEach(() => {
    service = new ExampleService();
  });

  it('should fetch data', async () => {
    const data = await service.getData();
    expect(data).toEqual({ value: 'test' });
  });
});
```

### 4. Hook 测试

```typescript
// src/hooks/__tests__/useExample.test.ts
import { renderHook, act } from '@testing-library/react-hooks';
import { useExample } from '../useExample';

describe('useExample', () => {
  it('should return initial value', () => {
    const { result } = renderHook(() => useExample());
    expect(result.current.value).toBe('initial');
  });

  it('should update value', () => {
    const { result } = renderHook(() => useExample());

    act(() => {
      result.current.setValue('new');
    });

    expect(result.current.value).toBe('new');
  });
});
```

### 5. 组件测试

```typescript
// src/components/__tests__/Button.test.tsx
import React from 'react';
import { render, fireEvent } from '@testing-library/react-native';
import { Button } from '../Button';

describe('Button', () => {
  it('should render correctly', () => {
    const { getByText } = render(<Button title="Click me" />);
    expect(getByText('Click me')).toBeTruthy();
  });

  it('should call onPress', () => {
    const onPress = jest.fn();
    const { getByText } = render(
      <Button title="Click me" onPress={onPress} />
    );

    fireEvent.press(getByText('Click me'));
    expect(onPress).toHaveBeenCalled();
  });

  it('should be disabled', () => {
    const onPress = jest.fn();
    const { getByText } = render(
      <Button title="Click me" onPress={onPress} disabled />
    );

    fireEvent.press(getByText('Click me'));
    expect(onPress).not.toHaveBeenCalled();
  });
});
```

---

## Mock 策略

### 1. API Mock

```typescript
// __mocks__/@/lib/api.ts
export const getApi = jest.fn().mockReturnValue({
  query: {},
  tx: {},
  rpc: {},
});

export const isApiReady = jest.fn().mockReturnValue(true);
```

### 2. Storage Mock

```typescript
// __mocks__/@/lib/keystore.ts
export const storeEncryptedMnemonic = jest.fn().mockResolvedValue(undefined);
export const retrieveEncryptedMnemonic = jest.fn().mockResolvedValue('test mnemonic');
export const hasWallet = jest.fn().mockResolvedValue(false);
```

### 3. Navigation Mock

```typescript
// __mocks__/expo-router.ts
export const useRouter = jest.fn().mockReturnValue({
  push: jest.fn(),
  replace: jest.fn(),
  back: jest.fn(),
});
```

---

## 测试覆盖率目标

### 阶段 1（1周）- 基础覆盖

- [ ] lib/ - 80%+
- [ ] stores/ - 60%+
- [ ] 总覆盖率 - 30%+

### 阶段 2（2周）- 核心覆盖

- [ ] services/ - 60%+
- [ ] hooks/ - 60%+
- [ ] 总覆盖率 - 50%+

### 阶段 3（1个月）- 全面覆盖

- [ ] components/ - 50%+
- [ ] features/ - 40%+
- [ ] 总覆盖率 - 60%+

---

## 运行测试

```bash
# 运行所有测试
npm test

# 监听模式
npm run test:watch

# 生成覆盖率报告
npm run test:coverage

# 运行特定文件
npm test -- type-guards.test.ts

# 更新快照
npm test -- -u
```

---

## CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '18'
      - run: npm ci
      - run: npm test -- --coverage
      - uses: codecov/codecov-action@v2
```

---

## 最佳实践

### 1. 测试命名

```typescript
// ✅ 好的命名
describe('WalletStore', () => {
  describe('createWallet', () => {
    it('should create wallet with valid password', () => {});
    it('should reject weak passwords', () => {});
  });
});

// ❌ 不好的命名
describe('Test', () => {
  it('test1', () => {});
  it('test2', () => {});
});
```

### 2. 测试隔离

```typescript
// ✅ 每个测试独立
beforeEach(() => {
  // 重置状态
  useStore.setState(initialState);
});

// ❌ 测试相互依赖
it('test1', () => {
  store.setValue('test');
});

it('test2', () => {
  // 依赖 test1 的状态
  expect(store.value).toBe('test');
});
```

### 3. 异步测试

```typescript
// ✅ 使用 async/await
it('should fetch data', async () => {
  const data = await fetchData();
  expect(data).toBeDefined();
});

// ❌ 忘记 await
it('should fetch data', () => {
  const data = fetchData(); // Promise 对象
  expect(data).toBeDefined(); // 错误
});
```

### 4. Mock 清理

```typescript
// ✅ 清理 mock
afterEach(() => {
  jest.clearAllMocks();
});

// 或者
beforeEach(() => {
  jest.resetAllMocks();
});
```

---

## 常见问题

### Q: 如何测试 Polkadot.js API？

A: Mock API 返回值

```typescript
jest.mock('@/lib/api', () => ({
  getApi: jest.fn().mockReturnValue({
    query: {
      system: {
        account: jest.fn().mockResolvedValue({
          toJSON: () => ({ data: { free: '1000' } }),
        }),
      },
    },
  }),
}));
```

### Q: 如何测试 React Native 组件？

A: 使用 @testing-library/react-native

```typescript
import { render, fireEvent } from '@testing-library/react-native';

const { getByText, getByTestId } = render(<Component />);
fireEvent.press(getByText('Button'));
```

### Q: 如何测试 Zustand Store？

A: 使用 renderHook

```typescript
import { renderHook, act } from '@testing-library/react-hooks';

const { result } = renderHook(() => useStore());
act(() => {
  result.current.action();
});
```

---

## 总结

测试是保证代码质量的关键。按照优先级逐步添加测试：

1. **P0**: 核心工具和 Store（1周）
2. **P1**: 服务和 Hooks（2周）
3. **P2**: 组件和功能模块（1个月）

目标：**60%+ 测试覆盖率**
