# 会员系统模块 (pallet-divination-membership)

Stardust 占卜平台的会员订阅管理模块，提供 6 级会员体系、DUST 代币奖励、每日签到和用户档案管理功能。

## 概述

本模块实现了完整的会员生态系统：

- **订阅管理**：6 级会员体系（Free → Diamond）
- **DUST 奖励**：活动奖励与防滥用机制
- **签到系统**：每日签到与连续签到奖励
- **用户档案**：部分加密的个人信息存储

## 会员等级

| 等级 | 月费 (USDT) | 年费 (USDT) | 存储折扣 | 免费 AI 次数/月 |
|------|------------|------------|---------|----------------|
| Free | 0 | - | 0% | 3 |
| Bronze | 5 | 50 | 10% | 10 |
| Silver | 25 | 250 | 20% | 30 |
| Gold | 80 | 800 | 30% | 100 |
| Platinum | 200 | 2000 | 40% | 300 |
| Diamond | 500 | 5000 | 50% | 无限 |

> 年费享受约 16.7% 折扣（10 个月价格）
> 
> **注意**：会员费用以 USDT 计价，支付时调用 pricing 模块实时换算为等值 DUST

## 核心功能

### 1. 订阅管理

```rust
// 订阅会员
subscribe(tier: MemberTier, duration: SubscriptionDuration, auto_renew: bool)

// 升级会员（按比例补差价）
upgrade_tier(new_tier: MemberTier)

// 取消自动续费
cancel_subscription()
```

### 2. 每日签到

```rust
// 每日签到领取 DUST 奖励
check_in()
```

签到奖励规则：
- 基础奖励：根据奖励池动态调整
- 连续 7 天及以上：1.5 倍奖励
- 每周签到满 7 天额外奖励

### 3. 用户档案

```rust
// 更新用户档案
update_profile(
    display_name: Vec<u8>,
    gender: Option<Gender>,
    birth_date: Option<BirthDate>,
    birth_hour: Option<u8>,
    longitude: Option<i32>,
    latitude: Option<i32>,
    encrypted_sensitive: Option<EncryptedData>,
)

// 清除敏感数据
clear_sensitive_data()
```

档案数据分层：
- **公开层**：昵称、性别
- **占卜层**：出生日期、时辰、经纬度（明文，用于自动填充）
- **加密层**：姓名、详细地址等敏感信息

### 4. 服务提供者

```rust
// 申请成为服务提供者
apply_provider()

// 管理员验证提供者（需 Root 权限）
verify_provider(provider: AccountId, verified: bool)
```

## 防滥用机制

### 新账户冷却期

- 新账户 7 天内无法领取奖励
- 冷却期约 50,400 个区块（12秒/块）

### 最低余额要求

- 账户余额 ≥ 1 DUST 才能领取奖励
- 防止空投账户刷奖励

### 每日奖励上限

- 每类奖励有每日上限
- 超出上限后当日无法继续领取

### 动态奖励调整

- 根据奖励池余额动态调整奖励系数
- 奖励池充足时奖励更高，不足时自动降低

## 奖励类型

```rust
pub enum RewardTxType {
    CheckIn,           // 每日签到
    AiInterpretation,  // AI 解读奖励
    Review,            // 评价奖励
    Referral,          // 推荐奖励
    Activity,          // 活动奖励
    Bonus,             // 额外奖励
}
```

## 存储结构

### MemberInfo - 会员信息

```rust
pub struct MemberInfo<BlockNumber, Balance> {
    pub tier: MemberTier,
    pub expires_at: BlockNumber,
    pub subscribed_at: BlockNumber,
    pub total_paid: Balance,
    pub auto_renew: bool,
}
```

### MemberProfile - 用户档案

```rust
pub struct MemberProfile<BlockNumber, MaxName, MaxEncrypted> {
    pub display_name: BoundedVec<u8, MaxName>,
    pub gender: Option<Gender>,
    pub birth_date: Option<BirthDate>,
    pub birth_hour: Option<u8>,
    pub longitude: Option<i32>,
    pub latitude: Option<i32>,
    pub encrypted_sensitive: Option<EncryptedSensitiveData>,
    pub is_provider: bool,
    pub provider_verified: bool,
    pub updated_at: BlockNumber,
}
```

### CheckInRecord - 签到记录

```rust
pub struct CheckInRecord {
    pub last_check_in_day: u32,  // 最后签到日
    pub streak: u32,             // 连续签到天数
    pub total_days: u32,         // 总签到天数
    pub this_week: u8,           // 本周签到位图
}
```

## 费用分配

会员费用分配：
- **90%** → 国库账户
- **10%** → 奖励池账户

## 配置参数

```rust
#[pallet::config]
pub trait Config: frame_system::Config {
    /// 奖励池分配比例（万分比，1000 = 10%）
    type RewardPoolAllocation: Get<u32>;
    
    /// 新账户冷却期（区块数）
    type NewAccountCooldown: Get<BlockNumber>;
    
    /// 领取奖励最低余额
    type MinBalanceForRewards: Get<Balance>;
    
    /// 每天区块数（约 7200）
    type BlocksPerDay: Get<BlockNumber>;
    
    /// 每月区块数（约 216000）
    type BlocksPerMonth: Get<BlockNumber>;
}
```

## 事件

```rust
Subscribed { who, tier, duration, amount_paid, expires_at }
TierUpgraded { who, old_tier, new_tier, amount_paid }
SubscriptionCancelled { who, expires_at }
CheckedIn { who, streak, reward }
RewardGranted { who, amount, tx_type }
ProfileUpdated { who }
ProviderApplied { who }
ProviderVerified { provider, verified }
```

## 依赖

```toml
[dependencies]
pallet-divination-membership = { path = "../membership", default-features = false }
```

## License

Apache-2.0
