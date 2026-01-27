# Pallet Stardust IPFS

## 模块概述

IPFS存储服务管理模块，提供去中心化内容固定（Pin）服务，是Stardust平台的核心基础设施模块。该模块实现了完整的IPFS存储服务系统，包括内容固定、运营者管理、三层分层策略、自动扣费、OCW健康巡检等核心功能，为整个平台的内容存储提供稳定可靠的去中心化基础设施。

本模块支持多种业务场景：
- **纪念平台**：逝者档案、墓位信息、供奉品等内容存储
- **占卜服务**：AI解读结果、NFT媒体、服务市场内容等存储
- **通用存储**：通过自定义域（Custom SubjectType）支持任意业务扩展

### 设计理念

- **多副本冗余**：通过多运营者节点确保内容的高可用性
- **分层存储**：根据内容重要性提供不同级别的存储服务
- **自动化管理**：OCW自动完成健康检查、故障迁移和周期扣费
- **经济激励**：通过保证金和SLA统计建立运营者激励机制
- **资金安全**：三层扣费机制确保服务的可持续性

## 核心功能

### 1. IPFS内容固定（Pin）
- **多层级Pin请求**：支持Critical/Standard/Temporary三个层级
- **自动运营者分配**：基于分层、优先级和容量自动选择运营者
- **多副本冗余机制**：根据Pin层级设定不同的副本数
- **状态追踪管理**：完整的Pin状态生命周期管理
- **智能重试机制**：失败时自动重新分配运营者

### 2. 运营者管理
- **分层运营者系统**：Core/Community/External三层运营者分类
- **动态注册注销**：支持运营者动态加入和退出
- **保证金锁定机制**：通过保证金确保运营者服务质量
- **SLA统计与奖惩**：实时统计运营者服务水平
- **状态管理**：Active/Suspended/Banned三种状态管理

### 3. 三层分层策略
- **Critical层**：5副本，6小时巡检，1.5x费率，适用于关键内容
- **Standard层**：3副本，24小时巡检，1.0x费率，适用于一般内容
- **Temporary层**：1副本，7天巡检，0.5x费率，适用于临时内容

### 4. 自动扣费机制
- **三层扣费策略**：IpfsPoolAccount → SubjectFunding → 宽限期
- **周期性扣费**：每7天自动扣除存储费用
- **宽限期保护**：资金不足时进入宽限期，保护现有服务
- **配额管理**：每个subject每月100 DUST免费配额

### 5. OCW健康巡检机制
- **自动状态检查**：定期检查所有Pin的健康状态
- **故障自动修复**：检测到故障时自动迁移到新运营者
- **降级处理**：当副本数不足时自动补充副本
- **SLA统计更新**：实时更新运营者服务质量统计

## 数据结构

### PinTier - Pin层级枚举
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum PinTier {
    Critical = 0,   // 关键层：5副本，6小时巡检，1.5x费率
    Standard = 1,   // 标准层：3副本，24小时巡检，1.0x费率
    Temporary = 2,  // 临时层：1副本，7天巡检，0.5x费率
}
```

### PinRequest - Pin请求结构
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PinRequest<T: Config> {
    pub creator: T::AccountId,                    // 请求创建者
    pub replicas: u32,                            // 所需副本数
    pub tier: PinTier,                            // Pin层级
    pub subject_type: SubjectType,                // 主体类型
    pub subject_id: u64,                          // 主体ID
    pub created_at: BlockNumberFor<T>,            // 创建时间
}
```

### OperatorInfo - 运营者信息
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct OperatorInfo<T: Config> {
    pub peer_id: BoundedVec<u8, T::MaxPeerIdLen>,     // IPFS节点PeerID
    pub capacity_gib: u32,                            // 声明存储容量（GiB）
    pub endpoint_hash: T::Hash,                       // IPFS Cluster API端点哈希
    pub cert_fingerprint: Option<T::Hash>,            // TLS证书指纹
    pub status: OperatorStatus,                       // 运营者状态
    pub registered_at: BlockNumberFor<T>,             // 注册时间
    pub layer: OperatorLayer,                         // 运营者分层
    pub priority: u8,                                 // 优先级（0-255）
}
```

### OperatorLayer - 运营者分层
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OperatorLayer {
    Core = 0,       // 核心节点（团队运营，最高优先级）
    Community = 1,  // 社区节点（社区运营，中等优先级）
    External = 2,   // 外部节点（第三方运营，低优先级）
}
```

### OperatorStatus - 运营者状态
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum OperatorStatus {
    Active = 0,     // 活跃状态，正常接受Pin任务
    Suspended = 1,  // 暂停状态，暂时不接受新任务
    Banned = 2,     // 封禁状态，完全禁止服务
}
```

### SlaStats - SLA统计数据
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct SlaStats<T: Config> {
    pub pinned_bytes: u64,        // 已固定内容总字节数
    pub probe_ok: u32,            // 健康巡检成功次数
    pub probe_fail: u32,          // 健康巡检失败次数
    pub migration_triggered: u32,  // 触发迁移的次数
}
```

### TierConfig - 分层配置
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct TierConfig {
    pub default_replicas: u32,       // 默认副本数
    pub check_interval_blocks: u32,  // 巡检间隔（区块数）
    pub price_multiplier_bps: u16,   // 价格倍数（基点，万分之一）
}
```

### SubjectType - 主体类型
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum SubjectType {
    General,      // 逝者相关内容（整合了text文本、media媒体、works作品等）
    General,         // 墓位相关内容
    OtcOrder,     // 供奉品内容
    OtcOrder,      // OTC订单内容
    Evidence,      // 证据类数据
    Custom(BoundedVec<u8, ConstU32<32>>), // 自定义域（预留扩展）
}
```

### PinMetadata - Pin元信息
```rust
#[derive(Decode, Encode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct PinMetadata<T: Config> {
    pub replicas: u32,                        // 当前副本数
    pub size: u64,                            // 内容大小（字节）
    pub created_at: BlockNumberFor<T>,        // 创建时间
    pub last_activity: BlockNumberFor<T>,     // 最后活动时间
}
```

## 存储项（Storage Items）

### PendingPins - 待处理Pin请求
```rust
pub type PendingPins<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,                    // CID哈希
    PinRequest<T>,              // Pin请求信息
    OptionQuery,
>;
```
存储所有待处理的Pin请求，键为CID哈希，值为Pin请求详细信息。

### PinAssignments - Pin分配记录
```rust
pub type PinAssignments<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,                            // CID哈希
    BoundedVec<T::AccountId, T::MaxOperators>, // 分配的运营者列表
    ValueQuery,
>;
```
记录每个CID分配给哪些运营者进行Pin操作。

### PinSuccess - Pin成功记录
```rust
pub type PinSuccess<T> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    T::Hash,                    // CID哈希
    Blake2_128Concat,
    T::AccountId,               // 运营者账户
    bool,                       // 是否成功
    ValueQuery,
>;
```
记录每个运营者对特定CID的Pin操作结果。

### PinStateOf - Pin状态映射
```rust
pub type PinStateOf<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,                    // CID哈希
    PinState,                   // Pin状态
    ValueQuery,
>;
```
跟踪每个CID的当前状态：Requested/Pinning/Pinned/Degraded/Failed。

### PinMetaOf - Pin元信息
```rust
pub type PinMetaOf<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::Hash,                    // CID哈希
    PinMetadata<T>,            // Pin元信息
    OptionQuery,
>;
```
存储Pin的元信息，包括副本数、大小、时间戳等。

### OperatorInfoOf - 运营者信息
```rust
pub type OperatorInfoOf<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,               // 运营者账户
    OperatorInfo<T>,           // 运营者信息
    OptionQuery,
>;
```
存储注册运营者的详细信息。

### OperatorSlaOf - 运营者SLA统计
```rust
pub type OperatorSlaOf<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,               // 运营者账户
    SlaStats<T>,               // SLA统计
    ValueQuery,
>;
```
记录每个运营者的服务质量统计数据。

### TierConfigs - 分层配置
```rust
pub type TierConfigs<T> = StorageMap<
    _,
    Blake2_128Concat,
    PinTier,                    // Pin层级
    TierConfig,                 // 层级配置
    ValueQuery,
>;
```
存储三个Pin层级的配置参数。

### SubjectQuotaUsed - 主体配额使用记录
```rust
pub type SubjectQuotaUsed<T> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    SubjectType,                // 主体类型
    Blake2_128Concat,
    u64,                        // 主体ID
    (T::Balance, BlockNumberFor<T>), // 使用量和重置时间
    ValueQuery,
>;
```
跟踪每个主体的配额使用情况。

### SubjectGracePeriod - 主体宽限期
```rust
pub type SubjectGracePeriod<T> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    SubjectType,                // 主体类型
    Blake2_128Concat,
    u64,                        // 主体ID
    BlockNumberFor<T>,          // 宽限期结束时间
    OptionQuery,
>;
```
记录进入宽限期的主体及其结束时间。

### PendingUnregistrations - 待注销运营者
```rust
pub type PendingUnregistrations<T> = StorageMap<
    _,
    Blake2_128Concat,
    T::AccountId,               // 运营者账户
    BlockNumberFor<T>,          // 宽限期结束时间
    OptionQuery,
>;
```
记录申请注销但还在宽限期的运营者。

### RegisteredDomains - 注册域记录
```rust
pub type RegisteredDomains<T> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,                    // 域名
    SubjectType,                // 对应的主体类型
    OptionQuery,
>;
```
记录已注册的内容域及其对应的主体类型。

## 主要调用方法（Dispatchable Functions）

### 用户接口

#### `request_pin`
请求固定IPFS内容到网络

```rust
#[pallet::call_index(0)]
pub fn request_pin(
    origin: OriginFor<T>,
    cid: Vec<u8>,                   // IPFS CID
    tier: PinTier,                  // Pin层级
) -> DispatchResult
```

**功能**：提交Pin请求，系统自动分配运营者并开始固定过程
**权限**：任何签名账户
**扣费**：从调用者账户直接扣除费用

#### `request_pin_for_subject`
为逝者请求固定内容

```rust
#[pallet::call_index(1)]
pub fn request_pin_for_subject(
    origin: OriginFor<T>,
    subject_id: u64,              // 逝者ID
    cid: Vec<u8>,                   // IPFS CID
    tier: Option<PinTier>,          // Pin层级（可选，默认Standard）
) -> DispatchResult
```

**功能**：为特定逝者固定内容，支持配额和SubjectFunding扣费
**权限**：逝者的owner账户
**扣费**：优先使用免费配额，然后从SubjectFunding扣费

#### `request_pin_for_grave`
为墓位请求固定内容

```rust
#[pallet::call_index(2)]
pub fn request_pin_for_grave(
    origin: OriginFor<T>,
    grave_id: u64,                  // 墓位ID
    cid: Vec<u8>,                   // IPFS CID
    tier: Option<PinTier>,          // Pin层级（可选）
) -> DispatchResult
```

**功能**：为特定墓位固定内容
**权限**：墓位的owner或管理员
**扣费**：从墓位对应的SubjectFunding账户扣费

#### `request_unpin`
请求取消固定内容

```rust
#[pallet::call_index(3)]
pub fn request_unpin(
    origin: OriginFor<T>,
    cid: Vec<u8>,                   // IPFS CID
) -> DispatchResult
```

**功能**：取消对特定内容的固定，释放存储空间
**权限**：原Pin请求的发起者

#### `fund_subject_account`
为主体账户充值

```rust
#[pallet::call_index(4)]
pub fn fund_subject_account(
    origin: OriginFor<T>,
    subject_type: SubjectType,      // 主体类型
    subject_id: u64,                // 主体ID
    amount: T::Balance,             // 充值金额
) -> DispatchResult
```

**功能**：为SubjectFunding账户充值，支持后续的自动扣费
**权限**：任何签名账户

### 运营者接口

#### `register_operator`
注册为IPFS运营者

```rust
#[pallet::call_index(10)]
pub fn register_operator(
    origin: OriginFor<T>,
    peer_id: Vec<u8>,               // IPFS节点PeerID
    capacity_gib: u32,              // 声明存储容量（GiB）
    endpoint_hash: T::Hash,         // IPFS Cluster API端点哈希
    cert_fingerprint: Option<T::Hash>, // TLS证书指纹
    layer: OperatorLayer,           // 运营者分层
    priority: u8,                   // 优先级（0-255）
) -> DispatchResult
```

**功能**：注册成为IPFS存储服务提供者
**权限**：任何签名账户
**要求**：需要锁定最小保证金，容量不少于最小要求

#### `unregister_operator`
注销运营者身份

```rust
#[pallet::call_index(11)]
pub fn unregister_operator(
    origin: OriginFor<T>,
) -> DispatchResult
```

**功能**：申请注销运营者，进入宽限期等待Pin迁移
**权限**：注册的运营者

#### `report_pin_success`
报告Pin操作成功

```rust
#[pallet::call_index(12)]
pub fn report_pin_success(
    origin: OriginFor<T>,
    cid: Vec<u8>,                   // IPFS CID
    actual_size: Option<u64>,       // 实际文件大小
) -> DispatchResult
```

**功能**：运营者报告Pin操作成功完成
**权限**：被分配该CID的运营者

#### `report_pin_failure`
报告Pin操作失败

```rust
#[pallet::call_index(13)]
pub fn report_pin_failure(
    origin: OriginFor<T>,
    cid: Vec<u8>,                   // IPFS CID
) -> DispatchResult
```

**功能**：运营者报告Pin操作失败
**权限**：被分配该CID的运营者

#### `update_operator_info`
更新运营者信息

```rust
#[pallet::call_index(14)]
pub fn update_operator_info(
    origin: OriginFor<T>,
    peer_id: Option<Vec<u8>>,       // 新PeerID
    capacity_gib: Option<u32>,      // 新容量
    endpoint_hash: Option<T::Hash>, // 新端点哈希
    cert_fingerprint: Option<Option<T::Hash>>, // 新证书指纹
    priority: Option<u8>,           // 新优先级
) -> DispatchResult
```

**功能**：更新运营者的配置信息
**权限**：注册的运营者

### 治理接口

#### `update_tier_config`
更新分层配置参数

```rust
#[pallet::call_index(20)]
pub fn update_tier_config(
    origin: OriginFor<T>,
    tier: PinTier,                  // 要更新的层级
    config: TierConfig,             // 新的配置参数
) -> DispatchResult
```

**功能**：调整Pin层级的配置参数
**权限**：治理源（GovernanceOrigin）

#### `set_operator_status`
设置运营者状态

```rust
#[pallet::call_index(21)]
pub fn set_operator_status(
    origin: OriginFor<T>,
    operator: T::AccountId,         // 运营者账户
    status: OperatorStatus,         // 新状态
) -> DispatchResult
```

**功能**：管理运营者的服务状态
**权限**：治理源

#### `set_operator_layer`
设置运营者分层

```rust
#[pallet::call_index(22)]
pub fn set_operator_layer(
    origin: OriginFor<T>,
    operator: T::AccountId,         // 运营者账户
    layer: OperatorLayer,           // 新分层
) -> DispatchResult
```

**功能**：调整运营者的分层级别
**权限**：治理源

## 事件定义（Events）

### Pin相关事件

#### `PinRequested`
Pin请求已提交
```rust
PinRequested {
    cid_hash: T::Hash,              // CID哈希
    payer: T::AccountId,            // 付费账户
    replicas: u32,                  // 副本数
    tier: PinTier,                  // Pin层级
}
```

#### `PinAssigned`
Pin已分配给运营者
```rust
PinAssigned {
    cid_hash: T::Hash,                           // CID哈希
    operators: Vec<T::AccountId>,                // 分配的运营者列表
}
```

#### `PinConfirmed`
运营者确认Pin成功
```rust
PinConfirmed {
    cid_hash: T::Hash,              // CID哈希
    operator: T::AccountId,         // 运营者账户
}
```

#### `PinFailed`
运营者报告Pin失败
```rust
PinFailed {
    cid_hash: T::Hash,              // CID哈希
    operator: T::AccountId,         // 运营者账户
}
```

#### `PinStateChanged`
Pin状态发生变化
```rust
PinStateChanged {
    cid_hash: T::Hash,              // CID哈希
    old_state: PinState,            // 旧状态
    new_state: PinState,            // 新状态
}
```

#### `UnpinRequested`
请求取消Pin
```rust
UnpinRequested {
    cid_hash: T::Hash,              // CID哈希
    requester: T::AccountId,        // 请求者
}
```

### 运营者相关事件

#### `OperatorRegistered`
运营者注册成功
```rust
OperatorRegistered {
    operator: T::AccountId,         // 运营者账户
    peer_id: Vec<u8>,               // PeerID
    capacity: u32,                  // 容量
    layer: OperatorLayer,           // 分层
}
```

#### `OperatorUnregistered`
运营者注销成功
```rust
OperatorUnregistered {
    operator: T::AccountId,         // 运营者账户
}
```

#### `OperatorUpdated`
运营者信息已更新
```rust
OperatorUpdated {
    operator: T::AccountId,         // 运营者账户
}
```

#### `OperatorStatusChanged`
运营者状态改变
```rust
OperatorStatusChanged {
    operator: T::AccountId,         // 运营者账户
    old_status: OperatorStatus,     // 旧状态
    new_status: OperatorStatus,     // 新状态
}
```

#### `OperatorLayerChanged`
运营者分层改变
```rust
OperatorLayerChanged {
    operator: T::AccountId,         // 运营者账户
    old_layer: OperatorLayer,       // 旧分层
    new_layer: OperatorLayer,       // 新分层
}
```

### 财务相关事件

#### `SubjectFunded`
主体账户充值
```rust
SubjectFunded {
    subject_type: SubjectType,      // 主体类型
    subject_id: u64,                // 主体ID
    amount: T::Balance,             // 充值金额
}
```

#### `BillingCharged`
周期扣费执行
```rust
BillingCharged {
    subject_type: SubjectType,      // 主体类型
    subject_id: u64,                // 主体ID
    amount: T::Balance,             // 扣费金额
    layer: PinTier,                 // Pin层级
}
```

#### `GracePeriodEntered`
进入宽限期
```rust
GracePeriodEntered {
    subject_type: SubjectType,      // 主体类型
    subject_id: u64,                // 主体ID
}
```

### 配置相关事件

#### `TierConfigUpdated`
分层配置已更新
```rust
TierConfigUpdated {
    tier: PinTier,                  // Pin层级
}
```

## 错误定义（Errors）

### 基础错误

#### `CidTooLong`
CID长度超过限制
```rust
/// CID长度不能超过最大限制
CidTooLong,
```

#### `InsufficientBalance`
账户余额不足
```rust
/// 账户余额不足以支付费用
InsufficientBalance,
```

#### `NotAuthorized`
操作未授权
```rust
/// 用户无权限执行此操作
NotAuthorized,
```

### 运营者相关错误

#### `OperatorNotFound`
运营者不存在
```rust
/// 指定的运营者未注册
OperatorNotFound,
```

#### `NotOperator`
不是运营者
```rust
/// 账户未注册为运营者
NotOperator,
```

#### `OperatorSuspended`
运营者已暂停
```rust
/// 运营者处于暂停状态，无法接受新任务
OperatorSuspended,
```

#### `OperatorBanned`
运营者已封禁
```rust
/// 运营者已被封禁，无法提供服务
OperatorBanned,
```

#### `InsufficientBond`
保证金不足
```rust
/// 运营者保证金低于最小要求
InsufficientBond,
```

#### `InsufficientCapacity`
容量不足
```rust
/// 运营者声明容量低于最小要求
InsufficientCapacity,
```

#### `OperatorHasActivePins`
运营者有活跃Pin
```rust
/// 运营者仍有正在服务的Pin，无法立即注销
OperatorHasActivePins,
```

### Pin相关错误

#### `PinNotFound`
Pin记录不存在
```rust
/// 指定的Pin请求不存在
PinNotFound,
```

#### `PinAlreadyExists`
Pin已存在
```rust
/// CID已经被Pin，无法重复Pin
PinAlreadyExists,
```

#### `NoAvailableOperators`
无可用运营者
```rust
/// 没有可用的运营者来处理Pin请求
NoAvailableOperators,
```

### 主体相关错误

#### `GeneralNotFound`
逝者不存在
```rust
/// 指定的逝者记录不存在
GeneralNotFound,
```

#### `GeneralNotFound`
墓位不存在
```rust
/// 指定的墓位记录不存在
GeneralNotFound,
```

#### `SubjectInGracePeriod`
主体在宽限期
```rust
/// 主体处于宽限期，无法提交新的Pin请求
SubjectInGracePeriod,
```

#### `QuotaExceeded`
配额已用完
```rust
/// 主体本月免费配额已用完
QuotaExceeded,
```

## 配置参数（Configuration）

### 基础类型配置
```rust
pub trait Config: frame_system::Config {
    /// 事件类型
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

    /// 货币操作接口
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

    /// 余额类型
    type Balance: Parameter + Member + AtLeast32BitUnsigned + Codec + From<u64> + TypeInfo + MaxEncodedLen;

    /// 治理源，用于管理员操作
    type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    /// 权重信息提供者
    type WeightInfo: WeightInfo;
}
```

### 容量限制配置
```rust
/// CID哈希最大长度（字节）
#[pallet::constant]
type MaxCidHashLen: Get<u32>;

/// PeerID最大长度（字节）
#[pallet::constant]
type MaxPeerIdLen: Get<u32>;

/// 单个Pin最大运营者数量
#[pallet::constant]
type MaxOperators: Get<u32>;
```

### 经济参数配置
```rust
/// 运营者最小保证金
#[pallet::constant]
type MinOperatorBond: Get<Self::Balance>;

/// 运营者最小声明容量（GiB）
#[pallet::constant]
type MinCapacityGiB: Get<u32>;

/// 每月免费配额（单位：Balance）
#[pallet::constant]
type MonthlyPublicFeeQuota: Get<Self::Balance>;

/// 配额重置周期（区块数）
#[pallet::constant]
type QuotaResetPeriod: Get<BlockNumberFor<Self>>;

/// 默认扣费周期（区块数）
#[pallet::constant]
type DefaultBillingPeriod: Get<BlockNumberFor<Self>>;
```

### 账户配置
```rust
/// IPFS公共池账户
#[pallet::constant]
type IpfsPoolAccount: Get<Self::AccountId>;

/// 运营者托管账户
#[pallet::constant]
type OperatorEscrowAccount: Get<Self::AccountId>;

/// 费用收集账户
#[pallet::constant]
type FeeCollector: Get<Self::AccountId>;

/// SubjectFunding账户派生用PalletId
#[pallet::constant]
type SubjectPalletId: Get<PalletId>;
```

### 外部接口配置
```rust
/// 逝者创建者提供者
type CreatorProvider: CreatorProvider<Self::AccountId, u64>;

/// 逝者Owner提供者
type OwnerProvider: OwnerProvider<Self::AccountId, u64>;

/// 墓位Owner提供者
type GeneralOwnerProvider: OwnerProvider<Self::AccountId, u64>;
```

### 域配置
```rust
/// 逝者域编码
#[pallet::constant]
type GeneralDomain: Get<u8>;

/// 墓位域编码
#[pallet::constant]
type GeneralDomain: Get<u8>;
```

## 使用示例

### 1. 运营者注册示例

```rust
use frame_support::{
    dispatch::DispatchResult,
    traits::{Get, Currency, ReservableCurrency},
};
use sp_runtime::traits::{Hash, Saturating};

// 注册为Core层运营者
let result = IpfsService::register_operator(
    RuntimeOrigin::signed(operator_account),
    b"QmOperatorPeerID12345".to_vec(),  // PeerID
    1024,                               // 1TB容量
    T::Hashing::hash(b"https://ipfs-cluster.example.com"), // 端点哈希
    Some(T::Hashing::hash(b"cert_fingerprint")), // 证书指纹
    OperatorLayer::Core,                // Core层
    10,                                 // 优先级10
);

assert_ok!(result);
```

### 2. 自动Pin（为逝者）示例

```rust
// 为逝者固定主图
let result = IpfsService::request_pin_for_subject(
    RuntimeOrigin::signed(subject_owner),
    subject_id,
    b"QmMainImageCID123".to_vec(),     // 主图CID
    Some(PinTier::Standard),           // 标准层级
);

assert_ok!(result);

// 系统会自动：
// 1. 检查caller是否为subject的owner
// 2. 优先使用免费配额
// 3. 配额不足时从SubjectFunding扣费
// 4. 自动分配3个运营者（Standard层级默认副本数）
// 5. 通知运营者开始Pin操作
```

### 3. SubjectFunding充值示例

```rust
// 为逝者的SubjectFunding账户充值
let result = IpfsService::fund_subject_account(
    RuntimeOrigin::signed(funder),
    SubjectType::General,
    subject_id,
    100 * DUST,                        // 充值100 DUST
);

assert_ok!(result);

// 充值后系统会：
// 1. 将资金转入派生的SubjectFunding账户
// 2. 如果该主体在宽限期，自动退出宽限期
// 3. 发出SubjectFunded事件
```

### 4. 业务模块集成示例

```rust
// 在业务pallet的Config中声明依赖
pub trait Config: frame_system::Config {
    type IpfsPinner: IpfsPinner<Self::AccountId, Self::Balance>;
}

// 在extrinsic中自动Pin内容
#[pallet::weight(10_000)]
pub fn create_subject_with_image(
    origin: OriginFor<T>,
    name_cid: Vec<u8>,
    main_image_cid: Vec<u8>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 创建逝者记录
    let subject_id = Self::do_create_subject(&who, name_cid.clone())?;

    // 自动Pin名称CID（Critical层级，重要数据）
    T::IpfsPinner::pin_cid_for_subject(
        who.clone(),
        subject_id,
        name_cid,
        Some(PinTier::Critical),
    )?;

    // 自动Pin主图CID（Standard层级）
    T::IpfsPinner::pin_cid_for_subject(
        who,
        subject_id,
        main_image_cid,
        Some(PinTier::Standard),
    )?;

    Ok(())
}
```

### 5. 健康检查和故障处理示例

```rust
// OCW自动执行的健康检查逻辑（简化）
pub fn offchain_worker(block_number: BlockNumberFor<T>) {
    // 1. 遍历所有Pinned状态的CID
    for (cid_hash, _) in PinStateOf::<T>::iter() {
        let assignments = PinAssignments::<T>::get(&cid_hash);

        // 2. 检查每个运营者的Pin状态
        for operator in assignments.iter() {
            if let Some(operator_info) = OperatorInfoOf::<T>::get(operator) {
                // 3. 通过HTTP请求检查IPFS Cluster状态
                let pin_status = Self::check_pin_status(&operator_info, &cid_hash);

                match pin_status {
                    Ok(true) => {
                        // Pin健康，更新SLA统计
                        Self::update_sla_success(operator);
                    },
                    _ => {
                        // Pin异常，触发迁移流程
                        Self::trigger_migration(&cid_hash, operator);
                        Self::update_sla_failure(operator);
                    }
                }
            }
        }
    }
}
```

### 6. 三层扣费机制示例

```rust
// 周期扣费逻辑（每7天执行一次）
pub fn do_billing_cycle() -> DispatchResult {
    for (cid_hash, pin_request) in PendingPins::<T>::iter() {
        let cost = Self::calculate_pin_cost(&pin_request);

        // 扣费顺序：IpfsPool → SubjectFunding → GracePeriod
        let charged = Self::try_charge_from_ipfs_pool(cost)
            .or_else(|| Self::try_charge_from_subject_funding(&pin_request, cost))
            .unwrap_or_else(|| {
                // 资金不足，进入宽限期
                Self::enter_grace_period(&pin_request);
                false
            });

        if charged {
            Self::deposit_event(Event::BillingCharged {
                subject_type: pin_request.subject_type,
                subject_id: pin_request.subject_id,
                amount: cost,
                layer: pin_request.tier,
            });
        }
    }

    Ok(())
}
```

## 集成说明

### 与subject模块集成

该模块通过`IpfsPinner` trait为`pallet-subject`提供自动Pin服务。

**架构说明**：pallet-subject内部整合了text（文本）、media（媒体）、works（作品）等内容类型子模块，所有这些内容的IPFS存储都通过统一的`SubjectType::General`进行管理。

```rust
// 在subject创建时自动Pin相关内容
impl<T: Config> IpfsPinner<T::AccountId, T::Balance> for Pallet<T> {
    fn pin_cid_for_subject(
        caller: T::AccountId,
        subject_id: u64,
        cid: Vec<u8>,
        tier: Option<PinTier>,
    ) -> DispatchResult {
        // 1. 验证caller权限（必须是subject的owner）
        // 2. 检查宽限期状态
        // 3. 计算费用并尝试扣费
        // 4. 提交Pin请求
        // 5. 分配运营者
    }
}
```

**自动Pin场景：**
- 逝者档案基础信息（Critical层级）
- 媒体内容（subject::media子模块）：照片、视频、音频（Standard层级）
- 文本内容（subject::text子模块）：文章、留言（Standard层级）
- 作品数据（subject::works子模块）：AI训练数据（Standard层级）
- 证据文件（evidence pallet）：法律文件（Critical层级）

### ContentRegistry接口集成

为新业务模块提供一键集成能力：

```rust
pub trait ContentRegistry {
    fn register_content(
        domain: Vec<u8>,        // 域名标识
        subject_id: u64,        // 主体ID
        cid: Vec<u8>,          // 内容CID
        tier: PinTier,         // Pin层级
    ) -> DispatchResult;
}

// 新模块使用示例
impl ContentRegistry for Pallet<T> {
    fn register_content(
        domain: Vec<u8>,
        subject_id: u64,
        cid: Vec<u8>,
        tier: PinTier,
    ) -> DispatchResult {
        // 自动注册域，派生SubjectFunding账户，提交Pin请求
    }
}
```

## OCW健康巡检机制

### 巡检流程

1. **定期触发**：每个区块都会触发OCW，根据配置的巡检间隔决定是否执行检查
2. **状态查询**：通过HTTP请求查询运营者的IPFS Cluster API
3. **结果评估**：根据查询结果更新Pin状态和SLA统计
4. **故障处理**：检测到故障时触发自动迁移流程

### 巡检实现

```rust
pub fn offchain_worker(block_number: BlockNumberFor<T>) {
    // 检查是否到达巡检时间
    if !Self::should_run_health_check(block_number) {
        return;
    }

    // 获取所有需要检查的Pin
    let pins_to_check = Self::get_pins_for_health_check(block_number);

    for (cid_hash, pin_tier) in pins_to_check {
        let operators = PinAssignments::<T>::get(&cid_hash);

        for operator in operators {
            // 检查单个运营者的Pin状态
            Self::check_operator_pin_health(&cid_hash, &operator);
        }

        // 评估整体Pin健康状况
        Self::evaluate_overall_pin_health(&cid_hash);
    }
}

fn check_operator_pin_health(
    cid_hash: &T::Hash,
    operator: &T::AccountId,
) {
    if let Some(operator_info) = OperatorInfoOf::<T>::get(operator) {
        // 构造IPFS Cluster API请求
        let api_url = Self::construct_cluster_api_url(&operator_info, cid_hash);

        // 发送HTTP请求
        let request = rt_offchain::http::Request::get(&api_url);
        let pending = request.send().map_err(|e| {
            log::error!("Failed to send request: {:?}", e);
            e
        });

        if let Ok(response) = pending.and_then(|p| p.wait()) {
            let status_code = response.code;
            let body = response.body().collect::<Vec<u8>>();

            // 解析响应并更新状态
            Self::process_health_check_response(cid_hash, operator, status_code, body);
        } else {
            // 请求失败，记录故障
            Self::record_health_check_failure(cid_hash, operator);
        }
    }
}
```

### 故障迁移机制

```rust
fn trigger_migration(cid_hash: &T::Hash, failed_operator: &T::AccountId) {
    // 1. 标记Pin为降级状态
    PinStateOf::<T>::mutate(cid_hash, |state| {
        *state = PinState::Degraded;
    });

    // 2. 从分配列表中移除故障运营者
    PinAssignments::<T>::mutate(cid_hash, |assignments| {
        assignments.retain(|op| op != failed_operator);
    });

    // 3. 选择新的运营者
    if let Some(new_operator) = Self::select_replacement_operator(cid_hash) {
        // 4. 分配给新运营者
        PinAssignments::<T>::mutate(cid_hash, |assignments| {
            assignments.try_push(new_operator.clone()).ok();
        });

        // 5. 发出迁移事件
        Self::deposit_event(Event::PinMigrated {
            cid_hash: *cid_hash,
            from_operator: failed_operator.clone(),
            to_operator: new_operator,
        });
    }
}
```

## 三层分层策略详解

### Critical层（关键级别）
- **目标**：最重要的内容，需要最高可靠性
- **副本数**：5个副本，确保高冗余
- **巡检频率**：6小时一次（3,600个区块）
- **费率倍数**：1.5x基础费率
- **适用场景**：
  - 逝者身份证明文件
  - 重要法律文档
  - 核心纪念内容
  - 证据文件

**配置示例**：
```rust
TierConfig {
    default_replicas: 5,           // 5个副本
    check_interval_blocks: 3_600,  // 6小时巡检
    price_multiplier_bps: 15_000,  // 1.5x费率（15000/10000）
}
```

### Standard层（标准级别）
- **目标**：一般内容的平衡方案
- **副本数**：3个副本，平衡可靠性和成本
- **巡检频率**：24小时一次（14,400个区块）
- **费率倍数**：1.0x基础费率（默认）
- **适用场景**：
  - 逝者照片
  - 一般纪念内容
  - 墓位封面
  - 常规文档

**配置示例**：
```rust
TierConfig {
    default_replicas: 3,           // 3个副本
    check_interval_blocks: 14_400, // 24小时巡检
    price_multiplier_bps: 10_000,  // 1.0x费率
}
```

### Temporary层（临时级别）
- **目标**：临时或测试内容，低成本方案
- **副本数**：1个副本，最低成本
- **巡检频率**：7天一次（100,800个区块）
- **费率倍数**：0.5x基础费率
- **适用场景**：
  - 测试内容
  - 临时文件
  - 草稿内容
  - 低重要性媒体

**配置示例**：
```rust
TierConfig {
    default_replicas: 1,            // 1个副本
    check_interval_blocks: 100_800, // 7天巡检
    price_multiplier_bps: 5_000,    // 0.5x费率
}
```

### 费率计算公式

```rust
fn calculate_pin_cost(
    base_cost: T::Balance,
    pin_tier: PinTier,
    file_size: u64,
) -> T::Balance {
    let tier_config = TierConfigs::<T>::get(pin_tier);
    let multiplier = tier_config.price_multiplier_bps;

    // 费用 = 基础费用 × 文件大小 × 层级倍数 × 副本数
    base_cost
        .saturating_mul(file_size.into())
        .saturating_mul(multiplier.into())
        .saturating_div(10_000u32.into()) // 基点转换
        .saturating_mul(tier_config.default_replicas.into())
}
```

## 最佳实践

### 1. 运营者节点部署

**硬件要求**：
- CPU：8核心以上
- 内存：16GB以上
- 存储：SSD，声明容量的120%
- 网络：千兆带宽，低延迟

**软件配置**：
```bash
# IPFS节点配置
ipfs config Datastore.StorageMax "2TB"
ipfs config --json Swarm.ConnMgr.HighWater 2000
ipfs config --json Swarm.ConnMgr.LowWater 1000

# IPFS Cluster配置
ipfs-cluster-service init
# 配置TLS证书
# 配置API端点
```

**安全配置**：
- 使用HTTPS端点
- 配置TLS证书验证
- 定期更新IPFS版本
- 监控存储容量使用

### 2. Pin层级选择策略

**Critical层级适用于**：
```rust
// 重要身份文件
IpfsPinner::pin_cid_for_subject(
    caller,
    subject_id,
    identity_document_cid,
    Some(PinTier::Critical), // 使用Critical层级
)?;
```

**Standard层级适用于**：
```rust
// 一般照片内容
IpfsPinner::pin_cid_for_subject(
    caller,
    subject_id,
    photo_cid,
    Some(PinTier::Standard), // 使用Standard层级
)?;
```

**Temporary层级适用于**：
```rust
// 测试或草稿内容
IpfsPinner::pin_cid_for_subject(
    caller,
    subject_id,
    draft_cid,
    Some(PinTier::Temporary), // 使用Temporary层级
)?;
```

### 3. 资金管理最佳实践

**定期充值策略**：
```rust
// 建议每月检查一次SubjectFunding余额
let balance = Self::subject_funding_balance(SubjectType::General, subject_id);
let monthly_cost = Self::estimate_monthly_cost(subject_id);

if balance < monthly_cost.saturating_mul(2u32.into()) {
    // 余额不足两个月费用，建议充值
    Self::fund_subject_account(
        origin,
        SubjectType::General,
        subject_id,
        monthly_cost.saturating_mul(6u32.into()), // 充值6个月
    )?;
}
```

**避免宽限期**：
- 设置余额监控告警
- 预充值足够的费用
- 使用自动充值脚本

### 4. 业务模块集成最佳实践

**推荐集成方式**：
```rust
// 1. 在Config中声明依赖
type IpfsPinner: IpfsPinner<Self::AccountId, Self::Balance>;

// 2. 在业务逻辑中自动Pin
#[pallet::weight(10_000)]
pub fn create_content(
    origin: OriginFor<T>,
    content_cid: Vec<u8>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;

    // 先创建业务记录
    let content_id = Self::do_create_content(&who, content_cid.clone())?;

    // 然后自动Pin内容
    T::IpfsPinner::pin_cid_for_subject(
        who,
        subject_id,
        content_cid,
        Some(PinTier::Standard),
    )?;

    Ok(())
}
```

**不推荐的做法**：
```rust
// 不要直接调用底层接口
IpfsService::request_pin(...)?; // ❌ 不推荐

// 不要忘记Pin重要内容
Self::create_content(...)?; // ❌ 创建内容但忘记Pin
```

### 5. 监控和运维

**关键指标监控**：
- Pin成功率（应 > 99%）
- 巡检失败率（应 < 1%）
- 运营者在线率（应 > 99.9%）
- 资金池余额状况
- 宽限期主体数量

**告警设置**：
```rust
// 示例：监控指标阈值
const SLA_FAILURE_THRESHOLD: u32 = 100; // 失败次数阈值
const GRACE_PERIOD_ALERT_COUNT: u32 = 10; // 宽限期主体告警阈值

if operator_sla.probe_fail > SLA_FAILURE_THRESHOLD {
    // 发出运营者SLA告警
    Self::alert_operator_sla_degraded(operator);
}
```

**日常维护**：
- 定期检查OCW运行状态
- 监控IPFS Cluster日志
- 验证Pin状态一致性
- 清理过期的临时Pin

### 6. 安全防护措施

**保证金管理**：
```rust
// 动态调整保证金要求
fn update_minimum_bond(new_bond: T::Balance) {
    MinOperatorBond::<T>::put(new_bond);

    // 通知现有运营者补充保证金
    for (operator, _) in OperatorInfoOf::<T>::iter() {
        let reserved = T::Currency::reserved_balance(&operator);
        if reserved < new_bond {
            Self::request_additional_bond(&operator, new_bond - reserved);
        }
    }
}
```

**DoS攻击防护**：
- 配额限制
- 费率门槛
- 宽限期保护
- 运营者容量限制

**数据一致性保证**：
- Pin状态原子性更新
- OCW重试机制
- 故障恢复流程
- 定期一致性检查

通过遵循这些最佳实践，可以确保IPFS存储服务的稳定性、安全性和经济性，为整个纪念平台提供可靠的去中心化存储基础设施。

## 与 Divination（占卜）模块集成

`stardust-ipfs` 模块为 `divination` 占卜模块体系提供去中心化存储服务。占卜模块使用 `SubjectType::Custom` 来注册自定义内容域。

### 占卜模块IPFS存储需求概览

| 模块 | 存储内容 | 建议Pin层级 | 估算大小 |
|------|---------|-------------|----------|
| **meihua** (梅花易数) | AI解读结果 | Standard | 2-10 KB |
| **tarot** (塔罗牌) | AI解读结果 | Standard | 2-10 KB |
| **xiaoliuren** (小六壬) | 占问事项 + AI解读 | Temporary/Standard | 2-11 KB |
| **daliuren** (大六壬) | 占问事项 + 解读内容 | Temporary/Standard | 5-21 KB |
| **liuyao** (六爻) | 卦辞 + 占问事项 | Standard/Temporary | 1-6 KB |
| **qimen** (奇门遁甲) | AI综合解读 | Standard | 5-30 KB |
| **ziwei** (紫微斗数) | 归档命盘数据（计划中） | Temporary | 3-10 KB |
| **ai** (AI解读) | 解读内容 + 摘要 | Standard/Temporary | 2-22 KB |
| **market** (服务市场) | 头像/问答/评价/举报证据等 | 多层级 | 5-150 KB |
| **nft** (占卜NFT) | 图片/描述/动画 | Critical/Standard | 100KB-55MB |
| **affiliate** (推广治理) | 治理提案内容 | Standard | 2-16 KB |

### 使用 Custom SubjectType

占卜模块应使用 `SubjectType::Custom` 来注册自己的内容域：

```rust
// 定义占卜相关的域标识
pub const DIVINATION_AI_DOMAIN: &[u8] = b"divination-ai";
pub const DIVINATION_MARKET_DOMAIN: &[u8] = b"divination-market";
pub const DIVINATION_NFT_DOMAIN: &[u8] = b"divination-nft";
pub const DIVINATION_CHART_DOMAIN: &[u8] = b"divination-chart";

// 创建 SubjectType
fn divination_subject_type(domain: &[u8]) -> SubjectType {
    SubjectType::Custom(
        BoundedVec::try_from(domain.to_vec())
            .expect("domain name within bounds")
    )
}

// 使用示例
let ai_subject = SubjectType::Custom(b"divination-ai".to_vec().try_into().unwrap());
let market_subject = SubjectType::Custom(b"divination-market".to_vec().try_into().unwrap());
```

### 占卜模块域名规范

| 域名 | 用途 | subject_id含义 |
|------|------|---------------|
| `divination-ai` | AI解读内容 | request_id |
| `divination-market` | 服务市场内容 | order_id / provider_id |
| `divination-nft` | NFT媒体内容 | nft_id |
| `divination-chart` | 命盘归档数据 | chart_id |
| `divination-gov` | 治理提案内容 | proposal_id |

### 集成示例：AI解读模块

```rust
// 在 AI 解读模块中自动 Pin 解读结果
#[pallet::weight(30_000_000)]
pub fn submit_ai_interpretation(
    origin: OriginFor<T>,
    chart_id: u64,
    interpretation_cid: Vec<u8>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 1. 验证权限（AI预言机）
    ensure!(T::AiOracle::is_authorized(&who), Error::<T>::NotAuthorized);
    
    // 2. 获取chart信息
    let chart = ChartById::<T>::get(chart_id).ok_or(Error::<T>::ChartNotFound)?;
    
    // 3. 自动Pin AI解读内容（使用Standard层级）
    T::IpfsPinner::pin_cid(
        who.clone(),
        SubjectType::Custom(b"divination-ai".to_vec().try_into().unwrap()),
        chart_id,
        interpretation_cid.clone(),
        Some(PinTier::Standard),
    )?;
    
    // 4. 更新chart的interpretation_cid
    ChartById::<T>::mutate(chart_id, |c| {
        if let Some(chart) = c {
            chart.interpretation_cid = Some(
                BoundedVec::try_from(interpretation_cid).unwrap()
            );
        }
    });
    
    Self::deposit_event(Event::AiInterpretationSubmitted { chart_id });
    Ok(())
}
```

### 集成示例：NFT媒体存储

```rust
// 铸造占卜NFT时自动Pin媒体文件
#[pallet::weight(50_000_000)]
pub fn mint_nft(
    origin: OriginFor<T>,
    divination_type: DivinationType,
    result_id: u64,
    name: Vec<u8>,
    image_cid: Vec<u8>,           // 图片 IPFS CID（必需）
    animation_cid: Option<Vec<u8>>, // 动画 IPFS CID（可选）
    royalty_rate: u16,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 创建NFT记录
    let nft_id = Self::do_create_nft(&who, divination_type, result_id, name)?;
    
    // Pin图片（Critical层级 - 高价值数字资产）
    T::IpfsPinner::pin_cid(
        who.clone(),
        SubjectType::Custom(b"divination-nft".to_vec().try_into().unwrap()),
        nft_id,
        image_cid.clone(),
        Some(PinTier::Critical),
    )?;
    
    // 如果有动画，也Pin（Standard层级）
    if let Some(anim_cid) = animation_cid.clone() {
        T::IpfsPinner::pin_cid(
            who.clone(),
            SubjectType::Custom(b"divination-nft".to_vec().try_into().unwrap()),
            nft_id,
            anim_cid,
            Some(PinTier::Standard),
        )?;
    }
    
    Ok(())
}
```

### 集成示例：服务市场

```rust
// 服务市场提交解读答案时Pin内容
#[pallet::weight(40_000_000)]
pub fn submit_answer(
    origin: OriginFor<T>,
    order_id: u64,
    answer_cid: Vec<u8>,  // 解读内容 IPFS CID
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    // 验证是该订单的服务提供者
    let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
    ensure!(order.provider == who, Error::<T>::NotProvider);
    
    // Pin解读内容（Standard层级 - 付费服务内容）
    T::IpfsPinner::pin_cid(
        who.clone(),
        SubjectType::Custom(b"divination-market".to_vec().try_into().unwrap()),
        order_id,
        answer_cid.clone(),
        Some(PinTier::Standard),
    )?;
    
    // 更新订单状态
    Self::do_complete_order(order_id, answer_cid)?;
    
    Ok(())
}

// 举报时Pin证据（Critical层级 - 法律相关）
#[pallet::weight(50_000_000)]
pub fn report_provider(
    origin: OriginFor<T>,
    provider: T::AccountId,
    report_type: ReportType,
    evidence_cid: Vec<u8>,  // 证据 IPFS CID
    description: Vec<u8>,
) -> DispatchResult {
    let who = ensure_signed(origin)?;
    
    let report_id = Self::do_create_report(&who, &provider, report_type, description)?;
    
    // Pin证据（Critical层级 - 争议/法律相关）
    T::IpfsPinner::pin_cid(
        who.clone(),
        SubjectType::Custom(b"divination-market".to_vec().try_into().unwrap()),
        report_id,
        evidence_cid,
        Some(PinTier::Critical),
    )?;
    
    Ok(())
}
```

### Pin层级选择指南（占卜模块）

| 内容类型 | 建议层级 | 理由 |
|----------|---------|------|
| NFT主图 | **Critical** | 高价值数字资产，需要最高可靠性 |
| 举报证据 | **Critical** | 法律/争议相关，不可丢失 |
| AI解读结果 | **Standard** | 核心业务数据，需要可靠存储 |
| 服务解读回答 | **Standard** | 付费服务内容，需要可靠存储 |
| 服务提供者介绍 | **Standard** | 商业信息，长期展示 |
| NFT描述/动画 | **Standard** | 资产附属信息 |
| 治理提案内容 | **Standard** | 治理记录，需要保留 |
| 占卜问题描述 | **Temporary** | 用户输入，可重新提交 |
| 追问/回复内容 | **Temporary** | 交互记录，时效性强 |
| 评价内容 | **Temporary** | 评价详情，辅助信息 |
| 归档命盘 | **Temporary** | 可从链上解档恢复 |

### SubjectFunding 充值（占卜模块）

为占卜服务的存储费用预先充值：

```rust
// 为服务提供者的内容存储充值
IpfsService::fund_subject_account(
    RuntimeOrigin::signed(provider),
    SubjectType::Custom(b"divination-market".to_vec().try_into().unwrap()),
    provider_id,  // 服务提供者ID作为subject_id
    100 * DUST,   // 充值100 DUST
)?;

// 为NFT系列的媒体存储充值
IpfsService::fund_subject_account(
    RuntimeOrigin::signed(creator),
    SubjectType::Custom(b"divination-nft".to_vec().try_into().unwrap()),
    collection_id,
    500 * DUST,   // 充值500 DUST（NFT媒体较大）
)?;
```

### 删除时的Unpin处理

当用户删除占卜相关记录时，应同步请求Unpin关联的IPFS内容：

```rust
// 删除命盘时清理IPFS存储
fn delete_chart(chart_id: u64) -> DispatchResult {
    let chart = ChartById::<T>::get(chart_id)?;
    
    // 1. 删除链上数据
    ChartById::<T>::remove(chart_id);
    
    // 2. 如果有AI解读CID，请求Unpin
    if let Some(cid) = chart.interpretation_cid {
        T::IpfsPinner::request_unpin(cid.to_vec())?;
    }
    
    // 3. 返还存储押金
    T::Currency::unreserve(&chart.creator, chart.deposit);
    
    Ok(())
}

// 销毁NFT时清理媒体文件
fn burn_nft(nft_id: u64) -> DispatchResult {
    let nft = Nfts::<T>::get(nft_id)?;
    
    // Unpin图片
    T::IpfsPinner::request_unpin(nft.metadata.image_cid.to_vec())?;
    
    // Unpin动画（如果有）
    if let Some(anim_cid) = nft.metadata.animation_cid {
        T::IpfsPinner::request_unpin(anim_cid.to_vec())?;
    }
    
    // 删除NFT记录
    Nfts::<T>::remove(nft_id);
    
    Ok(())
}
```

### 配置参数建议（占卜模块）

```rust
// runtime/src/lib.rs

parameter_types! {
    /// 占卜内容每 KB 存储基础费率
    pub const DivinationStorageFeePerKb: Balance = 1_000_000_000; // 0.001 DUST
    
    /// 占卜AI解读月免费配额
    pub const AiInterpretationMonthlyQuota: Balance = 50_000_000_000; // 50 KB等值
    
    /// NFT媒体最大大小 (50 MB)
    pub const MaxNftMediaSize: u64 = 50 * 1024 * 1024;
    
    /// 占卜模块扣费周期（区块数，约7天）
    pub const DivinationBillingPeriod: BlockNumber = 201_600; // 7天 @ 3秒/块
    
    /// 占卜模块宽限期（区块数，约7天）
    pub const DivinationGracePeriod: BlockNumber = 201_600;
}
```

### 相关文档

- [Divination IPFS集成指南](../divination/IPFS_INTEGRATION_GUIDE.md) - 详细的占卜模块IPFS存储需求分析
- [存储押金与删除机制分析](../divination/STORAGE_DEPOSIT_AND_DELETION_ANALYSIS.md) - 占卜模块存储押金设计