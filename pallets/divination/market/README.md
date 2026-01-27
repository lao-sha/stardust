# 通用玄学占卜服务市场 Pallet (pallet-divination-market)

去中心化的多类型占卜服务交易市场 - 支持梅花易数、八字命理、六爻、奇门遁甲、紫微斗数、塔罗牌等

## 概述

本模块实现了一个去中心化的占卜服务交易平台，连接占卜服务提供者与需求用户。通过 `DivinationProvider` trait 与各玄学核心模块解耦，支持多种占卜类型的服务交易。

## 架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        pallet-divination-market                             │
│                    (通用服务市场、订单管理、评价系统)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐                 │
│  │  服务提供者    │  │   服务套餐     │  │   订单系统     │                 │
│  │  Provider      │  │   Package      │  │   Order        │                 │
│  │  管理系统      │  │   管理系统     │  │   管理系统     │                 │
│  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘                 │
│          │                   │                   │                          │
│          └───────────────────┼───────────────────┘                          │
│                              ▼                                              │
│              ┌───────────────────────────────────┐                          │
│              │         核心业务逻辑              │                          │
│              │  • 下单支付      • 解读提交       │                          │
│              │  • 追问回复      • 评价信誉       │                          │
│              │  • 收益结算      • 提现管理       │                          │
│              └───────────────────────────────────┘                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                       DivinationProvider trait                              │
└─────────────────────────────┬───────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                  Runtime: CombinedDivinationProvider                        │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬───────────┤
│  Meihua  │   Bazi   │  Liuyao  │  Qimen   │  Ziwei   │  Tarot   │ Daliuren  │
│  梅花易数 │  八字    │  六爻    │  奇门    │  紫微    │  塔罗    │  大六壬   │
└──────────┴──────────┴──────────┴──────────┴──────────┴──────────┴───────────┘
```

## 核心功能

### 1. 服务提供者系统

#### 1.1 提供者注册与管理

```rust
/// 注册成为服务提供者
fn register_provider(
    origin,
    name: Vec<u8>,           // 显示名称
    bio: Vec<u8>,            // 个人简介
    specialties: u16,        // 擅长领域位图
    supported_types: u8,     // 支持的占卜类型位图
) -> DispatchResult

/// 更新提供者信息
fn update_provider(
    origin,
    name: Option<Vec<u8>>,
    bio: Option<Vec<u8>>,
    avatar_cid: Option<Vec<u8>>,
    specialties: Option<u16>,
    supported_divination_types: Option<u8>,
    accepts_urgent: Option<bool>,
) -> DispatchResult
```

#### 1.2 状态管理

| 状态 | 说明 | 触发条件 |
|------|------|----------|
| `Pending` | 待审核 | 新注册 |
| `Active` | 已激活 | 审核通过 / 恢复接单 |
| `Paused` | 已暂停 | 主动暂停接单 |
| `Banned` | 已封禁 | 违规被封 |
| `Deactivated` | 已注销 | 主动退出 |

```
注册 ──► Pending ──审核──► Active ◄──► Paused
                            │
                            ├──违规──► Banned
                            │
                            └──退出──► Deactivated
```

#### 1.3 等级体系

| 等级 | 名称 | 最低订单数 | 最低评分 | 平台费率 |
|------|------|-----------|---------|----------|
| 0 | Novice (新手) | 0 | - | 20% |
| 1 | Certified (认证) | 10 | 3.5★ | 15% |
| 2 | Senior (资深) | 50 | 4.0★ | 12% |
| 3 | Expert (专家) | 200 | 4.5★ | 10% |
| 4 | Master (大师) | 500 | 4.8★ | 8% |

**自动晋升机制**：每次收到评价后自动检查是否满足升级条件

### 2. 服务套餐系统

#### 2.1 套餐类型

| 类型 | 说明 | 基础时长 |
|------|------|----------|
| `TextReading` | 文字解卦 | 无限制 |
| `VoiceReading` | 语音解卦 | 10分钟 |
| `VideoReading` | 视频解卦 | 15分钟 |
| `LiveConsultation` | 实时咨询 | 30分钟 |

#### 2.2 套餐定义

```rust
pub struct ServicePackage<Balance, MaxDescLen> {
    pub id: u32,                      // 套餐 ID
    pub divination_type: DivinationType, // 占卜类型
    pub service_type: ServiceType,    // 服务类型
    pub name: BoundedVec<u8, 64>,     // 套餐名称
    pub description: BoundedVec<u8, MaxDescLen>, // 描述
    pub price: Balance,               // 价格
    pub duration: u32,                // 时长（分钟）
    pub follow_up_count: u8,          // 追问次数
    pub urgent_available: bool,       // 是否支持加急
    pub urgent_surcharge: u16,        // 加急加价（基点）
    pub is_active: bool,              // 是否启用
    pub sales_count: u32,             // 销量
}
```

### 3. 订单系统

#### 3.1 订单状态流转

```
                    ┌──取消──► Cancelled
                    │
PendingPayment ──支付──► Paid ──接单──► Accepted ──提交解读──► Completed ──评价──► Reviewed
                    │         │
                    │         └──拒绝──► Cancelled (退款)
                    │
                    └──超时──► Cancelled (退款)
```

#### 3.2 完整订单流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            订单生命周期                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  客户                        平台                         服务提供者         │
│   │                          │                              │              │
│   │──1. 选择套餐/占卜结果───►│                              │              │
│   │                          │                              │              │
│   │──2. 支付订单金额────────►│──托管到平台账户──►           │              │
│   │                          │                              │              │
│   │                          │──3. 通知新订单──────────────►│              │
│   │                          │                              │              │
│   │                          │◄──4. 接受/拒绝订单──────────│              │
│   │                          │                              │              │
│   │◄──5. 拒绝则退款─────────│                              │              │
│   │                          │                              │              │
│   │                          │◄──6. 提交解读(IPFS CID)────│              │
│   │                          │                              │              │
│   │                          │──7. 结算费用                 │              │
│   │                          │   • 提供者收益 → 余额        │              │
│   │                          │   • 平台手续费 → 平台账户    │              │
│   │                          │                              │              │
│   │──8. 追问（可选）────────►│──────────────────────────►│              │
│   │                          │                              │              │
│   │                          │◄──9. 回复追问────────────────│              │
│   │                          │                              │              │
│   │──10. 提交评价───────────►│──更新提供者评分──►           │              │
│   │                          │                              │              │
│   │                          │◄──11. 回复评价（可选）──────│              │
│   │                          │                              │              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 3.3 费用计算

```
原价 = 套餐价格
加急费 = 原价 × 加急比例 / 10000  (如果加急)
总金额 = 原价 + 加急费

平台费率 = 根据提供者等级（8%-20%）
平台手续费 = 总金额 × 平台费率 / 10000
提供者收益 = 总金额 - 平台手续费
```

### 4. 评价系统

#### 4.1 多维度评分

| 维度 | 说明 | 范围 |
|------|------|------|
| `overall_rating` | 总体评分 | 1-5★ |
| `accuracy_rating` | 准确度 | 1-5★ |
| `attitude_rating` | 服务态度 | 1-5★ |
| `response_rating` | 响应速度 | 1-5★ |

#### 4.2 评价功能

- **评价期限**：订单完成后 `ReviewPeriod` 区块内可评价
- **匿名评价**：支持匿名评价保护隐私
- **提供者回复**：提供者可回复评价
- **影响等级**：评分影响提供者等级晋升

### 5. 收益管理

#### 5.1 收益结算

```rust
// 订单完成时自动结算
提供者收益 = 订单金额 - 平台手续费
ProviderBalances[provider] += 提供者收益
```

#### 5.2 提现功能

```rust
/// 申请提现
fn request_withdrawal(
    origin,
    amount: Balance,
) -> DispatchResult
```

- 即时到账（从平台账户转出）
- 记录提现历史

## 存储结构

### 核心存储

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `Providers` | `Map<AccountId, Provider>` | 服务提供者信息 |
| `Packages` | `DoubleMap<AccountId, u32, Package>` | 服务套餐 |
| `Orders` | `Map<u64, Order>` | 订单详情 |
| `FollowUps` | `Map<u64, Vec<FollowUp>>` | 追问记录 |
| `Reviews` | `Map<u64, Review>` | 评价详情 |

### 索引存储

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `CustomerOrders` | `Map<AccountId, Vec<u64>>` | 客户订单索引 |
| `ProviderOrders` | `Map<AccountId, Vec<u64>>` | 提供者订单索引 |
| `ProviderBalances` | `Map<AccountId, Balance>` | 提供者余额 |
| `Withdrawals` | `Map<u64, WithdrawalRequest>` | 提现记录 |

### 统计存储

| 存储项 | 类型 | 说明 |
|--------|------|------|
| `MarketStatistics` | `MarketStats` | 全局市场统计 |
| `TypeStatistics` | `Map<DivinationType, TypeStats>` | 按类型统计 |

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `MinDeposit` | `Balance` | 100 DUST | 最小保证金 |
| `MinServicePrice` | `Balance` | 1 DUST | 最小服务价格 |
| `OrderTimeout` | `BlockNumber` | 28800 (48h) | 订单超时 |
| `AcceptTimeout` | `BlockNumber` | 1200 (2h) | 接单超时 |
| `ReviewPeriod` | `BlockNumber` | 100800 (7d) | 评价期限 |
| `WithdrawalCooldown` | `BlockNumber` | 14400 (24h) | 提现冷却 |
| `MaxNameLength` | `u32` | 64 | 名称最大长度 |
| `MaxBioLength` | `u32` | 256 | 简介最大长度 |
| `MaxDescriptionLength` | `u32` | 1024 | 描述最大长度 |
| `MaxCidLength` | `u32` | 64 | IPFS CID 最大长度 |
| `MaxPackagesPerProvider` | `u32` | 10 | 每个提供者最大套餐数 |
| `MaxFollowUpsPerOrder` | `u32` | 5 | 每订单最大追问数 |

## 可调用函数

### 提供者管理

| 函数 | 索引 | 权重 | 说明 |
|------|------|------|------|
| `register_provider` | 0 | 50M | 注册服务提供者 |
| `update_provider` | 1 | 30M | 更新提供者信息 |
| `pause_provider` | 2 | 20M | 暂停接单 |
| `resume_provider` | 3 | 20M | 恢复接单 |
| `deactivate_provider` | 4 | 30M | 注销提供者 |

### 套餐管理

| 函数 | 索引 | 权重 | 说明 |
|------|------|------|------|
| `create_package` | 5 | 40M | 创建服务套餐 |
| `update_package` | 6 | 30M | 更新套餐信息 |
| `remove_package` | 7 | 20M | 删除套餐 |

### 订单管理

| 函数 | 索引 | 权重 | 说明 |
|------|------|------|------|
| `create_order` | 8 | 50M | 创建订单（含支付） |
| `accept_order` | 9 | 30M | 接受订单 |
| `reject_order` | 10 | 40M | 拒绝订单 |
| `submit_answer` | 11 | 40M | 提交解读 |
| `cancel_order` | 17 | 40M | 取消订单 |

### 追问与评价

| 函数 | 索引 | 权重 | 说明 |
|------|------|------|------|
| `submit_follow_up` | 12 | 30M | 提交追问 |
| `answer_follow_up` | 13 | 30M | 回复追问 |
| `submit_review` | 14 | 40M | 提交评价 |
| `reply_review` | 15 | 25M | 回复评价 |

### 财务管理

| 函数 | 索引 | 权重 | 说明 |
|------|------|------|------|
| `request_withdrawal` | 16 | 40M | 申请提现 |

## 事件

| 事件 | 说明 |
|------|------|
| `ProviderRegistered` | 提供者注册成功 |
| `ProviderUpdated` | 提供者信息更新 |
| `ProviderPaused` | 提供者暂停接单 |
| `ProviderResumed` | 提供者恢复接单 |
| `ProviderDeactivated` | 提供者注销 |
| `ProviderTierUpgraded` | 提供者等级提升 |
| `PackageCreated` | 套餐创建 |
| `PackageUpdated` | 套餐更新 |
| `PackageRemoved` | 套餐删除 |
| `OrderCreated` | 订单创建 |
| `OrderPaid` | 订单支付成功 |
| `OrderAccepted` | 订单被接受 |
| `OrderRejected` | 订单被拒绝 |
| `AnswerSubmitted` | 解读提交 |
| `OrderCompleted` | 订单完成 |
| `OrderCancelled` | 订单取消 |
| `OrderRefunded` | 订单退款 |
| `FollowUpSubmitted` | 追问提交 |
| `FollowUpAnswered` | 追问回复 |
| `ReviewSubmitted` | 评价提交 |
| `ReviewReplied` | 评价回复 |
| `WithdrawalRequested` | 提现申请 |
| `WithdrawalCompleted` | 提现完成 |

## 错误类型

| 错误 | 说明 |
|------|------|
| `ProviderAlreadyExists` | 提供者已存在 |
| `ProviderNotFound` | 提供者不存在 |
| `ProviderNotActive` | 提供者未激活 |
| `ProviderBanned` | 提供者已封禁 |
| `InsufficientDeposit` | 保证金不足 |
| `PackageNotFound` | 套餐不存在 |
| `TooManyPackages` | 套餐数量超限 |
| `PriceTooLow` | 价格过低 |
| `OrderNotFound` | 订单不存在 |
| `InvalidOrderStatus` | 订单状态无效 |
| `NotOrderOwner` | 非订单所有者 |
| `NotProvider` | 非服务提供者 |
| `InsufficientBalance` | 余额不足 |
| `NoFollowUpsRemaining` | 无追问次数 |
| `FollowUpNotFound` | 追问不存在 |
| `AlreadyReviewed` | 已评价 |
| `InvalidRating` | 评分无效 |
| `ReviewPeriodExpired` | 评价期已过 |
| `InvalidWithdrawalAmount` | 提现金额无效 |
| `CannotOrderSelf` | 不能给自己下单 |
| `DivinationResultNotFound` | 占卜结果不存在 |
| `DivinationTypeNotSupported` | 不支持的占卜类型 |

## 擅长领域位图

| 位 | 领域 | 说明 |
|----|------|------|
| 0 | Career | 事业运势 |
| 1 | Relationship | 感情婚姻 |
| 2 | Wealth | 财运投资 |
| 3 | Health | 健康养生 |
| 4 | Education | 学业考试 |
| 5 | Travel | 出行旅游 |
| 6 | Legal | 官司诉讼 |
| 7 | Finding | 寻人寻物 |
| 8 | FengShui | 风水堪舆 |
| 9 | DateSelection | 择日选时 |

## 使用示例

### 注册服务提供者

```rust
// 支持梅花易数(0)和八字(1)，擅长事业(0)和财运(2)
DivinationMarket::register_provider(
    origin,
    b"张三大师".to_vec(),
    b"从业20年，专注事业财运分析".to_vec(),
    0b0000_0101,  // 擅长: 事业 + 财运
    0b0000_0011,  // 支持: 梅花 + 八字
)?;
```

### 创建服务套餐

```rust
DivinationMarket::create_package(
    origin,
    DivinationType::Meihua,      // 梅花易数
    ServiceType::TextReading,    // 文字解卦
    b"详细文字解卦".to_vec(),
    b"根据卦象进行详细分析...".to_vec(),
    10 * UNIT,                   // 10 DUST
    0,                           // 无时长限制
    3,                           // 3次追问
    true,                        // 支持加急
    5000,                        // 加急+50%
)?;
```

### 下单流程

```rust
// 1. 创建订单（自动支付）
DivinationMarket::create_order(
    origin,
    provider_account,
    DivinationType::Meihua,
    hexagram_id,              // 卦象 ID
    package_id,
    b"QmXxx...".to_vec(),     // 问题描述 IPFS CID
    false,                    // 不加急
)?;

// 2. 提供者接单
DivinationMarket::accept_order(provider_origin, order_id)?;

// 3. 提供者提交解读
DivinationMarket::submit_answer(
    provider_origin,
    order_id,
    b"QmYyy...".to_vec(),     // 解读内容 IPFS CID
)?;

// 4. 客户提交追问（可选）
DivinationMarket::submit_follow_up(
    origin,
    order_id,
    b"QmZzz...".to_vec(),     // 追问内容 CID
)?;

// 5. 提供者回复追问
DivinationMarket::answer_follow_up(
    provider_origin,
    order_id,
    0,                        // 追问索引
    b"QmAaa...".to_vec(),     // 回复内容 CID
)?;

// 6. 客户提交评价
DivinationMarket::submit_review(
    origin,
    order_id,
    5,    // 总体 5 星
    5,    // 准确度 5 星
    5,    // 态度 5 星
    4,    // 响应 4 星
    Some(b"QmBbb...".to_vec()), // 评价内容 CID
    false,                      // 非匿名
)?;
```

### 前端集成示例

```typescript
import { ApiPromise } from '@polkadot/api';

// 查询提供者信息
const provider = await api.query.divinationMarket.providers(accountId);
console.log('提供者:', provider.toHuman());

// 查询套餐列表
const packages = await api.query.divinationMarket.packages.entries(providerAccount);
console.log('套餐列表:', packages.map(([k, v]) => v.toHuman()));

// 查询订单详情
const order = await api.query.divinationMarket.orders(orderId);
console.log('订单:', order.toHuman());

// 查询市场统计
const stats = await api.query.divinationMarket.marketStatistics();
console.log('市场统计:', stats.toHuman());

// 查询按类型统计
const meihuaStats = await api.query.divinationMarket.typeStatistics('Meihua');
console.log('梅花易数统计:', meihuaStats.toHuman());
```

## 测试

```bash
# 运行测试
SKIP_WASM_BUILD=1 cargo test -p pallet-divination-market

# 测试覆盖
# - 提供者注册/更新/暂停/恢复/注销
# - 套餐创建/更新/删除
# - 订单创建/接受/拒绝/取消
# - 解读提交/追问/回复
# - 评价提交/回复
# - 提现申请
# - 等级自动晋升
# - 费用计算验证
```

## 安全考虑

1. **资金安全**
   - 订单金额托管到平台账户
   - 保证金使用 `reserve` 机制锁定
   - 提现需验证余额充足

2. **权限控制**
   - 只有订单客户可以取消订单
   - 只有提供者可以接单/提交解读
   - 只有订单完成后才能评价

3. **状态验证**
   - 所有状态转换都有严格验证
   - 防止重复评价
   - 评价期限检查

4. **占卜结果验证**
   - 通过 `DivinationProvider` trait 验证结果存在
   - 确保订单关联真实的占卜结果

## 版本历史

- **v0.1.0** (2025-11-29)：初始版本
  - 完整的提供者管理系统
  - 服务套餐系统
  - 订单生命周期管理
  - 多维度评价系统
  - 收益结算与提现
  - 多占卜类型支持
  - 完整测试覆盖

## 相关模块

- **pallet-divination-common**：通用类型与 trait 定义
- **pallet-divination-privacy**：隐私保护与服务提供者注册
- **pallet-divination-nft**：占卜结果 NFT 铸造
- **pallet-divination-ai**：AI 解读服务
- **pallet-meihua**：梅花易数排盘
- **pallet-bazi**：八字排盘
- **pallet-liuyao**：六爻排盘
- **pallet-qimen**：奇门遁甲排盘
- **pallet-ziwei**：紫微斗数排盘
- **pallet-tarot**：塔罗牌排盘

## 未来规划：服务提供者档案增强

> 原 `pallet-divination-profile` 规划内容已合并至此

### 规划功能

1. **档案状态管理**
   - `Pending`（待审核）→ `Active`（活跃）→ `Suspended`（暂停）→ `Banned`（封禁）

2. **资质认证增强**
   - 证书类型：学历证书、专业资格、行业协会认证、师承证明、获奖证书
   - 管理员审核认证流程
   - 认证徽章展示

3. **保证金机制**
   - 注册需缴纳保证金（已实现 `MinDeposit`）
   - 违规扣除保证金
   - 退出返还保证金

4. **档案搜索与推荐**
   - 按专业领域搜索
   - 按评分排序
   - 推荐算法

### 当前实现位置

| 功能 | 模块 | 状态 |
|------|------|------|
| 服务提供者注册 | `privacy` | ✅ 已实现 |
| 服务提供者类型 | `privacy` | ✅ 已实现 |
| 信誉分系统 | `privacy` | ✅ 已实现 |
| 详细资料管理 | `market` | ✅ 已实现 |
| 资质证书上传 | `market` | ✅ 已实现 |
| 等级体系 | `market` | ✅ 已实现 |
| 评价系统 | `market` | ✅ 已实现 |

## 许可证

Unlicense
