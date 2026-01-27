# 批量优化脚本指南

由于页面数量较多（40+个），手动逐个优化效率较低。建议采用以下策略：

## 策略1：按模块分批优化

### 第一批：占卜页面（8个）- 已完成示例
- ✅ bazi.tsx - 参考模板
- ziwei.tsx
- qimen.tsx
- liuyao.tsx
- meihua.tsx
- tarot.tsx
- daliuren.tsx
- xiaoliuren.tsx

**优化模式**：
1. 使用 DivinationTemplate 组件
2. 提取表单渲染逻辑
3. 提取结果渲染逻辑
4. 集成 divinationService

### 第二批：市场相关（8个）
- market/order/create.tsx
- market/order/list.tsx
- market/order/[id].tsx
- market/provider/[id].tsx
- market/provider/register.tsx
- market/review/create.tsx
- market/search.tsx
- market/privacy-settings.tsx

**优化模式**：
1. 使用 Card, Button, Input 组件
2. 集成 divinationMarketService
3. 使用 useWallet, useAsync Hooks

### 第三批：解卦师相关（7个）
- diviner/register.tsx
- diviner/dashboard.tsx
- diviner/profile.tsx
- diviner/earnings.tsx
- diviner/reviews.tsx
- diviner/orders/index.tsx
- diviner/orders/[id].tsx
- diviner/packages/create.tsx
- diviner/packages/index.tsx

### 第四批：做市商相关（6个）
- maker/apply/deposit.tsx
- maker/apply/info.tsx
- maker/apply/pending.tsx
- maker/dashboard.tsx
- maker/deposit/index.tsx
- maker/deposit/replenish.tsx
- maker/deposit/withdraw/index.tsx
- maker/deposit/withdraw/status.tsx
- maker/penalties/index.tsx
- maker/penalties/[penaltyId]/index.tsx
- maker/penalties/[penaltyId]/appeal.tsx
- maker/settings.tsx

### 第五批：钱包相关（4个）
- ✅ wallet/transfer.tsx - 已完成
- wallet/transactions.tsx
- wallet/manage.tsx
- wallet/buy-dust/index.tsx
- wallet/buy-dust/order.tsx
- wallet/buy-dust/first-purchase.tsx
- wallet/buy-dust/[orderId]/index.tsx
- wallet/buy-dust/[orderId]/waiting.tsx
- wallet/buy-dust/[orderId]/complete.tsx

### 第六批：其他功能（剩余）
- profile/edit.tsx
- contacts/add.tsx
- chat相关
- matchmaking相关
- 等等

## 策略2：使用查找替换

### 常见替换模式

#### 1. 导入语句
```bash
# 查找
import { ActivityIndicator } from 'react-native';

# 替换为
import { LoadingSpinner } from '@/components/common';
```

#### 2. 加载状态
```bash
# 查找
const [isLoading, setIsLoading] = useState(false);

# 替换为
const { execute, isLoading } = useAsync();
```

#### 3. 钱包访问
```bash
# 查找
const { address } = useWalletStore();

# 替换为
const { address, balance, ensureUnlocked } = useWallet();
```

## 策略3：优先级排序

### 高优先级（立即优化）
1. 占卜页面 - 代码重复率最高
2. 市场页面 - 核心业务功能
3. 解卦师页面 - 核心业务功能

### 中优先级（本周完成）
4. 做市商页面
5. 钱包页面
6. Profile页面

### 低优先级（有余力时）
7. Chat页面
8. Contacts页面
9. Matchmaking页面

## 实施建议

### 方案A：逐个手动优化（推荐）
- 优点：质量高，可以深度优化
- 缺点：耗时较长
- 适用：核心页面

### 方案B：批量查找替换
- 优点：速度快
- 缺点：可能遗漏细节
- 适用：简单重复的优化

### 方案C：混合方案（最佳）
1. 核心页面手动优化（占卜、市场、解卦师）
2. 简单页面批量替换（列表、详情）
3. 复杂页面逐步重构

## 预期时间

- 每个简单页面：5-10分钟
- 每个复杂页面：15-30分钟
- 总计40+页面：约8-15小时

## 质量检查

每完成一批后检查：
- [ ] 导入语句正确
- [ ] 组件使用正确
- [ ] 服务集成正确
- [ ] Hooks使用正确
- [ ] 样式统一
- [ ] 功能正常

## 下一步

建议按以下顺序执行：
1. 完成第一批（占卜页面）- 2-3小时
2. 完成第二批（市场页面）- 2-3小时
3. 完成第三批（解卦师页面）- 2-3小时
4. 评估效果，调整策略
5. 继续优化剩余页面
