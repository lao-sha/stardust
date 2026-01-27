# Pallet Affiliate

## 模块概述

**pallet-affiliate** 是一个统一联盟计酬系统，整合了原有的5个联盟计酬相关pallet，提供完整的15层推荐关系管理和多模式分成计酬功能。

### 设计理念

- **统一**：整合5个独立模块为单一pallet，降低维护成本
- **灵活**：支持三种结算模式（即时/周结算/混合），适应不同业务场景
- **高效**：15层推荐链压缩算法，自动分配奖励
- **易用**：简化的API设计，降低集成难度

**版本**: v1.0.0
**整合日期**: 2025-10-28

## 整合自

本模块整合了以下5个独立pallet的功能：

- `pallet-affiliate`: 资金托管
- `pallet-affiliate-config`: 配置管理
- `pallet-affiliate-instant`: 即时分成
- `pallet-affiliate-weekly`: 周结算
- `pallet-memo-referrals`: 推荐关系

## 核心功能

### 1. 推荐关系管理（referral.rs）

推荐关系是联盟计酬的基础，建立推荐人与被推荐人之间的永久绑定关系。

#### 1.1 推荐人绑定

- **绑定方式**：新用户通过推荐码绑定推荐人
- **绑定特性**：推荐关系一经绑定永久生效，不可修改
- **循环检测**：自动检测并防止循环引用（A→B→C→A）
- **资格验证**：推荐人必须是有效会员
- **最大搜索深度**：`MaxSearchHops` 配置参数，防止无限循环

#### 1.2 推荐码管理

- **认领机制**：有效会员可认领自定义推荐码
- **长度限制**：推荐码长度范围 [MIN_CODE_LEN, MaxCodeLen]（默认4-16字节）
- **唯一性保证**：推荐码全局唯一，先到先得
- **双向映射**：账户↔推荐码双向映射，快速查询
- **默认推荐码**：系统可为会员自动生成默认推荐码（账户ID前8位十六进制）

#### 1.3 推荐链查询

- **向上追溯**：从指定账户向上追溯推荐人链条
- **最大层数**：最多追溯15层（MAX_REFERRAL_CHAIN）
- **快速查询**：使用 `Sponsors` StorageMap 实现 O(1) 查询
- **防护机制**：最大搜索深度保护，防止无限循环

### 2. 配置管理（types.rs）

配置管理提供灵活的结算模式和分成比例配置，支持动态调整。

#### 2.1 三种结算模式

| 模式 | 描述 | 特点 | 适用场景 |
|------|------|------|----------|
| **Weekly** | 全部周结算 | 批量结算，节省Gas | 大额交易，社区激励 |
| **Instant** | 全部即时分成 | 实时转账，立即到账 | 小额交易，快速激励 |
| **Hybrid** | 混合模式 | 前N层即时，后M层周结算 | 平衡效率与激励 |

**混合模式配置示例**：
- 前5层即时分成：快速激励直推团队
- 后10层周结算：稳定收益，批量结算

#### 2.2 分成比例配置

**即时分成比例（InstantLevelPercents）**：
- 15层独立配置，每层0-100%
- 默认配置（总计99%）：
  ```rust
  [30, 25, 15, 10, 7, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1]
  ```
  - L1: 30%（最高奖励）
  - L2: 25%
  - L3: 15%
  - L4: 10%
  - L5: 7%
  - L6: 3%
  - L7-L9: 各2%
  - L10-L15: 各1%

**周结算分成比例（WeeklyLevelPercents）**：
- 15层独立配置，每层0-100%
- 默认配置（总计82%）：
  ```rust
  [20, 10, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4]
  ```
  - L1: 20%
  - L2: 10%
  - L3-L15: 各4%

#### 2.3 每周区块数配置

- **用途**：用于周期计算和活跃期管理
- **默认值**：100800块（假设6秒出块，1周≈100800块）
- **可调整**：管理员可根据实际出块时间调整

### 3. 资金托管（escrow.rs）

资金托管提供独立的托管账户，实现联盟奖励资金与平台资金的隔离。

#### 3.1 独立托管账户

- **账户派生**：使用 `EscrowPalletId` 派生独立托管账户
- **资金隔离**：托管账户与平台资金完全隔离
- **安全性**：托管账户仅受本模块控制，无私钥泄露风险

#### 3.2 托管操作

| 操作 | 描述 | 调用时机 |
|------|------|----------|
| **存入** | 从付款人账户转入托管账户 | 用户购买/供奉时 |
| **提取** | 从托管账户转出指定金额 | 即时分成/周结算时 |
| **批量提取** | 批量转账到多个账户 | 周结算批量分配时 |
| **余额查询** | 查询托管账户当前余额 | 审计/监控时 |

#### 3.3 累计统计

- **TotalDeposited**：累计存入金额（审计用）
- **TotalWithdrawn**：累计提取金额（审计用）
- **实时余额**：通过 `Currency::free_balance` 查询

### 4. 即时分成（instant.rs）

即时分成在交易发生时立即分配奖励，实现实时转账、立即到账。

#### 4.1 实时转账机制

```
交易流程：
1. 获取推荐链（最多15层）
2. 获取即时分成比例配置
3. 逐层验证推荐人资格
4. 计算分成金额（基于配置比例）
5. 立即转账到推荐人账户
6. 发射事件（InstantRewardDistributed）
7. 更新累计统计
```

#### 4.2 推荐人资格验证

**验证规则**：
- 必须是有效会员（通过 `MembershipProvider` 验证）
- 层级不超过15层
- 账户状态正常（可接收转账）

**验证失败处理**：
- 无效推荐人：份额跳过该层，继续分配下一层
- 转账失败：跳过该层，继续分配下一层
- 容错设计：单个账户失败不影响其他账户

#### 4.3 分成金额计算

**计算公式**：
```rust
分成金额 = 可分配金额 × 层级比例 / 100
```

**示例**（假设可分配金额1000 DUST，L1=30%）：
```
L1奖励 = 1000 × 30 / 100 = 300 DUST
```

#### 4.4 累计统计

- **TotalInstantDistributed**：累计即时分配金额
- **按账户统计**：通过事件日志追溯
- **按层级统计**：通过事件日志追溯

### 5. 周结算（weekly.rs）

周结算按周期批量结算账户，记账分配、定期结算。

#### 5.1 记账分配机制

**消费上报流程**：
```
1. 计算当前周编号（block_number / blocks_per_week）
2. 获取推荐链（最多15层）
3. 获取周结算分成比例配置
4. 逐层累计应得金额到 Entitlement 存储
5. 更新活跃期（如有时长）
6. 更新直推活跃数
```

#### 5.2 活跃期管理

**活跃期概念**：
- 账户的活跃截止周（ActiveUntilWeek）
- 仅活跃账户可获得周结算奖励
- 供奉延长活跃期（按供奉时长）

**直推活跃数管理**：
- 统计每个账户的活跃直推数量（DirectActiveCount）
- 新激活直推时，增加推荐人的直推计数
- 用于未来动态层数调整（预留功能）

#### 5.3 周期结算

**结算流程**：
```
1. 获取当前结算游标（支持分批结算）
2. 迭代 Entitlement 存储，获取待结算账户
3. 批量转账（do_batch_withdraw）
4. 清理已结算账户的 Entitlement
5. 更新累计统计（TotalWeeklyDistributed）
6. 更新结算游标
7. 发射事件（CycleSettled）
```

**分批结算机制**：
- **max_accounts**：每次最多结算的账户数（防区块超载）
- **游标机制**：记录当前结算进度（SettleCursor）
- **继续结算**：未完成时更新游标，下次继续
- **结算完成**：全部完成后清空游标

#### 5.4 累计统计

- **TotalWeeklyDistributed**：累计周结算分配金额
- **按周期统计**：通过事件日志追溯
- **按账户统计**：通过 Entitlement 存储查询

### 6. 统一分配入口（distribute.rs）

统一分配入口提供自动路由功能，根据结算模式自动选择分配方式。

#### 6.1 系统费用扣除

**扣费规则**（通用分配场景）：
- **销毁**：5%（发送到 BurnAccount）
- **国库**：2%（发送到 TreasuryAccount）
- **存储**：3%（发送到 StorageAccount）
- **可分配**：90%（进入联盟奖励池）

**示例**（假设总金额1000 DUST）：
```
销毁：1000 × 5% = 50 DUST
国库：1000 × 2% = 20 DUST
存储：1000 × 3% = 30 DUST
可分配：1000 - 50 - 20 - 30 = 900 DUST
```

#### 6.2 会员专用分配

**特殊规则**：
- 会员费金额100%分配到推荐链，无系统扣费
- 使用即时分成模式（快速到账）
- 激励推荐行为，促进会员增长

**调用接口**：
```rust
do_distribute_membership_rewards(buyer, amount)
```

#### 6.3 模式自动路由

| 模式 | 路由策略 | 返回值 |
|------|----------|--------|
| **Weekly** | 调用 `do_report_consumption`（15层） | 0（周结算时立即返回0） |
| **Instant** | 调用 `do_instant_distribute`（15层） | 实际分配总额 |
| **Hybrid** | 先即时分成（前N层），再周结算（后M层） | 即时分配总额 |

#### 6.4 容错处理

- **余额不足**：自动跳过该层，继续下一层
- **链条中断**：单个账户失败不影响其他账户
- **转账失败**：记录失败原因（通过日志），继续分配
- **原子操作**：分配过程使用事务，失败自动回滚

## 15层压缩算法详解

### 算法目标

从推荐链中找到最多15层合格的推荐人，根据配置的分成比例分配奖励。

### 算法流程

```
输入：buyer（购买者）、distributable_amount（可分配金额）、levels（分配层数）

1. 获取推荐链：get_referral_chain(buyer) → [sponsor1, sponsor2, ..., sponsorN]
2. 获取分成比例：InstantLevelPercents 或 WeeklyLevelPercents
3. 遍历推荐链（最多15层）：
   for (index, referrer) in referral_chain.iter().take(levels) {
       a. 获取该层分成比例：percent = level_percents[index]
       b. 验证推荐人资格：is_valid_referrer(referrer, level)
       c. 计算分成金额：share = distributable_amount × percent / 100
       d. 转账/记账：
          - 即时模式：Currency::transfer
          - 周结算：Entitlement::mutate
       e. 发射事件
   }
4. 返回实际分配总额
```

### 压缩规则（精简版）

**当前实现（简化版）**：
- 仅检查会员有效性和活跃期
- 不做复杂的持仓门槛验证
- 不做动态层数调整

**未来扩展（可选）**：
- 持仓门槛：按持仓量决定可拿代数
- 直推要求：按直推数量决定可拿代数
- 动态层数：根据团队活跃度动态调整

### 资金流向

**即时分成模式**：
```
用户支付 → 托管账户 → 15层推荐人（立即转账）
```

**周结算模式**：
```
用户支付 → 托管账户 → Entitlement 记账 → 周末批量结算 → 推荐人
```

**混合模式**：
```
用户支付 → 托管账户 → {
    前N层：立即转账
    后M层：Entitlement 记账 → 周末结算
}
```

### 不足层数处理

**场景**：推荐链不足15层（例如只有5层）

**处理方式**：
- 仅分配实际存在的层数（5层）
- 未分配的份额（L6-L15）保留在托管账户
- 管理员可配置：并入国库/销毁/其他用途

## 三种结算模式对比

### 对比表格

| 对比维度 | Weekly | Instant | Hybrid |
|----------|--------|---------|--------|
| **到账时间** | 周末结算 | 立即到账 | 部分立即，部分周末 |
| **Gas成本** | 低（批量结算） | 高（逐笔转账） | 中等 |
| **用户体验** | 延迟高 | 实时反馈 | 平衡 |
| **系统负载** | 峰值高（周末） | 均匀分布 | 均匀分布 |
| **适用场景** | 大额交易 | 小额交易 | 通用 |
| **复杂度** | 高（游标/分批） | 低 | 中等 |

### 模式切换

**切换方式**：
```rust
// 切换到即时分成模式
set_settlement_mode(1, 0, 0)

// 切换到周结算模式
set_settlement_mode(0, 0, 0)

// 切换到混合模式（前5层即时，后10层周结算）
set_settlement_mode(2, 5, 10)
```

**注意事项**：
- 模式切换立即生效
- 已记账的周结算不受影响，仍然按原计划结算
- 建议在低峰期切换模式

### 最佳实践

**推荐配置**：
- **测试环境**：使用 Instant 模式，快速验证功能
- **生产环境**：使用 Hybrid 模式（前5层即时，后10层周结算）
- **特殊活动**：临时切换到 Instant 模式，提升用户体验

## 数据结构

### 核心类型定义

```rust
/// 结算模式枚举
pub enum SettlementMode {
    Weekly,
    Instant,
    Hybrid {
        instant_levels: u8,
        weekly_levels: u8,
    },
}

/// 分成比例配置（15层）
pub type LevelPercents = BoundedVec<u8, ConstU32<15>>;

/// 推荐链最大层数
pub const MAX_REFERRAL_CHAIN: u32 = 15;

/// 推荐码长度范围
pub const MIN_CODE_LEN: u32 = 4;
pub const MAX_CODE_LEN: u32 = 16;
```

### 存储项

#### 推荐关系存储（3个）

| 存储项 | 键类型 | 值类型 | 说明 |
|--------|--------|--------|------|
| `Sponsors` | AccountId | AccountId | 账户的推荐人 |
| `AccountByCode` | BoundedVec<u8> | AccountId | 推荐码→账户 |
| `CodeByAccount` | AccountId | BoundedVec<u8> | 账户→推荐码 |

#### 配置存储（4个）

| 存储项 | 值类型 | 说明 | 默认值 |
|--------|--------|------|--------|
| `SettlementMode` | SettlementMode | 当前结算模式 | Weekly |
| `InstantLevelPercents` | LevelPercents | 即时分成比例（15层） | [30,25,15,10,7,3,2,2,2,1,1,1,1,1,1] |
| `WeeklyLevelPercents` | LevelPercents | 周结算比例（15层） | [20,10,4,4,4,4,4,4,4,4,4,4,4,4,4] |
| `BlocksPerWeek` | BlockNumber | 每周区块数 | 100800 |

#### 托管存储（2个）

| 存储项 | 值类型 | 说明 |
|--------|--------|------|
| `TotalDeposited` | Balance | 累计存入金额（审计用） |
| `TotalWithdrawn` | Balance | 累计提取金额（审计用） |

#### 即时分成存储（1个）

| 存储项 | 值类型 | 说明 |
|--------|--------|------|
| `TotalInstantDistributed` | Balance | 累计即时分配金额 |

#### 周结算存储（6个）

| 存储项 | 键类型 | 值类型 | 说明 |
|--------|--------|--------|------|
| `Entitlement` | (u32, AccountId) | Balance | 每周应得金额 |
| `ActiveUntilWeek` | AccountId | u32 | 活跃截止周 |
| `DirectActiveCount` | AccountId | u32 | 直推活跃数 |
| `SettleCursor` | u32 | u32 | 结算游标（账户索引） |
| `CurrentSettlingCycle` | - | Option<u32> | 当前结算周期 |
| `TotalWeeklyDistributed` | - | Balance | 累计周结算金额 |

## 主要调用方法

### 用户接口

#### `bind_sponsor(sponsor_code)`

绑定推荐人

**参数**：
- `sponsor_code: Vec<u8>` - 推荐人的推荐码

**权限**：任何签名账户

**前置条件**：
- 自己未绑定过推荐人
- 推荐码存在且有效
- 推荐人是有效会员
- 不形成循环引用

**效果**：建立永久推荐关系

**示例**：
```rust
// Rust调用
pallet_affiliate::Pallet::<Runtime>::bind_sponsor(
    origin,
    b"ALICE123".to_vec()
)?;

// 前端调用
await api.tx.affiliate
  .bindSponsor("ALICE123")
  .signAndSend(account);
```

#### `claim_code(code)`

认领推荐码

**参数**：
- `code: Vec<u8>` - 自定义推荐码（4-16字节）

**权限**：任何签名账户

**前置条件**：
- 是有效会员
- 未认领过推荐码
- 推荐码未被占用

**效果**：获得推荐码，可用于推荐他人

**示例**：
```rust
// Rust调用
pallet_affiliate::Pallet::<Runtime>::claim_code(
    origin,
    b"BOB456".to_vec()
)?;

// 前端调用
await api.tx.affiliate
  .claimCode("BOB456")
  .signAndSend(account);
```

### 管理员接口（AdminOrigin）

#### `set_settlement_mode(mode_id, instant_levels, weekly_levels)`

设置结算模式

**参数**：
- `mode_id: u8` - 模式ID（0=Weekly, 1=Instant, 2=Hybrid）
- `instant_levels: u8` - 即时层数（Hybrid模式）
- `weekly_levels: u8` - 周结算层数（Hybrid模式）

**权限**：AdminOrigin

**校验**：
- Hybrid模式下：instant_levels + weekly_levels ≤ 15

**示例**：
```rust
// 设置为混合模式：前5层即时，后10层周结算
pallet_affiliate::Pallet::<Runtime>::set_settlement_mode(
    admin_origin,
    2, // Hybrid
    5, // instant_levels
    10 // weekly_levels
)?;

// 前端调用
await api.tx.affiliate
  .setSettlementMode(2, 5, 10)
  .signAndSend(adminAccount);
```

#### `set_instant_percents(percents)`

设置即时分成比例

**参数**：
- `percents: Vec<u8>` - 15层分成比例数组（每项0-100）

**权限**：AdminOrigin

**校验**：数组长度必须为15

**示例**：
```rust
// 设置为逐层递减
pallet_affiliate::Pallet::<Runtime>::set_instant_percents(
    admin_origin,
    vec![10, 8, 6, 5, 4, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1]
)?;

// 前端调用
const percents = [10, 8, 6, 5, 4, 3, 2, 2, 1, 1, 1, 1, 1, 1, 1];
await api.tx.affiliate
  .setInstantPercents(percents)
  .signAndSend(adminAccount);
```

#### `set_weekly_percents(percents)`

设置周结算分成比例

**参数**：
- `percents: Vec<u8>` - 15层分成比例数组（每项0-100）

**权限**：AdminOrigin

**校验**：数组长度必须为15

**示例**：
```rust
// 设置为L1-L2高，L3-L15平均
pallet_affiliate::Pallet::<Runtime>::set_weekly_percents(
    admin_origin,
    vec![8, 6, 5, 4, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1]
)?;

// 前端调用
const percents = [8, 6, 5, 4, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1];
await api.tx.affiliate
  .setWeeklyPercents(percents)
  .signAndSend(adminAccount);
```

#### `set_blocks_per_week(blocks)`

设置每周区块数

**参数**：
- `blocks: BlockNumber` - 每周区块数

**权限**：AdminOrigin

**默认值**：100800（6秒出块，1周≈100800块）

**示例**：
```rust
// 测试网（1分钟出块）
pallet_affiliate::Pallet::<Runtime>::set_blocks_per_week(
    admin_origin,
    10080u32.into()
)?;

// 前端调用
await api.tx.affiliate
  .setBlocksPerWeek(10080)
  .signAndSend(adminAccount);
```

### 周结算接口

#### `settle_cycle(cycle, max_accounts)`

结算指定周期

**参数**：
- `cycle: u32` - 周期编号（从0开始）
- `max_accounts: u32` - 最多结算账户数（防区块超载）

**权限**：任何签名账户（鼓励社区参与）

**逻辑**：
- 按游标继续上次未完成的结算
- 最多处理max_accounts个账户
- 未完成则更新游标，下次继续
- 全部完成则清空游标

**示例**：
```rust
// 结算第5周，每次最多100个账户
pallet_affiliate::Pallet::<Runtime>::settle_cycle(
    origin,
    5,
    100
)?;

// 前端调用
await api.tx.affiliate
  .settleCycle(5, 100)
  .signAndSend(account);
```

### 内部方法（供其他pallet调用）

#### `bind_sponsor_internal(who, sponsor)`

绑定推荐人（内部方法）

**参数**：
- `who: &AccountId` - 被推荐人
- `sponsor: &AccountId` - 推荐人

**特点**：
- 不验证，不发射事件
- 仅用于其他pallet内部绑定推荐关系
- 供系统级操作使用（如：初始化账户）

## 事件定义

### 推荐关系事件

| 事件 | 字段 | 描述 |
|------|------|------|
| `SponsorBound` | who, sponsor | 推荐人已绑定 |
| `CodeClaimed` | who, code | 推荐码已认领 |

### 配置管理事件

| 事件 | 字段 | 描述 |
|------|------|------|
| `SettlementModeSet` | - | 结算模式已更新 |
| `InstantPercentsSet` | - | 即时分成比例已更新 |
| `WeeklyPercentsSet` | - | 周结算比例已更新 |
| `BlocksPerWeekSet` | blocks | 每周区块数已更新 |

### 托管事件

| 事件 | 字段 | 描述 |
|------|------|------|
| `Deposited` | from, amount | 资金已存入托管 |
| `Withdrawn` | to, amount | 资金已从托管提取 |

### 即时分成事件

| 事件 | 字段 | 描述 |
|------|------|------|
| `InstantRewardDistributed` | referrer, buyer, level, amount | 即时奖励已分配 |

### 周结算事件

| 事件 | 字段 | 描述 |
|------|------|------|
| `CycleSettled` | cycle, settled_count, total_amount | 周期已结算 |

## 错误定义

### 推荐关系错误

| 错误 | 描述 |
|------|------|
| `AlreadyBound` | 已绑定推荐人 |
| `CodeNotFound` | 推荐码不存在 |
| `CannotBindSelf` | 不能绑定自己 |
| `WouldCreateCycle` | 会形成循环引用 |
| `NotMember` | 不是有效会员 |
| `CodeTooLong` | 推荐码过长 |
| `CodeTooShort` | 推荐码过短 |
| `CodeAlreadyTaken` | 推荐码已被占用 |
| `AlreadyHasCode` | 已拥有推荐码 |

### 配置管理错误

| 错误 | 描述 |
|------|------|
| `InvalidPercents` | 无效的分成比例 |
| `HybridLevelsTooMany` | 混合模式层数超限（>15） |
| `InvalidMode` | 无效的模式ID |

### 托管错误

| 错误 | 描述 |
|------|------|
| `WithdrawFailed` | 提款失败 |

## 配置参数

### Runtime配置示例

```rust
impl pallet_affiliate::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;

    // 托管 PalletId
    type EscrowPalletId = AffiliatePalletId;

    // 权限控制
    type WithdrawOrigin = EnsureRoot<AccountId>;
    type AdminOrigin = EnsureRoot<AccountId>;

    // 会员信息提供者
    type MembershipProvider = Membership;

    // 推荐码配置
    type MaxCodeLen = ConstU32<32>;
    type MaxSearchHops = ConstU32<15>;

    // 系统账户
    type BurnAccount = BurnAccount;
    type TreasuryAccount = TreasuryAccount;
    type StorageAccount = StorageAccount;
}
```

### PalletId配置

```rust
parameter_types! {
    pub const AffiliatePalletId: PalletId = PalletId(*b"py/afflt");
}
```

### 常量配置

```rust
parameter_types! {
    pub const MaxCodeLen: u32 = 32;       // 推荐码最大长度
    pub const MaxSearchHops: u32 = 15;    // 推荐链最大搜索深度
}
```

## 使用示例

### 场景1：新用户注册并绑定推荐人

```rust
// 步骤1：新用户获取推荐码（线下/链外）
let sponsor_code = b"ALICE123".to_vec();

// 步骤2：绑定推荐人
api.tx.affiliate
  .bindSponsor(sponsor_code)
  .signAndSend(newUser);

// 步骤3：成为会员后认领推荐码
let my_code = b"BOB456".to_vec();
api.tx.affiliate
  .claimCode(my_code)
  .signAndSend(newUser);

// 步骤4：可以开始推荐其他用户
// 分享推荐码 "BOB456" 给其他人
```

### 场景2：会员购买（自动分配奖励）

```rust
// 会员购买会员资格（100 DUST）
let amount = 100_000_000_000_000u128; // 100 DUST

// 系统自动调用分配接口（在会员购买逻辑中）
pallet_affiliate::Pallet::<Runtime>::do_distribute_membership_rewards(
    &buyer,
    amount.into()
)?;

// 根据结算模式自动分配：
// - Instant模式：立即转账给15层推荐人
// - Weekly模式：记账到本周应得金额
// - Hybrid模式：前N层即时，后M层周结算
```

### 场景3：供奉业务（90%进入联盟奖励池）

```rust
// 用户供奉（1000 DUST）
let gross_amount = 1000_000_000_000_000u128; // 1000 DUST

// 系统调用分配接口（在供奉逻辑中）
pallet_affiliate::Pallet::<Runtime>::do_distribute_rewards(
    &buyer,
    gross_amount.into(),
    Some(52) // 52周（1年）
)?;

// 扣费：
// - 销毁：5% = 50 DUST
// - 国库：2% = 20 DUST
// - 存储：3% = 30 DUST
// - 可分配：90% = 900 DUST

// 分配900 DUST给推荐链（按配置比例）
```

### 场景4：周结算

```rust
// 每周开始后，任何人可调用结算
let current_week = 5u32;
let max_accounts = 100u32;

// 第一次调用（结算100个账户）
api.tx.affiliate
  .settleCycle(current_week, max_accounts)
  .signAndSend(anyone);

// 如果未完成，继续调用（结算下100个账户）
api.tx.affiliate
  .settleCycle(current_week, max_accounts)
  .signAndSend(anyone);

// 直到全部账户结算完成
```

### 场景5：查询推荐链

```typescript
// 查询推荐人
const sponsor = await api.query.affiliate.sponsors(account.address);
console.log("My sponsor:", sponsor.toString());

// 查询推荐码
const code = await api.query.affiliate.codeByAccount(account.address);
console.log("My code:", new TextDecoder().decode(code.toU8a()));

// 查询完整推荐链（需要递归查询）
async function getReferralChain(account) {
  const chain = [];
  let current = account;

  for (let i = 0; i < 15; i++) {
    const sponsor = await api.query.affiliate.sponsors(current);
    if (sponsor.isEmpty) break;

    chain.push(sponsor.toString());
    current = sponsor.toString();
  }

  return chain;
}

const chain = await getReferralChain(account.address);
console.log("My referral chain:", chain);
```

### 场景6：管理员配置

```typescript
// 切换到混合模式：前5层即时，后10层周结算
await api.tx.affiliate
  .setSettlementMode(2, 5, 10)
  .signAndSend(adminAccount);

// 调整即时分成比例（更高激励）
const newInstantPercents = [35, 30, 20, 15, 10, 8, 6, 5, 4, 3, 2, 2, 1, 1, 1];
await api.tx.affiliate
  .setInstantPercents(newInstantPercents)
  .signAndSend(adminAccount);

// 调整每周区块数（测试网）
await api.tx.affiliate
  .setBlocksPerWeek(10080) // 1分钟出块
  .signAndSend(adminAccount);
```

## 集成说明

### 与 pallet-memo-offerings 集成

**集成方式**：通过 `OnOfferingCommitted` hook 自动触发联盟奖励分配

```rust
// 在 pallet-memorial 的 offer 函数中
impl<T: Config> Pallet<T> {
    pub fn do_offer(...) -> DispatchResult {
        // ... 供奉逻辑 ...

        // 调用联盟奖励分配
        let distributed = pallet_affiliate::Pallet::<T>::do_distribute_rewards(
            &who,
            amount,
            duration_weeks
        )?;

        // ... 后续逻辑 ...
    }
}
```

### 与 pallet-divination-membership 集成

**集成方式**：通过 `MembershipProvider` trait 提供会员信息

```rust
// pallet-divination-membership 实现 MembershipProvider
impl<T: Config> pallet_affiliate::MembershipProvider<T::AccountId> for Pallet<T> {
    fn is_valid_member(who: &T::AccountId) -> bool {
        // 检查会员状态
        Self::member_status(who).is_active()
    }
}
```

### 与 pallet-ledger 集成

**集成方式**：通过事件监听记录联盟奖励统计

```rust
// pallet-ledger 监听 InstantRewardDistributed 事件
impl<T: Config> Pallet<T> {
    fn on_instant_reward(referrer: &T::AccountId, amount: Balance) {
        // 记录推荐奖励统计
        Self::update_referral_stats(referrer, amount);
    }
}
```

### 与 pallet-escrow 集成

**集成方式**：本模块自包含托管功能，无需外部 escrow pallet

**如果需要外部 escrow**：
```rust
// 配置 WithdrawOrigin 为 pallet-escrow 的管理权限
type WithdrawOrigin = pallet_escrow::EnsureEscrowAdmin<AccountId>;
```

## Trading Pallet 集成

### AffiliateDistributor Trait 实现

本模块实现了 `AffiliateDistributor` trait，供 Trading Pallet 调用：

```rust
pub trait AffiliateDistributor<AccountId, Balance, BlockNumber> {
    fn distribute_rewards(
        buyer: &AccountId,
        amount: Balance,
        target: Option<(u8, u64)>,
    ) -> Result<Balance, DispatchError>;
}
```

### Trading Pallet 调用示例

```rust
// 在 pallet-trading 中
impl<T: Config> Pallet<T> {
    pub fn do_trade(...) -> DispatchResult {
        // ... 交易逻辑 ...

        // 调用联盟奖励分配
        let distributed = T::AffiliateDistributor::distribute_rewards(
            &buyer,
            commission_amount,
            Some((domain, id))
        )?;

        // ... 后续逻辑 ...
    }
}
```

## 最佳实践

### 1. 推荐关系建立

**推荐**：
- 在用户注册时提示绑定推荐人
- 提供推荐码输入框，支持复制粘贴
- 验证推荐码有效性后再提交交易
- 显示推荐人信息（昵称/头像），增强信任

**不推荐**：
- 允许用户随意更改推荐人（破坏推荐链稳定性）
- 强制绑定推荐人（影响用户体验）
- 使用过短的推荐码（容易碰撞）

### 2. 结算模式选择

**推荐配置**：
- **生产环境**：Hybrid模式（前5层即时，后10层周结算）
- **测试环境**：Instant模式（快速验证功能）
- **特殊活动**：临时切换到Instant模式（提升用户体验）

**切换时机**：
- 在低峰期切换模式（避免影响用户体验）
- 提前公告模式切换（给用户心理预期）
- 监控切换后的系统负载（及时调整）

### 3. 分成比例设计

**推荐策略**：
- L1-L3：高比例（30%+），激励直推团队
- L4-L10：中比例（5-10%），平衡收益
- L11-L15：低比例（1-3%），扩大覆盖面

**调整原则**：
- 总比例控制在100%以内（避免超发）
- 即时分成比例 > 周结算比例（快速激励）
- 定期分析数据，优化比例配置

### 4. 周结算优化

**推荐设置**：
- `max_accounts`: 100-200（根据区块大小调整）
- 结算时机：非高峰期（减少对用户交易的影响）
- 分批结算：多次调用，避免单次超载

**监控指标**：
- 结算耗时（单次结算的区块数）
- 待结算账户数（Entitlement 存储大小）
- 结算失败率（转账失败的账户数）

### 5. 安全考虑

**防护措施**：
- 推荐链循环检测（MaxSearchHops限制）
- 会员身份验证（仅有效会员可成为推荐人）
- 资金隔离（托管账户独立）
- 原子操作（分配过程原子化，失败自动回滚）
- 容错机制（单个账户失败不影响其他账户）

**审计要点**：
- 定期检查托管账户余额（TotalDeposited - TotalWithdrawn）
- 监控分配失败事件（转账失败/推荐人无效）
- 追溯推荐链完整性（检测异常推荐关系）

### 6. 性能优化

**优化策略**：
- 分批结算：防止单次结算过多账户导致区块超载
- 游标机制：记录结算进度，支持分批执行
- 惰性计算：仅在需要时计算活跃期和层数
- 去重机制：避免重复分配

**监控指标**：
- 推荐链查询耗时（get_referral_chain）
- 分配耗时（do_instant_distribute / do_report_consumption）
- 结算耗时（do_settle_cycle）
- 存储大小（Sponsors / Entitlement）

## 前端集成要点

### 1. 推荐码展示

```typescript
// 显示用户的推荐码
const MyReferralCode = () => {
  const [code, setCode] = useState("");

  useEffect(() => {
    api.query.affiliate.codeByAccount(account.address)
      .then(c => setCode(new TextDecoder().decode(c.toU8a())));
  }, [account]);

  return (
    <div>
      <p>我的推荐码：{code}</p>
      <button onClick={() => navigator.clipboard.writeText(code)}>
        复制推荐码
      </button>
    </div>
  );
};
```

### 2. 推荐链可视化

```typescript
// 显示推荐链（树状图）
const ReferralChainTree = () => {
  const [chain, setChain] = useState([]);

  useEffect(() => {
    getReferralChain(account.address).then(setChain);
  }, [account]);

  return (
    <div className="referral-tree">
      {chain.map((sponsor, index) => (
        <div key={index} className="level">
          <span>L{index + 1}:</span>
          <span>{sponsor}</span>
        </div>
      ))}
    </div>
  );
};
```

### 3. 联盟奖励统计

```typescript
// 显示联盟奖励统计
const AffiliateRewards = () => {
  const [stats, setStats] = useState({
    instantRewards: 0,
    weeklyRewards: 0,
    totalRewards: 0
  });

  // 监听事件，累计奖励
  useEffect(() => {
    api.query.system.events((events) => {
      events.forEach(({ event }) => {
        if (api.events.affiliate.InstantRewardDistributed.is(event)) {
          const { referrer, amount } = event.data;
          if (referrer.eq(account.address)) {
            setStats(prev => ({
              ...prev,
              instantRewards: prev.instantRewards + amount.toNumber()
            }));
          }
        }
      });
    });
  }, []);

  return (
    <div className="rewards-stats">
      <div>即时奖励：{stats.instantRewards} DUST</div>
      <div>周结算奖励：{stats.weeklyRewards} DUST</div>
      <div>累计奖励：{stats.totalRewards} DUST</div>
    </div>
  );
};
```

### 4. 周结算提醒

```typescript
// 周结算倒计时
const SettlementCountdown = () => {
  const [countdown, setCountdown] = useState(0);

  useEffect(() => {
    const updateCountdown = async () => {
      const now = await api.query.system.number();
      const blocksPerWeek = await api.query.affiliate.blocksPerWeek();
      const currentWeek = Math.floor(now.toNumber() / blocksPerWeek.toNumber());
      const nextWeekBlock = (currentWeek + 1) * blocksPerWeek.toNumber();
      const remaining = nextWeekBlock - now.toNumber();

      setCountdown(remaining);
    };

    updateCountdown();
    const interval = setInterval(updateCountdown, 6000); // 每6秒更新

    return () => clearInterval(interval);
  }, []);

  const remainingTime = countdown * 6; // 秒数
  const days = Math.floor(remainingTime / 86400);
  const hours = Math.floor((remainingTime % 86400) / 3600);

  return (
    <div className="settlement-countdown">
      <p>距离下次周结算：{days}天 {hours}小时</p>
    </div>
  );
};
```

## 未来扩展

### 1. 动态层数调整

**目标**：根据团队活跃度和持仓量动态调整可拿代数

**实现思路**：
```rust
// 根据直推活跃数调整层数
fn get_eligible_levels(referrer: &T::AccountId) -> u8 {
    let direct_active = DirectActiveCount::<T>::get(referrer);

    match direct_active {
        0..=2 => 3,    // 3层
        3..=5 => 5,    // 5层
        6..=10 => 10,  // 10层
        _ => 15,       // 15层
    }
}
```

### 2. 持仓门槛验证

**目标**：按持仓量决定可拿代数

**实现思路**：
```rust
// 根据持仓量调整层数
fn get_eligible_levels_by_balance(referrer: &T::AccountId) -> u8 {
    let balance = T::Currency::free_balance(referrer);

    if balance >= 10000.into() { 15 }      // 10000 DUST → 15层
    else if balance >= 5000.into() { 10 }  // 5000 DUST → 10层
    else if balance >= 1000.into() { 5 }   // 1000 DUST → 5层
    else { 3 }                             // 其他 → 3层
}
```

### 3. 奖励加速机制

**目标**：特殊活动期间提高分成比例

**实现思路**：
```rust
// 活动加速配置
pub struct BoostConfig {
    pub start_block: BlockNumber,
    pub end_block: BlockNumber,
    pub multiplier: u8, // 倍数（100 = 1倍，200 = 2倍）
}

// 应用加速
fn apply_boost(amount: Balance, config: &BoostConfig) -> Balance {
    if is_in_boost_period(config) {
        amount * config.multiplier / 100
    } else {
        amount
    }
}
```

### 4. NFT奖励

**目标**：将特殊联盟成就铸造为NFT

**实现思路**：
- 推荐100人：铸造"推广大使"NFT
- 团队累计消费10万DUST：铸造"金牌团队"NFT
- 连续52周活跃：铸造"年度之星"NFT

### 5. 联盟排行榜

**目标**：统计推荐奖励排行，激励竞争

**实现思路**：
```rust
// 排行榜存储
pub type LeaderBoard<T> = StorageMap<
    _,
    Blake2_128Concat,
    u32, // period
    BoundedVec<(AccountId, Balance), MaxLeaders>, // top N
>;

// 更新排行榜
fn update_leaderboard(period: u32, account: &AccountId, amount: Balance) {
    LeaderBoard::<T>::mutate(period, |board| {
        // 插入/更新排名
        // 保留前N名
    });
}
```

## 迁移说明

### 从旧版本迁移

如果你之前使用了以下任一模块，现在统一使用本模块：

| 旧模块 | 新模块功能 |
|--------|-----------|
| `pallet-affiliate` | `pallet-affiliate::资金托管功能` |
| `pallet-affiliate-config` | `pallet-affiliate::配置功能` |
| `pallet-affiliate-instant` | `pallet-affiliate::即时分成功能` |
| `pallet-affiliate-weekly` | `pallet-affiliate::周结算功能` |
| `pallet-memo-referrals` | `pallet-affiliate::推荐关系功能` |

### 迁移步骤

1. **移除旧依赖**：
```toml
# 删除 Cargo.toml 中的旧依赖
# pallet-affiliate = { ... }
# pallet-affiliate-config = { ... }
# pallet-affiliate-instant = { ... }
# pallet-affiliate-weekly = { ... }
# pallet-memo-referrals = { ... }

# 添加新依赖
pallet-affiliate = { path = "../pallets/affiliate", default-features = false }
```

2. **更新Runtime配置**：
```rust
// 删除旧配置
// impl pallet_affiliate_config::Config for Runtime { ... }
// impl pallet_affiliate_instant::Config for Runtime { ... }
// impl pallet_affiliate_weekly::Config for Runtime { ... }
// impl pallet_memo_referrals::Config for Runtime { ... }

// 添加新配置
impl pallet_affiliate::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type EscrowPalletId = AffiliatePalletId;
    type WithdrawOrigin = EnsureRoot<AccountId>;
    type AdminOrigin = EnsureRoot<AccountId>;
    type MembershipProvider = Membership;
    type MaxCodeLen = ConstU32<32>;
    type MaxSearchHops = ConstU32<15>;
    type BurnAccount = BurnAccount;
    type TreasuryAccount = TreasuryAccount;
    type StorageAccount = StorageAccount;
}
```

3. **更新construct_runtime宏**：
```rust
construct_runtime! {
    pub enum Runtime {
        // 删除旧模块
        // Affiliate: pallet_affiliate,
        // AffiliateConfig: pallet_affiliate_config,
        // AffiliateInstant: pallet_affiliate_instant,
        // AffiliateWeekly: pallet_affiliate_weekly,
        // Referrals: pallet_memo_referrals,

        // 添加新模块
        Affiliate: pallet_affiliate,
    }
}
```

4. **数据迁移**（如有必要）：
```rust
// 迁移推荐关系数据
pub fn migrate_referrals() -> Weight {
    // 从旧存储读取
    let old_sponsors = pallet_memo_referrals::Sponsors::<Runtime>::iter();

    // 写入新存储
    for (account, sponsor) in old_sponsors {
        pallet_affiliate::Sponsors::<Runtime>::insert(account, sponsor);
    }

    // 返回权重
    Weight::from_parts(1_000_000, 0)
}
```

## 故障排查

### 常见问题

**Q1: 推荐码无效**

A: 检查推荐码是否正确（区分大小写），推荐人是否已认领推荐码，推荐人是否为有效会员。

**Q2: 无法绑定推荐人**

A: 检查是否已绑定过推荐人（推荐关系一经绑定不可更改），是否尝试绑定自己，是否形成循环引用。

**Q3: 未收到联盟奖励**

A: 检查结算模式（Weekly模式需等待周结算），检查推荐人资格（是否为有效会员），检查活跃期（Weekly模式需保持活跃）。

**Q4: 周结算失败**

A: 检查托管账户余额（是否充足），检查结算参数（max_accounts是否合理），检查区块大小（是否超载）。

**Q5: 推荐链查询返回空**

A: 检查账户是否已绑定推荐人，检查推荐人账户是否存在，检查最大搜索深度配置。

### 调试方法

**启用详细日志**：
```bash
RUST_LOG=pallet_affiliate=debug ./target/release/solochain-template-node --dev
```

**查询存储状态**：
```typescript
// 查询推荐人
const sponsor = await api.query.affiliate.sponsors(account.address);

// 查询推荐码
const code = await api.query.affiliate.codeByAccount(account.address);

// 查询结算模式
const mode = await api.query.affiliate.settlementMode();

// 查询托管账户余额
const escrowAccount = api.query.affiliate.escrowAccount();
const balance = await api.query.balances.freeBalance(escrowAccount);
```

**监听事件**：
```typescript
api.query.system.events((events) => {
  events.forEach(({ event }) => {
    if (api.events.affiliate.SponsorBound.is(event)) {
      console.log("推荐人已绑定:", event.data);
    }
    if (api.events.affiliate.InstantRewardDistributed.is(event)) {
      console.log("即时奖励已分配:", event.data);
    }
    if (api.events.affiliate.CycleSettled.is(event)) {
      console.log("周期已结算:", event.data);
    }
  });
});
```

## 参考资料

### 相关文档

- [Substrate官方文档](https://docs.substrate.io/)
- [Polkadot-JS API文档](https://polkadot.js.org/docs/)
- [FRAME开发指南](https://docs.substrate.io/reference/frame-pallets/)

### 相关模块

- `pallet-divination-membership`: 会员系统
- `pallet-memorial`: 纪念服务
- `pallet-ledger`: 账本统计
- `pallet-escrow`: 托管服务（可选）

### 技术栈

- **Substrate**: v1.0.0
- **Polkadot SDK**: stable2409
- **FRAME**: v2
- **Rust**: 1.77+

---

**维护者**: Stardust Team
**最后更新**: 2025-11-11
**版本**: v1.0.0
