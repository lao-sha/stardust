# 快速优化指南

**目标**: 5分钟内完成一个页面的优化

---

## 🎯 优化检查清单

### ✅ 第一步：导入替换（30秒）

```typescript
// ❌ 删除这些
import { ActivityIndicator, TextInput, Pressable } from 'react-native';

// ✅ 添加这些
import { Card, Button, Input, LoadingSpinner, EmptyState } from '@/components/common';
import { useWallet, useAsync, useClipboard } from '@/hooks';
import { xxxService } from '@/services'; // 根据需要导入
```

---

### ✅ 第二步：状态管理替换（1分钟）

```typescript
// ❌ 旧方式
const [isLoading, setIsLoading] = useState(false);
const [error, setError] = useState<string | null>(null);
const { address } = useWalletStore();
const balance = '0.00'; // 假数据

// ✅ 新方式
const { address, balance, isUnlocked, ensureUnlocked } = useWallet();
const { execute, isLoading, error } = useAsync();
```

---

### ✅ 第三步：UI组件替换（2分钟）

#### 卡片组件
```typescript
// ❌ 旧方式
<View style={styles.card}>
  {children}
</View>

// ✅ 新方式
<Card>
  {children}
</Card>

// 删除 styles.card 相关样式
```

#### 输入框组件
```typescript
// ❌ 旧方式
<View style={styles.inputGroup}>
  <Text style={styles.label}>标签</Text>
  <TextInput
    style={styles.input}
    value={value}
    onChangeText={setValue}
    placeholder="请输入"
  />
</View>

// ✅ 新方式
<Input
  label="标签"
  value={value}
  onChangeText={setValue}
  placeholder="请输入"
  error={errorMessage} // 可选
/>

// 删除 styles.inputGroup, styles.label, styles.input
```

#### 按钮组件
```typescript
// ❌ 旧方式
<Pressable
  style={[styles.button, isLoading && styles.buttonDisabled]}
  onPress={handleSubmit}
  disabled={isLoading}
>
  {isLoading ? (
    <ActivityIndicator color="#FFF" />
  ) : (
    <Text style={styles.buttonText}>提交</Text>
  )}
</Pressable>

// ✅ 新方式
<Button
  title="提交"
  onPress={handleSubmit}
  loading={isLoading}
  disabled={!isValid}
/>

// 删除 styles.button, styles.buttonText, styles.buttonDisabled
```

#### 加载状态
```typescript
// ❌ 旧方式
{isLoading && (
  <View style={styles.loading}>
    <ActivityIndicator size="large" color="#B2955D" />
    <Text style={styles.loadingText}>加载中...</Text>
  </View>
)}

// ✅ 新方式
{isLoading && <LoadingSpinner text="加载中..." />}

// 删除 styles.loading, styles.loadingText
```

#### 空状态
```typescript
// ❌ 旧方式
{data.length === 0 && (
  <View style={styles.empty}>
    <Text style={styles.emptyText}>暂无数据</Text>
  </View>
)}

// ✅ 新方式
{data.length === 0 && (
  <EmptyState
    icon="file-tray-outline"
    title="暂无数据"
    description="请稍后再试"
  />
)}

// 删除 styles.empty, styles.emptyText
```

---

### ✅ 第四步：集成真实服务（1.5分钟）

#### 模式1：简单查询
```typescript
// ❌ 旧方式
useEffect(() => {
  // TODO: 从链上获取数据
  setTimeout(() => {
    setData(mockData);
  }, 500);
}, []);

// ✅ 新方式
useEffect(() => {
  if (address) {
    loadData();
  }
}, [address]);

const loadData = async () => {
  try {
    await execute(async () => {
      const result = await someService.getData(address!);
      setData(result);
    });
  } catch (error) {
    Alert.alert('错误', '加载数据失败');
  }
};
```

#### 模式2：提交交易
```typescript
// ❌ 旧方式
const handleSubmit = async () => {
  setIsLoading(true);
  try {
    // TODO: 调用链上方法
    await new Promise(resolve => setTimeout(resolve, 2000));
    Alert.alert('提示', '功能即将上线');
  } catch (error) {
    Alert.alert('失败', '请稍后重试');
  } finally {
    setIsLoading(false);
  }
};

// ✅ 新方式
const handleSubmit = async () => {
  // 确保钱包已解锁
  const unlocked = await ensureUnlocked();
  if (!unlocked) {
    Alert.alert('提示', '请先解锁钱包');
    return;
  }

  try {
    await execute(async () => {
      const result = await someService.submitTransaction(params);
      Alert.alert('成功', '操作已完成');
    });
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : '操作失败';
    Alert.alert('错误', errorMessage);
  }
};
```

---

## 🔥 常见服务调用

### 钱包相关
```typescript
// 转账
import { getApi } from '@/lib/api';
import { signAndSend } from '@/lib/signer';

const api = await getApi();
const amountBigInt = BigInt(Math.floor(parseFloat(amount) * 1e12));
const tx = api.tx.balances.transfer(recipient, amountBigInt.toString());
await signAndSend(api, tx, address!, (status) => {
  console.log('Status:', status);
});
```

### Bridge相关
```typescript
import { bridgeService } from '@/services/bridge.service';

// 做市商兑换
const swapId = await bridgeService.makerSwap(
  makerId,
  dustAmountBigInt,
  tronAddress,
  (status) => setTxStatus(status)
);

// 获取价格
const price = await bridgeService.getDustPrice();

// 获取历史
const history = await bridgeService.getSwapHistory(address);
```

### Trading相关
```typescript
import { tradingService } from '@/services/trading.service';

// 获取做市商列表
const makers = await tradingService.getMakers();

// 创建订单
const orderId = await tradingService.createOrder(
  address,
  makerId,
  dustAmount,
  paymentCommit,
  contactCommit,
  (status) => setTxStatus(status)
);

// 获取价格
const price = await tradingService.getDustPrice();
```

### Divination相关
```typescript
import { divinationService } from '@/services/divination.service';

// 保存占卜结果
const recordId = await divinationService.saveDivination(
  address,
  DivinationType.Bazi,
  resultData,
  (status) => setTxStatus(status)
);

// 获取历史记录
const records = await divinationService.getDivinationHistory(
  address,
  DivinationType.Bazi
);

// 获取统计
const stats = await divinationService.getDivinationStats(address);
```

---

## 📋 完整示例

### 优化前（200行）
```typescript
import { useState } from 'react';
import { View, Text, TextInput, Pressable, ActivityIndicator, Alert } from 'react-native';
import { useWalletStore } from '@/stores';

export default function OldPage() {
  const { address } = useWalletStore();
  const [value, setValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const balance = '0.00'; // 假数据

  const handleSubmit = async () => {
    if (!value) {
      Alert.alert('提示', '请输入内容');
      return;
    }

    setIsLoading(true);
    try {
      // TODO: 调用链上方法
      await new Promise(resolve => setTimeout(resolve, 2000));
      Alert.alert('提示', '功能即将上线');
    } catch (error) {
      Alert.alert('失败', '请稍后重试');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.card}>
        <View style={styles.inputGroup}>
          <Text style={styles.label}>输入内容</Text>
          <TextInput
            style={styles.input}
            value={value}
            onChangeText={setValue}
            placeholder="请输入"
          />
        </View>

        <Pressable
          style={[styles.button, isLoading && styles.buttonDisabled]}
          onPress={handleSubmit}
          disabled={isLoading}
        >
          {isLoading ? (
            <ActivityIndicator color="#FFF" />
          ) : (
            <Text style={styles.buttonText}>提交</Text>
          )}
        </Pressable>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
  card: { backgroundColor: '#FFF', borderRadius: 12, padding: 16 },
  inputGroup: { marginBottom: 16 },
  label: { fontSize: 14, marginBottom: 8 },
  input: { borderWidth: 1, borderColor: '#DDD', borderRadius: 8, padding: 12 },
  button: { backgroundColor: '#B2955D', padding: 16, borderRadius: 8, alignItems: 'center' },
  buttonDisabled: { opacity: 0.6 },
  buttonText: { color: '#FFF', fontSize: 16, fontWeight: '600' },
});
```

### 优化后（120行，减少40%）
```typescript
import { useState } from 'react';
import { View, Alert } from 'react-native';
import { Card, Button, Input } from '@/components/common';
import { useWallet, useAsync } from '@/hooks';
import { someService } from '@/services';

export default function NewPage() {
  const { address, balance, ensureUnlocked } = useWallet();
  const { execute, isLoading } = useAsync();
  const [value, setValue] = useState('');
  const [error, setError] = useState('');

  const handleSubmit = async () => {
    // 验证
    if (!value) {
      setError('请输入内容');
      return;
    }

    // 确保钱包已解锁
    const unlocked = await ensureUnlocked();
    if (!unlocked) {
      Alert.alert('提示', '请先解锁钱包');
      return;
    }

    try {
      await execute(async () => {
        const result = await someService.submit(address!, value);
        Alert.alert('成功', '操作已完成');
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '操作失败';
      Alert.alert('错误', errorMessage);
    }
  };

  return (
    <View style={styles.container}>
      <Card>
        <Input
          label="输入内容"
          value={value}
          onChangeText={(text) => {
            setValue(text);
            setError('');
          }}
          error={error}
          placeholder="请输入"
        />

        <Button
          title="提交"
          onPress={handleSubmit}
          loading={isLoading}
          disabled={!value}
        />
      </Card>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16 },
});
```

---

## ⚡ 快捷键提示

### VS Code 快捷键
- `Ctrl+D` - 选中下一个相同内容
- `Ctrl+Shift+L` - 选中所有相同内容
- `Alt+Click` - 多光标编辑
- `Ctrl+/` - 注释/取消注释

### 批量替换技巧
1. 选中 `<View style={styles.card}>` 
2. `Ctrl+Shift+L` 选中所有
3. 替换为 `<Card>`
4. 手动调整闭合标签

---

## 🎓 最佳实践

### DO ✅
- 使用通用组件
- 使用自定义Hooks
- 集成真实服务
- 添加错误处理
- 添加加载状态
- 添加表单验证

### DON'T ❌
- 不要重复造轮子
- 不要硬编码假数据
- 不要忽略错误处理
- 不要跳过钱包解锁检查
- 不要使用 setTimeout 模拟异步

---

## 📞 需要帮助？

遇到问题时：
1. 查看 `docs/OPTIMIZATION_COMPLETE.md` 了解详细案例
2. 参考已优化的页面（transfer.tsx, bridge/maker.tsx）
3. 查看服务文档（src/services/）
4. 查看组件文档（src/components/common/）

---

**记住**: 优化不是一次性的，而是持续的过程。每次优化一个页面，积累经验，逐步提升整体代码质量！
