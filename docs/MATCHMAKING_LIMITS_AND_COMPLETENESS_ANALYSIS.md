# 婚恋模块限制与完整度深度分析

**分析日期**: 2026-01-25  
**分析范围**: 用户查看限制、聊天发起限制、前端组件完整度

---

## 一、用户查看限制分析

### 1.1 链端实现现状

#### ✅ 已定义但未使用
在 `pallets/matchmaking/interaction/src/lib.rs` 中：

```rust
/// 每日配额信息
pub struct DailyQuota {
    pub likes_used: u32,           // ✅ 已实现
    pub super_likes_used: u32,      // ✅ 已实现
    pub views_used: u32,            // ⚠️ 已定义但未使用
    pub last_reset_day: u32,
}
```

**问题**：
- `views_used` 字段已定义，但**没有任何函数检查或更新它**
- 没有 `check_and_consume_view_quota()` 函数
- 没有配置常量定义每日查看限制（如 `FreeDailyViews`）

#### ❌ 缺失的功能

1. **查看配额检查函数**
   ```rust
   // 需要添加
   pub fn check_and_consume_view_quota(user: &T::AccountId) -> DispatchResult
   ```

2. **配置常量**
   ```rust
   // 需要添加
   type FreeDailyViews: Get<u32>;
   type MemberDailyViews: Get<u32>;
   ```

3. **查看资料时的配额消耗**
   - 在推荐列表查看用户资料时应该消耗配额
   - 在查看详细资料时应该消耗配额

### 1.2 是否需要查看限制？

#### ✅ **强烈建议实现**

**理由**：

1. **防止滥用**
   - 防止用户无限制浏览所有用户资料
   - 防止爬虫批量抓取用户信息
   - 保护用户隐私

2. **商业模式**
   - 免费用户：每日限制查看数量（如 50 次）
   - 会员用户：更多查看次数（如 200 次）
   - 超级会员：无限制查看

3. **用户体验**
   - 限制可以引导用户更精准地选择
   - 避免信息过载
   - 提高匹配质量

4. **系统资源**
   - 减少不必要的链上查询
   - 降低推荐算法计算压力

### 1.3 实现建议

#### 链端实现

```rust
// 1. 添加配置常量
#[pallet::config]
pub trait Config {
    /// 免费用户每日查看配额
    #[pallet::constant]
    type FreeDailyViews: Get<u32>;
    
    /// 会员每日查看配额
    #[pallet::constant]
    type MemberDailyViews: Get<u32>;
}

// 2. 添加查看配额检查函数
pub fn check_and_consume_view_quota(user: &T::AccountId) -> DispatchResult {
    let max_views = Self::get_daily_view_limit(user);
    
    if max_views == 0 {
        return Ok(()); // 无限制
    }
    
    DailyQuotas::<T>::try_mutate(user, |quota| {
        Self::maybe_reset_quota(quota);
        
        if quota.views_used >= max_views {
            return Err(Error::<T>::DailyViewQuotaExceeded.into());
        }
        
        quota.views_used = quota.views_used.saturating_add(1);
        Ok(())
    })
}

// 3. 在查看资料时调用
pub fn view_profile(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 检查查看配额
    Self::check_and_consume_view_quota(&who)?;
    
    // 记录查看行为（可选）
    // ...
    
    Ok(())
}
```

#### 前端实现

```typescript
// 在 matchmaking.service.ts 中添加
async viewProfile(
  target: string,
  onStatusChange?: StatusCallback
): Promise<void> {
  const api = this.getApi();
  const address = await getCurrentSignerAddress();
  
  if (!api.tx.matchmakingInteraction?.viewProfile) {
    throw new Error("Matchmaking pallet not available");
  }
  
  const tx = api.tx.matchmakingInteraction.viewProfile(target);
  await signAndSend(api, tx, address, onStatusChange);
}

// 在 discover.tsx 中调用
useEffect(() => {
  if (currentProfile) {
    matchmakingService.viewProfile(currentProfile.owner);
  }
}, [currentProfile]);
```

---

## 二、聊天发起限制分析

### 2.1 链端实现现状

#### ✅ **已完整实现**

在 `pallets/matchmaking/interaction/src/lib.rs` 中：

1. **存储结构**
   ```rust
   pub struct ChatInitiationQuota {
       pub chats_initiated: u32,
       pub last_reset_day: u32,
   }
   ```

2. **配置常量**
   ```rust
   type FreeDailyChatInitiations: Get<u32>;
   type MonthlyMemberDailyChatInitiations: Get<u32>;
   type YearlyMemberDailyChatInitiations: Get<u32>;
   ```

3. **核心函数**
   ```rust
   pub fn initiate_matchmaking_chat(
       origin: OriginFor<T>,
       receiver: T::AccountId,
   ) -> DispatchResult
   ```

4. **配额检查**
   ```rust
   pub fn check_and_consume_chat_initiation_quota(
       sender: &T::AccountId,
   ) -> DispatchResult
   ```

#### ✅ **权限规则完善**

- ✅ 已匹配用户可发起聊天（消耗配额）
- ✅ 收到超级喜欢后可发起聊天（不消耗配额）
- ✅ 已有会话可继续聊天（不消耗配额）
- ✅ 被动回复不消耗配额

### 2.2 前端实现现状

#### ❌ **未实现**

在 `frontend/src/services/matchmaking.service.ts` 中：

**缺失的函数**：
```typescript
// ❌ 缺少
async initiateChat(
  receiver: string,
  onStatusChange?: StatusCallback
): Promise<void>
```

**问题**：
- 前端没有调用 `initiate_matchmaking_chat` 函数
- 在发送消息前没有检查聊天权限
- 可能导致用户直接发送消息，绕过配额检查

### 2.3 实现建议

#### 前端实现

```typescript
// 在 matchmaking.service.ts 中添加
async initiateChat(
  receiver: string,
  onStatusChange?: StatusCallback
): Promise<void> {
  const api = this.getApi();
  const address = await getCurrentSignerAddress();
  
  if (!api.tx.matchmakingInteraction?.initiateMatchmakingChat) {
    throw new Error("Matchmaking pallet not available");
  }
  
  const tx = api.tx.matchmakingInteraction.initiateMatchmakingChat(receiver);
  await signAndSend(api, tx, address, onStatusChange);
}

// 在聊天页面中，发送第一条消息前调用
async function sendFirstMessage(receiver: string, message: string) {
  // 1. 先发起聊天（检查配额）
  await matchmakingService.initiateChat(receiver);
  
  // 2. 然后发送消息
  await chatService.sendMessage(receiver, message);
}
```

---

## 三、前端组件完整度分析

### 3.1 页面清单

| 页面路径 | 文件 | 状态 | 完整度 |
|---------|------|------|--------|
| `/matchmaking` | `index.tsx` | ✅ 已实现 | 80% |
| `/matchmaking/discover` | `discover.tsx` | ⚠️ 部分实现 | 40% |
| `/matchmaking/matches` | `matches.tsx` | ⚠️ 部分实现 | 30% |
| `/matchmaking/requests` | `requests.tsx` | ❓ 未查看 | - |
| `/matchmaking/create-profile` | `create-profile.tsx` | ❓ 未查看 | - |

### 3.2 功能完整度分析

#### ✅ 已实现的功能

1. **基础页面结构**
   - ✅ 首页（index.tsx）
   - ✅ 发现页面（discover.tsx）
   - ✅ 匹配列表（matches.tsx）

2. **基础交互**
   - ✅ 点赞（like）
   - ✅ 超级喜欢（superLike）
   - ✅ 跳过（pass）
   - ✅ 屏蔽（blockUser）

3. **资料管理**
   - ✅ 创建资料
   - ✅ 更新资料
   - ✅ 查询资料

#### ❌ 缺失的功能

1. **推荐算法**
   ```typescript
   // discover.tsx 中
   const loadProfiles = useCallback(async () => {
     // ❌ 这里应该调用推荐算法
     // 暂时使用模拟数据
     setProfiles([]);
   }, [address]);
   ```
   
   **需要实现**：
   - 调用链端推荐算法
   - 根据择偶条件筛选
   - 隐私模式检查

2. **查看配额显示**
   ```typescript
   // ❌ 缺少查看配额显示
   // 应该显示：今日剩余查看次数
   ```

3. **聊天发起配额显示**
   ```typescript
   // ❌ 缺少聊天配额显示
   // 应该显示：今日剩余聊天发起次数
   ```

4. **匹配列表数据加载**
   ```typescript
   // matches.tsx 中
   const loadMatches = useCallback(async () => {
     const matchIds = await matchmakingService.getUserMatches(address);
     // ❌ 这里需要根据 matchId 获取对方的 profile
     // 暂时使用空数组
     setMatches([]);
   }, [address]);
   ```

5. **合婚请求管理**
   - ❌ 创建合婚请求页面
   - ❌ 授权/拒绝合婚请求
   - ❌ 查看合婚报告

6. **隐私设置页面**
   - ❌ 隐私模式切换
   - ❌ 字段级隐私设置

7. **择偶条件设置**
   - ❌ 择偶条件编辑页面

### 3.3 链端接口完整度

#### ✅ 已实现的接口

**Profile 模块**：
- ✅ `createProfile`
- ✅ `updateProfile`
- ✅ `updatePreferences`
- ✅ `linkBazi`
- ✅ `updatePrivacyMode`
- ✅ `deleteProfile`
- ✅ `payMonthlyFee`
- ✅ `uploadPhoto`
- ✅ `uploadPhotosBatch`

**Interaction 模块**：
- ✅ `initializeSalt`
- ✅ `like`
- ✅ `superLike`
- ✅ `pass`
- ✅ `blockUser`
- ✅ `unblockUser`
- ✅ `initiateMatchmakingChat` ⚠️ 前端未调用

**Matching 模块**：
- ✅ `createRequest`
- ✅ `authorizeRequest`
- ✅ `rejectRequest`
- ✅ `cancelRequest`
- ✅ `generateReport`

#### ❌ 缺失的接口

1. **查看资料接口**
   ```rust
   // ❌ 需要添加
   pub fn view_profile(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult
   ```

2. **推荐算法接口**
   ```rust
   // ❌ 需要添加（在 recommendation pallet 中）
   pub fn get_recommendations(
       origin: OriginFor<T>,
       limit: u32,
   ) -> Vec<T::AccountId>
   ```

3. **查询配额接口**
   ```rust
   // ❌ 需要添加
   pub fn get_remaining_quota(user: &T::AccountId) -> (u32, u32, u32) // (likes, super_likes, views)
   pub fn get_remaining_chat_quota(user: &T::AccountId) -> (u32, u32)
   ```

---

## 四、关键问题总结

### 4.1 用户查看限制

| 项目 | 状态 | 优先级 |
|------|------|--------|
| 链端配额检查 | ❌ 未实现 | 🔴 高 |
| 链端查看接口 | ❌ 未实现 | 🔴 高 |
| 前端调用 | ❌ 未实现 | 🟡 中 |
| 配额显示 | ❌ 未实现 | 🟡 中 |

**建议**：**必须实现**，防止滥用和保护隐私。

### 4.2 聊天发起限制

| 项目 | 状态 | 优先级 |
|------|------|--------|
| 链端配额检查 | ✅ 已实现 | - |
| 链端发起接口 | ✅ 已实现 | - |
| 前端调用 | ❌ 未实现 | 🔴 高 |
| 配额显示 | ❌ 未实现 | 🟡 中 |

**建议**：**必须实现前端调用**，确保配额检查生效。

### 4.3 前端完整度

| 模块 | 完整度 | 缺失功能 |
|------|--------|----------|
| 基础页面 | 60% | 推荐算法、数据加载 |
| 交互功能 | 70% | 查看配额、聊天配额 |
| 资料管理 | 80% | 字段级隐私设置 |
| 合婚功能 | 30% | 大部分功能缺失 |

---

## 五、实施建议

### 5.1 优先级排序

#### 🔴 高优先级（必须实现）

1. **聊天发起限制前端调用**
   - 在发送第一条消息前调用 `initiateMatchmakingChat`
   - 显示配额不足的错误提示

2. **用户查看限制链端实现**
   - 实现 `view_profile` 函数
   - 实现 `check_and_consume_view_quota` 函数
   - 添加配置常量

3. **推荐算法接口**
   - 实现推荐算法查询接口
   - 前端调用获取推荐列表

#### 🟡 中优先级（建议实现）

4. **配额显示**
   - 在发现页面显示剩余查看次数
   - 在聊天页面显示剩余聊天发起次数

5. **匹配列表数据加载**
   - 根据匹配列表获取用户资料
   - 显示匹配时间等信息

#### 🟢 低优先级（可选）

6. **合婚功能完善**
   - 合婚请求管理页面
   - 合婚报告展示

7. **隐私设置完善**
   - 字段级隐私设置界面

### 5.2 实施步骤

#### 第一阶段：核心限制功能

1. **链端实现查看限制**
   ```rust
   // 1. 添加配置常量
   // 2. 实现 check_and_consume_view_quota
   // 3. 实现 view_profile 函数
   // 4. 添加错误类型 DailyViewQuotaExceeded
   ```

2. **前端实现查看限制调用**
   ```typescript
   // 1. 在 service 中添加 viewProfile
   // 2. 在 discover.tsx 中调用
   // 3. 显示配额不足提示
   ```

3. **前端实现聊天发起限制调用**
   ```typescript
   // 1. 在 service 中添加 initiateChat
   // 2. 在聊天页面发送第一条消息前调用
   // 3. 显示配额不足提示
   ```

#### 第二阶段：用户体验优化

4. **配额显示**
   ```typescript
   // 1. 添加查询配额接口
   // 2. 在页面显示剩余配额
   // 3. 配额不足时显示升级提示
   ```

5. **推荐算法集成**
   ```typescript
   // 1. 实现推荐算法查询
   // 2. 前端调用获取推荐列表
   // 3. 根据隐私模式过滤
   ```

#### 第三阶段：功能完善

6. **匹配列表完善**
7. **合婚功能完善**
8. **隐私设置完善**

---

## 六、结论

### 6.1 用户查看限制

**结论**：**必须实现**

- 链端已定义 `views_used` 但未使用
- 需要实现完整的查看配额检查机制
- 对防止滥用和保护隐私至关重要

### 6.2 聊天发起限制

**结论**：**链端已实现，前端需补充**

- 链端实现完整，包括配额检查和权限规则
- 前端未调用 `initiateMatchmakingChat` 函数
- 需要在前端发送消息前调用，确保配额检查生效

### 6.3 前端完整度

**结论**：**约 60% 完整**

- 基础页面结构已实现
- 核心交互功能已实现
- 但缺少关键功能：
  - 推荐算法集成
  - 配额显示
  - 数据加载完善
  - 合婚功能

### 6.4 总体建议

1. **立即实施**：聊天发起限制前端调用
2. **尽快实施**：用户查看限制完整实现
3. **逐步完善**：推荐算法、配额显示等功能

---

**文档版本**: v1.0  
**最后更新**: 2026-01-25

