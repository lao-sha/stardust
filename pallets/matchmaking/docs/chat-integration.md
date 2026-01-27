# 婚恋模块集成 Chat Pallet 开发文档

## 一、概述

### 目标
1. **匹配后自动授予聊天权限**
2. **每日发起聊天配额**：限制免费用户每日可主动发起的新聊天数
3. **超级喜欢特权**：收到超级喜欢可直接发起聊天，不受配额限制

### 核心规则

| 场景 | 是否消耗配额 |
|------|-------------|
| 首次主动发起聊天 | ✅ 消耗 |
| 已有会话继续聊天 | ❌ 不消耗 |
| 被动回复（对方先发起） | ❌ 不消耗 |
| 超级喜欢特权发起 | ❌ 不消耗 |

---

## 二、数据结构

```rust
/// 聊天发起配额
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, Default)]
pub struct ChatInitiationQuota {
    pub chats_initiated: u32,
    pub last_reset_day: u32,
}

/// 聊天发起方式
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq)]
pub enum ChatInitiationType {
    InitiatedByMe,        // 我主动发起（消耗配额）
    InitiatedByOther,     // 对方先发起（不消耗）
    SuperLikePrivilege,   // 超级喜欢特权（不消耗）
}

/// 聊天会话信息
#[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ChatSessionInfo {
    pub created_at: u64,
    pub initiation_type: ChatInitiationType,
}
```

---

## 三、新增存储

```rust
/// 聊天会话记录
#[pallet::storage]
pub type ChatSessions<T: Config> = StorageDoubleMap<
    _, Blake2_128Concat, T::AccountId,
    Blake2_128Concat, [u8; 32],  // target_hash
    ChatSessionInfo, OptionQuery,
>;

/// 聊天发起配额
#[pallet::storage]
pub type ChatInitiationQuotas<T: Config> = StorageMap<
    _, Blake2_128Concat, T::AccountId,
    ChatInitiationQuota, ValueQuery,
>;
```

---

## 四、新增配置

```rust
type FreeDailyChatInitiations: Get<u32>;         // 免费用户: 3
type MonthlyMemberDailyChatInitiations: Get<u32>; // 月费会员: 20
type YearlyMemberDailyChatInitiations: Get<u32>;  // 年费会员: 0 (无限)
type ChatPermission: SceneAuthorizationManager<...>;
```

---

## 五、核心逻辑

### 5.1 检查聊天权限

```rust
pub fn can_initiate_chat(sender, receiver) -> Result<ChatInitiationType> {
    // 1. 检查是否被屏蔽
    // 2. 检查是否已有聊天会话 → 直接返回
    // 3. 检查超级喜欢特权 → SuperLikePrivilege
    // 4. 检查是否已匹配
    // 5. 检查对方是否先发起 → InitiatedByOther
    // 6. 首次主动发起 → InitiatedByMe
}
```

### 5.2 发送婚恋消息

```rust
#[pallet::call_index(9)]
pub fn send_matchmaking_message(
    origin, receiver, content_cid, msg_type_code
) -> DispatchResult {
    let sender = ensure_signed(origin)?;
    
    // 1. 检查聊天权限
    let initiation_type = Self::can_initiate_chat(&sender, &receiver)?;
    
    // 2. 根据发起类型处理
    match initiation_type {
        InitiatedByMe => {
            Self::check_and_consume_chat_initiation_quota(&sender)?;
            Self::record_chat_session(&sender, &receiver, initiation_type)?;
            Self::grant_chat_permission(&sender, &receiver)?;
        },
        InitiatedByOther | SuperLikePrivilege => {
            Self::record_chat_session(&sender, &receiver, initiation_type)?;
            if matches!(initiation_type, SuperLikePrivilege) {
                Self::grant_chat_permission(&sender, &receiver)?;
            }
        },
    }
    
    // 3. 调用 chat-core 发送消息
    Ok(())
}
```

---

## 六、超级喜欢特权

```
A 超级喜欢 B → B 收到超级喜欢 → B 可免配额发起聊天给 A
```

检查逻辑：
```rust
fn has_super_like_privilege(sender, receiver) -> bool {
    let receiver_hash = compute_target_hash(receiver);
    let super_likes = SuperLikesReceived::<T>::get(sender);
    super_likes.iter().any(|sl| sl.sender_hash == receiver_hash)
}
```

---

## 七、会员配额矩阵

| 会员等级 | 每日可发起聊天数 |
|----------|-----------------|
| 免费用户 | 3 次 |
| 月费会员 | 20 次 |
| 年费会员 | 无限 |

---

## 八、Chat-Permission 集成

```rust
// 授予权限（首次发起聊天时）
T::ChatPermission::grant_bidirectional_scene_authorization(
    *b"matchmak",
    user_a, user_b,
    SceneType::Custom(b"matchmaking"),
    SceneId::None,
    None,  // 永不过期
    Vec::new(),
)?;

// 撤销权限（屏蔽用户时）
T::ChatPermission::revoke_all_by_source(*b"matchmak", &who, &target);
```

---

## 九、实现计划

| 优先级 | 任务 | 工作量 |
|--------|------|--------|
| P0 | 添加存储和类型定义 | 0.5 天 |
| P0 | 实现 can_initiate_chat | 0.5 天 |
| P0 | 实现配额检查和消耗 | 0.5 天 |
| P0 | 实现 send_matchmaking_message | 1 天 |
| P1 | 集成 chat-permission | 0.5 天 |
| P1 | 在 block_user 中撤销权限 | 0.5 天 |

---

## 十、错误类型

```rust
DailyChatInitiationQuotaExceeded,    // 每日配额已用完
NotMatchedOrNoSuperLikePrivilege,    // 未匹配且无特权
ChatSessionNotFound,                  // 会话不存在
```

---

## 十一、事件

```rust
ChatSessionEstablished { user, target_hash, initiation_type },
ChatInitiationQuotaConsumed { user, remaining, limit },
```
