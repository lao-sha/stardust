#![cfg_attr(not(feature = "std"), no_std)]

/// Stardust智能群聊系统 - 简化版本
///
/// 这是一个最小可用版本，提供基础的群聊功能：
/// - 创建群组
/// - 发送消息
/// - 加入/离开群组
/// - 获取群组信息和消息
///
/// 高级功能（量子抗性加密、AI决策等）将在后续版本中逐步添加

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{Get, Randomness, UnixTime, Currency, ReservableCurrency, ExistenceRequirement},
    PalletId,
};
use sp_runtime::traits::{Saturating, Zero};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{
    BoundedVec, RuntimeDebug,
};
use sp_std::vec::Vec;

// 使用 chat-common 的共享类型
pub use pallet_chat_common::{MessageType, EncryptionMode};

// 函数级中文注释：导入共享媒体工具库用于媒体验证和哈希计算
use media_utils::{
    ImageValidator, VideoValidator, AudioValidator, MediaError
};

// MessageType 和 EncryptionMode 已移至 pallet-chat-common，通过 pub use 重新导出

/// 群组成员角色
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

/// 群组信息结构
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct GroupInfo<AccountId, GroupName, GroupDescription> {
    pub id: u64,
    pub name: GroupName,
    pub description: Option<GroupDescription>,
    pub owner: AccountId,
    pub member_count: u32,
    pub encryption_mode: EncryptionMode,
    pub is_public: bool,
    pub created_at: u64,
}

/// 群组消息结构
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct GroupMessage<AccountId, BoundedString> {
    pub id: u64,
    pub group_id: u64,
    pub sender: AccountId,
    pub content: BoundedString,
    pub message_type: MessageType,
    pub timestamp: u64,
}

/// 群组成员结构
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct GroupMember<AccountId> {
    pub account_id: AccountId,
    pub role: MemberRole,
    pub joined_at: u64,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// 余额类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// Pallet配置trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 随机数生成器（用于群组ID和密钥生成）
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 时间服务（用于消息时间戳）
        type TimeProvider: UnixTime;

        /// 货币类型（用于保证金）
        type Currency: ReservableCurrency<Self::AccountId>;

        /// 群组名称最大长度
        #[pallet::constant]
        type MaxGroupNameLen: Get<u32>;

        /// 群组描述最大长度
        #[pallet::constant]
        type MaxGroupDescriptionLen: Get<u32>;

        /// 群组最大成员数
        #[pallet::constant]
        type MaxGroupMembers: Get<u32>;

        /// 单用户最大群组数
        #[pallet::constant]
        type MaxGroupsPerUser: Get<u32>;

        /// 消息内容最大长度
        #[pallet::constant]
        type MaxMessageLen: Get<u32>;

        /// 群组历史消息保留数量
        #[pallet::constant]
        type MaxGroupMessageHistory: Get<u32>;

        /// IPFS CID最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 加密密钥最大长度
        #[pallet::constant]
        type MaxKeyLen: Get<u32>;

        /// Pallet ID（用于生成内部账户）
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// 消息发送频率限制（每分钟）
        #[pallet::constant]
        type MessageRateLimit: Get<u32>;

        /// 群组创建冷却期（区块数）
        #[pallet::constant]
        type GroupCreationCooldown: Get<BlockNumberFor<Self>>;

        /// 创建群组保证金兜底值（DUST数量，pricing不可用时使用）
        #[pallet::constant]
        type GroupDeposit: Get<BalanceOf<Self>>;

        /// 创建群组保证金USD价值（精度10^6，5_000_000 = 5 USDT）
        #[pallet::constant]
        type GroupDepositUsd: Get<u64>;

        /// 保证金计算器（统一的 USD 价值动态计算）
        type DepositCalculator: pallet_trading_common::DepositCalculator<BalanceOf<Self>>;

        /// 国库账户（罚没资金转入）
        type TreasuryAccount: Get<Self::AccountId>;

        /// 治理权限来源（用于处理违规）
        type GovernanceOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;

        /// Weight信息（用于基准测试）
        type WeightInfo: WeightInfo;
    }

    // WeightInfo trait 和实现已移至 weights.rs

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // 类型别名
    type GroupNameOf<T> = BoundedVec<u8, <T as Config>::MaxGroupNameLen>;
    type GroupDescriptionOf<T> = BoundedVec<u8, <T as Config>::MaxGroupDescriptionLen>;
    type MessageContentOf<T> = BoundedVec<u8, <T as Config>::MaxMessageLen>;

    /// 存储项：群组信息
    #[pallet::storage]
    #[pallet::getter(fn groups)]
    pub type Groups<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        GroupInfo<T::AccountId, GroupNameOf<T>, GroupDescriptionOf<T>>,
    >;

    /// 存储项：群组成员
    #[pallet::storage]
    #[pallet::getter(fn group_members)]
    pub type GroupMembers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64, // group_id
        Blake2_128Concat,
        T::AccountId, // member
        GroupMember<T::AccountId>,
    >;

    /// 存储项：用户的群组列表
    #[pallet::storage]
    #[pallet::getter(fn user_groups)]
    pub type UserGroups<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxGroupsPerUser>,
        ValueQuery,
    >;

    /// 存储项：群组消息
    #[pallet::storage]
    #[pallet::getter(fn group_messages)]
    pub type GroupMessages<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64, // group_id
        Blake2_128Concat,
        u64, // message_id
        GroupMessage<T::AccountId, MessageContentOf<T>>,
    >;

    /// 存储项：下一个消息ID（按群组）
    #[pallet::storage]
    #[pallet::getter(fn next_message_id)]
    pub type NextMessageId<T: Config> = StorageMap<_, Blake2_128Concat, u64, u64, ValueQuery>;

    /// 存储项：群组保证金
    #[pallet::storage]
    #[pallet::getter(fn group_deposits)]
    pub type GroupDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // group_id
        BalanceOf<T>,
    >;

    /// 存储项：封禁群组列表
    #[pallet::storage]
    #[pallet::getter(fn banned_groups)]
    pub type BannedGroups<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // group_id
        u64, // 封禁时间戳
    >;

    /// 事件定义
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 群组已创建 [创建者, 群组ID]
        GroupCreated { creator: T::AccountId, group_id: u64 },
        /// 群组消息已发送 [群组ID, 发送者, 消息ID]
        GroupMessageSent { group_id: u64, sender: T::AccountId, message_id: u64 },
        /// 成员已加入群组 [群组ID, 成员]
        MemberJoined { group_id: u64, member: T::AccountId },
        /// 成员已离开群组 [群组ID, 成员]
        MemberLeft { group_id: u64, member: T::AccountId },
        /// 群组加密模式已更新 [群组ID, 新模式]
        GroupEncryptionUpdated { group_id: u64, new_mode: u8 },
        /// 群组已解散 [群组ID]
        GroupDisbanded { group_id: u64 },
        /// 群组保证金已锁定
        GroupDepositLocked { group_id: u64, owner: T::AccountId, amount: BalanceOf<T> },
        /// 群组保证金已释放
        GroupDepositReleased { group_id: u64, owner: T::AccountId, amount: BalanceOf<T> },
        /// 群组保证金已扣除
        GroupDepositSlashed { group_id: u64, owner: T::AccountId, amount: BalanceOf<T>, reason: GroupViolationType },
        /// 群组已被封禁
        GroupBanned { group_id: u64, reason: GroupViolationType },
    }

    /// 群组违规类型
    #[derive(Clone, Copy, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, RuntimeDebug)]
    pub enum GroupViolationType {
        /// 轻微违规（5%）- 广告、轻微骚扰
        Minor,
        /// 一般违规（10%）- 不当言论
        Moderate,
        /// 严重违规（20%）- 色情、暴力内容
        Severe,
        /// 特别严重（50%）- 诈骗、违法
        Critical,
        /// 永久封禁（100%）- 多次违规
        PermanentBan,
    }

    impl GroupViolationType {
        /// 获取扣除比例（基点，10000 = 100%）
        pub fn slash_bps(&self) -> u16 {
            match self {
                GroupViolationType::Minor => 500,       // 5%
                GroupViolationType::Moderate => 1000,   // 10%
                GroupViolationType::Severe => 2000,     // 20%
                GroupViolationType::Critical => 5000,   // 50%
                GroupViolationType::PermanentBan => 10000, // 100%
            }
        }

        /// 是否需要封禁群组
        pub fn should_ban(&self) -> bool {
            matches!(self, GroupViolationType::PermanentBan)
        }
    }

    /// 错误定义
    #[pallet::error]
    pub enum Error<T> {
        /// 群组不存在
        GroupNotFound,
        /// 消息不存在
        MessageNotFound,
        /// 非群组成员
        NotGroupMember,
        /// 非群组管理员
        NotGroupAdmin,
        /// 非群组所有者
        NotGroupOwner,
        /// 群组已满
        GroupFull,
        /// 用户群组数量已达上限
        UserGroupLimitExceeded,
        /// 群组名称太长
        GroupNameTooLong,
        /// 群组描述太长
        GroupDescriptionTooLong,
        /// 消息内容太长
        MessageContentTooLong,
        /// 发送频率过快
        TooFrequent,
        /// 已是群组成员
        AlreadyMember,
        /// 不是群组成员
        NotMember,
        /// 群组ID生成失败（重试次数过多）
        GroupIdGenerationFailed,
        /// 函数级中文注释：媒体验证错误 - 无效的媒体格式
        InvalidMediaFormat,
        /// 函数级中文注释：媒体验证错误 - 文件太大
        MediaFileTooLarge,
        /// 函数级中文注释：媒体验证错误 - 文件太小
        MediaFileTooSmall,
        /// 函数级中文注释：媒体验证错误 - 不支持的媒体类型
        UnsupportedMediaType,
        /// 余额不足
        InsufficientBalance,
        /// 群组已被封禁
        GroupBanned,
        /// 保证金不存在
        DepositNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建群组
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_group())]
        pub fn create_group(
            origin: OriginFor<T>,
            name: Vec<u8>,
            description: Option<Vec<u8>>,
            encryption_mode: u8,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证名称长度
            let bounded_name = GroupNameOf::<T>::try_from(name)
                .map_err(|_| Error::<T>::GroupNameTooLong)?;

            // 验证描述长度
            let bounded_description = if let Some(desc) = description {
                Some(GroupDescriptionOf::<T>::try_from(desc)
                    .map_err(|_| Error::<T>::GroupDescriptionTooLong)?)
            } else {
                None
            };

            // 检查用户群组数量限制
            let user_groups = Self::user_groups(&who);
            ensure!(
                user_groups.len() < T::MaxGroupsPerUser::get() as usize,
                Error::<T>::UserGroupLimitExceeded
            );

            // 计算并锁定保证金（5 USDT 等值的 DUST）
            let deposit = Self::calculate_deposit_amount();
            T::Currency::reserve(&who, deposit)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // 生成唯一的10位数随机群组ID
            let group_id = Self::generate_unique_group_id()?;

            // 获取当前时间戳
            let now = T::TimeProvider::now().as_secs();

            // 转换加密模式
            let encryption_mode_enum = EncryptionMode::from_u8(encryption_mode);

            // 创建群组信息
            let group_info = GroupInfo {
                id: group_id,
                name: bounded_name,
                description: bounded_description,
                owner: who.clone(),
                member_count: 1,
                encryption_mode: encryption_mode_enum,
                is_public,
                created_at: now,
            };

            // 存储群组信息
            Groups::<T>::insert(&group_id, &group_info);

            // 记录保证金
            GroupDeposits::<T>::insert(&group_id, deposit);

            Self::deposit_event(Event::GroupDepositLocked {
                group_id,
                owner: who.clone(),
                amount: deposit,
            });

            // 添加创建者为群主
            let owner_member = GroupMember {
                account_id: who.clone(),
                role: MemberRole::Owner,
                joined_at: now,
            };
            GroupMembers::<T>::insert(&group_id, &who, &owner_member);

            // 更新用户群组列表
            UserGroups::<T>::mutate(&who, |groups| {
                let _ = groups.try_push(group_id);
            });

            // 发出事件
            Self::deposit_event(Event::GroupCreated { creator: who, group_id });

            Ok(())
        }

        /// 发送群组消息
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::send_group_message())]
        pub fn send_group_message(
            origin: OriginFor<T>,
            group_id: u64,
            content: Vec<u8>,
            message_type: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证群组存在
            let _group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;

            // 验证发送者是群组成员
            ensure!(
                GroupMembers::<T>::contains_key(&group_id, &who),
                Error::<T>::NotGroupMember
            );

            // 验证内容长度
            let bounded_content = MessageContentOf::<T>::try_from(content)
                .map_err(|_| Error::<T>::MessageContentTooLong)?;

            // 转换消息类型
            let message_type_enum = MessageType::from_u8(message_type);

            // 生成消息ID
            let message_id = Self::next_message_id(&group_id);
            let next_id = message_id.saturating_add(1);
            NextMessageId::<T>::insert(&group_id, next_id);

            // 获取当前时间戳
            let now = T::TimeProvider::now().as_secs();

            // 创建消息
            let message = GroupMessage {
                id: message_id,
                group_id,
                sender: who.clone(),
                content: bounded_content,
                message_type: message_type_enum,
                timestamp: now,
            };

            // 存储消息
            GroupMessages::<T>::insert(&group_id, &message_id, &message);

            // 发出事件
            Self::deposit_event(Event::GroupMessageSent { group_id, sender: who, message_id });

            Ok(())
        }

        /// 加入群组
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::join_group())]
        pub fn join_group(
            origin: OriginFor<T>,
            group_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证群组存在
            let mut group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;

            // 验证用户不是已有成员
            ensure!(
                !GroupMembers::<T>::contains_key(&group_id, &who),
                Error::<T>::AlreadyMember
            );

            // 验证群组未满
            ensure!(
                group.member_count < T::MaxGroupMembers::get(),
                Error::<T>::GroupFull
            );

            // 验证用户群组数量限制
            let user_groups = Self::user_groups(&who);
            ensure!(
                user_groups.len() < T::MaxGroupsPerUser::get() as usize,
                Error::<T>::UserGroupLimitExceeded
            );

            // 获取当前时间戳
            let now = T::TimeProvider::now().as_secs();

            // 添加成员
            let member = GroupMember {
                account_id: who.clone(),
                role: MemberRole::Member,
                joined_at: now,
            };
            GroupMembers::<T>::insert(&group_id, &who, &member);

            // 更新群组成员数
            group.member_count = group.member_count.saturating_add(1);
            Groups::<T>::insert(&group_id, &group);

            // 更新用户群组列表
            UserGroups::<T>::mutate(&who, |groups| {
                let _ = groups.try_push(group_id);
            });

            // 发出事件
            Self::deposit_event(Event::MemberJoined { group_id, member: who });

            Ok(())
        }

        /// 离开群组
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::leave_group())]
        pub fn leave_group(
            origin: OriginFor<T>,
            group_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证群组存在
            let mut group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;

            // 验证用户是群组成员
            let member = GroupMembers::<T>::get(&group_id, &who)
                .ok_or(Error::<T>::NotMember)?;

            // 如果是群主，解散群组
            if member.role == MemberRole::Owner {
                return Self::do_disband_group(group_id);
            }

            // 移除成员
            GroupMembers::<T>::remove(&group_id, &who);

            // 更新群组成员数
            group.member_count = group.member_count.saturating_sub(1);
            Groups::<T>::insert(&group_id, &group);

            // 从用户群组列表中移除
            UserGroups::<T>::mutate(&who, |groups| {
                groups.retain(|&g| g != group_id);
            });

            // 发出事件
            Self::deposit_event(Event::MemberLeft { group_id, member: who });

            Ok(())
        }

        /// 解散群组（仅群主）
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::disband_group())]
        pub fn disband_group(
            origin: OriginFor<T>,
            group_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证群组存在
            let _group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;

            // 验证用户是群主
            let member = GroupMembers::<T>::get(&group_id, &who)
                .ok_or(Error::<T>::NotMember)?;
            ensure!(member.role == MemberRole::Owner, Error::<T>::NotGroupOwner);

            Self::do_disband_group(group_id)
        }

        /// 处理群组违规（治理权限）
        /// 
        /// 根据违规类型扣除保证金并执行相应处罚
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn handle_group_violation(
            origin: OriginFor<T>,
            group_id: u64,
            violation_type: GroupViolationType,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            // 验证群组存在
            let group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;
            let owner = group.owner.clone();

            // 检查是否已被封禁
            ensure!(!BannedGroups::<T>::contains_key(&group_id), Error::<T>::GroupBanned);

            // 获取保证金
            let deposit = GroupDeposits::<T>::get(&group_id)
                .ok_or(Error::<T>::DepositNotFound)?;

            // 计算扣除金额
            let slash_bps = violation_type.slash_bps();
            let slash_amount = deposit.saturating_mul(slash_bps.into()) / 10000u32.into();

            if !slash_amount.is_zero() {
                // 解除锁定
                T::Currency::unreserve(&owner, slash_amount);
                
                // 转入国库
                let treasury = T::TreasuryAccount::get();
                let _ = T::Currency::transfer(
                    &owner,
                    &treasury,
                    slash_amount,
                    ExistenceRequirement::AllowDeath,
                );

                // 更新保证金记录
                let remaining = deposit.saturating_sub(slash_amount);
                if remaining.is_zero() {
                    GroupDeposits::<T>::remove(&group_id);
                } else {
                    GroupDeposits::<T>::insert(&group_id, remaining);
                }

                Self::deposit_event(Event::GroupDepositSlashed {
                    group_id,
                    owner: owner.clone(),
                    amount: slash_amount,
                    reason: violation_type,
                });
            }

            // 处理封禁
            if violation_type.should_ban() {
                let now = T::TimeProvider::now().as_secs();
                BannedGroups::<T>::insert(&group_id, now);

                Self::deposit_event(Event::GroupBanned {
                    group_id,
                    reason: violation_type,
                });
            }

            Ok(())
        }

        /// 补充群组保证金
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn top_up_group_deposit(
            origin: OriginFor<T>,
            group_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证群组存在
            let group = Self::groups(&group_id).ok_or(Error::<T>::GroupNotFound)?;

            // 验证是群主
            ensure!(group.owner == who, Error::<T>::NotGroupOwner);

            // 检查是否已被封禁
            ensure!(!BannedGroups::<T>::contains_key(&group_id), Error::<T>::GroupBanned);

            // 锁定保证金
            T::Currency::reserve(&who, amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // 更新保证金记录
            let current = GroupDeposits::<T>::get(&group_id).unwrap_or_else(Zero::zero);
            let new_total = current.saturating_add(amount);
            GroupDeposits::<T>::insert(&group_id, new_total);

            Self::deposit_event(Event::GroupDepositLocked {
                group_id,
                owner: who,
                amount,
            });

            Ok(())
        }
    }

    // 内部函数
    impl<T: Config> Pallet<T> {
        /// 生成唯一的10位数随机群组ID
        ///
        /// 该函数生成一个范围在 1,000,000,000 到 9,999,999,999 之间的随机数
        /// 并确保该ID在系统中唯一（未被使用）
        ///
        /// # 返回值
        /// - Ok(u64): 成功生成的唯一群组ID
        /// - Err(Error): 如果重试100次后仍无法生成唯一ID则返回错误
        ///
        /// # 实现细节
        /// - 使用链上随机数生成器获取高质量随机源
        /// - 最多重试100次防止无限循环
        /// - 使用11位数ID范围(约900亿),在群组数量达到数十亿之前碰撞概率极低
        fn generate_unique_group_id() -> Result<u64, Error<T>> {
            const MAX_RETRIES: u32 = 100;
            const MIN_ID: u64 = 10_000_000_000;  // 11位数最小值
            const MAX_ID: u64 = 99_999_999_999;  // 11位数最大值
            const ID_RANGE: u64 = MAX_ID - MIN_ID + 1;

            for attempt in 0..MAX_RETRIES {
                // 使用唯一的subject防止同一区块内产生相同随机数
                let mut subject = b"group_id_".to_vec();
                subject.extend_from_slice(&attempt.to_le_bytes());
                let (random_seed, _) = T::Randomness::random(&subject);
                let random_bytes = random_seed.as_ref();

                // 从随机种子中提取u64
                // 注意: 如果random_bytes长度不足8字节,使用0填充
                let mut bytes = [0u8; 8];
                let len = random_bytes.len().min(8);
                bytes[..len].copy_from_slice(&random_bytes[..len]);
                let random_u64 = u64::from_le_bytes(bytes);

                // 将随机数映射到10位数范围内
                let group_id = MIN_ID + (random_u64 % ID_RANGE);

                // 检查ID是否已存在
                if !Groups::<T>::contains_key(group_id) {
                    return Ok(group_id);
                }

                // ID已存在,继续重试
            }

            // 重试次数耗尽,返回错误
            Err(Error::<T>::GroupIdGenerationFailed)
        }

        /// 解散群组的内部实现
        fn do_disband_group(group_id: u64) -> DispatchResult {
            // 获取群组信息以获取群主
            let group = Groups::<T>::get(&group_id).ok_or(Error::<T>::GroupNotFound)?;
            let owner = group.owner.clone();

            // 1. 释放保证金
            if let Some(deposit) = GroupDeposits::<T>::take(&group_id) {
                T::Currency::unreserve(&owner, deposit);
                Self::deposit_event(Event::GroupDepositReleased {
                    group_id,
                    owner: owner.clone(),
                    amount: deposit,
                });
            }

            // 2. 收集所有成员账户
            let members: Vec<T::AccountId> = GroupMembers::<T>::iter_prefix(&group_id)
                .map(|(account, _)| account)
                .collect();

            // 3. 从每个成员的群组列表中移除该群组
            for member in members.iter() {
                UserGroups::<T>::mutate(member, |groups| {
                    groups.retain(|&g| g != group_id);
                });
            }

            // 4. 移除所有成员记录
            let _result = GroupMembers::<T>::clear_prefix(&group_id, u32::MAX, None);

            // 5. 移除群组信息
            Groups::<T>::remove(&group_id);

            // 6. 移除群组消息
            let _result = GroupMessages::<T>::clear_prefix(&group_id, u32::MAX, None);

            // 7. 发出事件
            Self::deposit_event(Event::GroupDisbanded { group_id });

            Ok(())
        }

        /// 函数级详细中文注释：验证媒体内容
        ///
        /// 根据消息类型验证媒体文件格式和完整性：
        /// - Image: 使用 ImageValidator 验证图片格式
        /// - Voice: 使用 AudioValidator 验证音频格式
        /// - File: 根据实际格式选择验证器
        /// - Text: 不需要验证
        ///
        /// # 参数
        /// - `content`: 媒体文件的二进制数据
        /// - `message_type`: 消息类型
        ///
        /// # 返回
        /// - `Ok(())`: 验证成功
        /// - `Err(Error)`: 验证失败
        pub fn validate_media_content(
            content: &[u8],
            message_type: &MessageType,
        ) -> Result<(), Error<T>> {
            match message_type {
                MessageType::Text | MessageType::System | MessageType::AI => {
                    // 文本/系统/AI消息不需要媒体验证
                    Ok(())
                },
                MessageType::Image => {
                    // 验证图片格式
                    ImageValidator::validate(content)
                        .map(|_| ())
                        .map_err(Self::convert_media_error)
                },
                MessageType::Voice => {
                    // 验证音频格式
                    AudioValidator::validate(content)
                        .map(|_| ())
                        .map_err(Self::convert_media_error)
                },
                MessageType::Video => {
                    // 验证视频格式
                    VideoValidator::validate(content)
                        .map(|_| ())
                        .map_err(Self::convert_media_error)
                },
                MessageType::File => {
                    // 文件类型需要根据实际内容判断
                    // 先尝试图片验证
                    if ImageValidator::validate(content).is_ok() {
                        return Ok(());
                    }
                    // 再尝试视频验证
                    if VideoValidator::validate(content).is_ok() {
                        return Ok(());
                    }
                    // 最后尝试音频验证
                    if AudioValidator::validate(content).is_ok() {
                        return Ok(());
                    }
                    // 都不匹配则返回错误
                    Err(Error::<T>::UnsupportedMediaType)
                },
            }
        }

        /// 函数级中文注释：将 MediaError 转换为 pallet Error
        fn convert_media_error(err: MediaError) -> Error<T> {
            match err {
                MediaError::FileTooSmall => Error::<T>::MediaFileTooSmall,
                MediaError::FileTooLarge => Error::<T>::MediaFileTooLarge,
                MediaError::UnsupportedFormat => Error::<T>::UnsupportedMediaType,
                MediaError::UnsupportedMimeType => Error::<T>::UnsupportedMediaType,
                MediaError::InvalidHeader => Error::<T>::InvalidMediaFormat,
                MediaError::InvalidPngHeader => Error::<T>::InvalidMediaFormat,
                _ => Error::<T>::InvalidMediaFormat,
            }
        }

        /// 计算保证金金额（5 USDT 等值的 DUST）
        /// 
        /// 使用统一的 DepositCalculator trait 计算
        pub fn calculate_deposit_amount() -> BalanceOf<T> {
            use pallet_trading_common::DepositCalculator;
            T::DepositCalculator::calculate_deposit(
                T::GroupDepositUsd::get(),
                T::GroupDeposit::get(),
            )
        }
    }
}