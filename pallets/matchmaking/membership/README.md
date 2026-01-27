# 婚恋会员模块 (Matchmaking Membership)

婚恋平台的年费会员管理模块，用于区分免费用户和付费会员。

## 功能概述

- **会员订阅**：支持月付、季付、半年付、年付和终身会员
- **会员续费**：支持手动续费和自动续费
- **会员升级**：从年费会员升级到终身会员
- **权益管理**：不同等级享有不同权益
- **使用量追踪**：追踪每日功能使用情况

## 会员等级

| 等级 | 说明 |
|------|------|
| Free | 免费用户，基础功能 |
| Annual | 年费会员，完整功能 |
| Lifetime | 终身会员，永久权益 |

## 订阅时长与折扣

| 时长 | 折扣 |
|------|------|
| 1个月 | 无折扣 |
| 3个月 | 5% |
| 6个月 | 10% |
| 12个月 | 20% |
| 终身 | 单独定价 |

## 会员权益对比

| 权益 | Free | Annual | Lifetime |
|------|------|--------|----------|
| 每日推荐数 | 10 | 50 | 100 |
| 超级喜欢 | ❌ | 5/天 | 10/天 |
| 合婚分析 | 1/天 | 10/天 | 30/天 |
| 查看谁喜欢我 | ❌ | ✅ | ✅ |
| 查看访客 | ❌ | ✅ | ✅ |
| 隐身浏览 | ❌ | ✅ | ✅ |
| 优先展示 | ❌ | ✅ | ✅ |
| 消息已读回执 | ❌ | ✅ | ✅ |
| 高级筛选 | ❌ | ✅ | ✅ |
| 专属客服 | ❌ | ❌ | ✅ |

## 费用分配

会员费用分配（15层推荐链）：

| 分配项 | 比例 |
|--------|------|
| 销毁 | 5% |
| 国库 | 2% |
| 存储 | 3% |
| 推荐链分配 | 90% |

## 使用示例

### 订阅会员

```rust
// 订阅年费会员（12个月，20%折扣）
MatchmakingMembership::subscribe(
    origin,
    SubscriptionDuration::OneYear,
    true,  // 自动续费
    Some(referrer),  // 推荐人
)?;
```

### 续费

```rust
// 续费3个月
MatchmakingMembership::renew(
    origin,
    SubscriptionDuration::ThreeMonths,
)?;
```

### 升级到终身会员

```rust
// 升级到终身会员（支付差价）
MatchmakingMembership::upgrade(origin)?;
```

### 使用权益

```rust
// 使用超级喜欢
MatchmakingMembership::use_benefit(
    origin,
    BenefitType::SuperLike,
)?;
```

## Trait 接口

### MembershipProvider

供其他模块查询会员状态：

```rust
// 获取会员等级
let tier = T::MembershipProvider::get_tier(&who);

// 检查是否是有效会员
let is_member = T::MembershipProvider::is_active_member(&who);

// 检查是否有特定权益
let can_see = T::MembershipProvider::has_benefit(&who, MembershipBenefit::SeeWhoLikesMe);
```

### MembershipUsageTracker

供其他模块记录功能使用：

```rust
// 检查是否可以使用超级喜欢
if T::MembershipUsageTracker::can_use_super_like(&who) {
    T::MembershipUsageTracker::record_super_like_usage(&who)?;
}
```

## 依赖关系

```
pallet-matchmaking-membership
    │
    ├── pallet-matchmaking-common
    ├── pallet-matchmaking-profile
    ├── pallet-trading-common (PricingProvider)
    └── pallet-affiliate (AffiliateDistributor)
```

## 存储项

| 存储 | 说明 |
|------|------|
| Memberships | 会员信息 |
| DailyUsages | 每日使用记录 |
| GlobalStats | 全局统计 |
| SubscriptionHistory | 订阅历史 |

## 事件

| 事件 | 说明 |
|------|------|
| Subscribed | 会员已订阅 |
| Renewed | 会员已续费 |
| Upgraded | 会员已升级 |
| AutoRenewCancelled | 自动续费已取消 |
| Expired | 会员已过期 |
| BenefitUsed | 权益已使用 |
