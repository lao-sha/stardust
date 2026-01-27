# 婚恋模块 (Matchmaking Module)

基于八字命理的去中心化婚恋平台模块。

## 模块结构

```
pallets/matchmaking/
├── common/          # 共享类型和工具
│   ├── types.rs     # 用户资料、匹配结果等类型
│   ├── traits.rs    # 匹配算法 trait
│   └── lib.rs
├── profile/         # 用户资料管理
│   ├── lib.rs       # 资料创建、更新、查询
│   └── tests.rs
├── matching/        # 匹配算法
│   ├── lib.rs       # 匹配算法实现
│   ├── bazi.rs      # 八字合婚算法
│   ├── personality.rs # 性格匹配算法
│   └── tests.rs
├── recommendation/  # 推荐系统
│   ├── lib.rs       # 推荐算法
│   └── tests.rs
├── interaction/     # 互动功能
│   ├── lib.rs       # 点赞、关注、超级喜欢
│   └── tests.rs
└── membership/      # 会员管理
    ├── lib.rs       # 会员订阅、续费、升级
    ├── types.rs     # 会员等级、权益类型
    ├── traits.rs    # MembershipProvider trait
    └── tests.rs
```

## 功能概述

### 1. Common - 共享类型

- **类型定义**：Gender、ProfilePrivacyMode、EducationLevel、InteractionType 等
- **Trait 接口**：BaziDataProvider、ProfileProvider、MatchingAlgorithm 等

### 2. Profile - 用户资料管理

- **资料创建**：创建婚恋资料（昵称、性别、出生日期等）
- **资料更新**：更新个人信息、头像、简介
- **择偶条件**：设置年龄范围、位置偏好、教育水平等
- **八字绑定**：绑定八字命盘用于合婚分析
- **隐私设置**：公开/仅匹配可见/完全私密

### 3. Matching - 匹配算法

- **八字合婚**：
  - 日柱天干合（甲己合、乙庚合等）
  - 地支六合/六冲分析
  - 五行互补分析
- **性格匹配**：
  - 互补性格加分
  - 冲突性格减分
  - 共同优点加分
- **合婚请求**：创建、授权、生成报告

### 4. Recommendation - 推荐系统

- **推荐策略**：
  - 基于匹配评分
  - 基于活跃度
  - 基于地理位置
- **推荐更新**：定期刷新推荐列表

### 5. Interaction - 互动功能

- **点赞**：表达好感
- **超级喜欢**：付费功能
- **跳过**：跳过当前用户
- **屏蔽**：屏蔽不想看到的用户
- **匹配检测**：互相喜欢自动匹配

### 6. Membership - 会员管理

- **会员等级**：Free（免费）、Annual（年费）、Lifetime（终身）
- **订阅时长**：1/3/6/12个月，终身
- **会员权益**：推荐数、超级喜欢、合婚分析、隐身浏览等
- **使用量追踪**：每日功能使用限额

## 算法权重

| 维度 | 权重 | 说明 |
|------|------|------|
| 日柱合婚 | 30% | 天干合、地支六合/六冲 |
| 五行互补 | 25% | 用神、喜神、忌神配合 |
| 性格匹配 | 20% | 互补性格、冲突性格 |
| 神煞分析 | 15% | 桃花、红鸾等（预留） |
| 大运配合 | 10% | 婚姻运势同步（预留） |

## 匹配建议

| 评分 | 建议 |
|------|------|
| 90-100 | 天作之合 |
| 75-89 | 良缘佳配 |
| 60-74 | 中等缘分 |
| 40-59 | 需要磨合 |
| 0-39 | 不建议 |

## 使用流程

### 创建资料

```rust
// 创建婚恋资料
Profile::create_profile(
    origin,
    nickname,
    Gender::Female,
    Some(birth_date),
    Some(location),
    Some(bio),
)?;

// 设置择偶条件
Profile::update_preferences(
    origin,
    (25, 35),  // 年龄范围
    Some(location),
    Some(EducationLevel::Bachelor),
    None,
)?;

// 绑定八字
Profile::link_bazi(origin, bazi_id)?;
```

### 合婚分析

```rust
// 创建合婚请求
Matching::create_request(
    origin,
    party_b,
    party_a_bazi_id,
    party_b_bazi_id,
)?;

// 乙方授权
Matching::authorize_request(origin, request_id)?;

// 生成报告
Matching::generate_report(origin, request_id)?;
```

### 互动功能

```rust
// 点赞
Interaction::like(origin, target)?;

// 超级喜欢
Interaction::super_like(origin, target)?;

// 屏蔽
Interaction::block_user(origin, target)?;
```

## 依赖关系

```
pallet-matchmaking-common
    │
    ├── pallet-divination-common
    └── pallet-bazi-chart

pallet-matchmaking-profile
    │
    ├── pallet-matchmaking-common
    ├── pallet-divination-privacy
    └── pallet-stardust-ipfs

pallet-matchmaking-matching
    │
    ├── pallet-matchmaking-common
    ├── pallet-matchmaking-profile
    └── pallet-bazi-chart

pallet-matchmaking-recommendation
    │
    ├── pallet-matchmaking-common
    ├── pallet-matchmaking-profile
    └── pallet-matchmaking-matching

pallet-matchmaking-interaction
    │
    ├── pallet-matchmaking-common
    └── pallet-matchmaking-profile
```

## 隐私保护

- **双方授权**：合婚分析需要双方授权
- **隐私模式**：支持公开/仅匹配可见/完全私密
- **TEE 加密**：敏感数据使用 TEE 加密存储

## 免责声明

本模块提供的合婚分析仅供参考，不作为婚姻决策的唯一依据。
