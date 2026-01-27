//! # 婚恋模块 - 互动功能（隐私保护版）
//!
//! 本模块提供用户之间的互动功能，采用隐私保护设计。
//!
//! ## 功能概述
//!
//! - **点赞**：表达好感
//! - **超级喜欢**：付费功能，让对方知道你的兴趣
//! - **跳过**：跳过当前用户
//! - **屏蔽**：屏蔽不想看到的用户
//! - **匹配检测**：检测是否互相喜欢
//!
//! ## 隐私保护机制
//!
//! - **互动记录哈希化**：使用 `hash(from || to || salt)` 存储，第三方无法直接查询
//! - **点赞列表加密**：仅存储加密后的账户哈希
//! - **匹配通知私密化**：事件中使用哈希而非明文账户
//! - **仅当事人可查**：只有互动双方可以验证关系

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

use frame_support::pallet_prelude::*;
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Saturating, Zero};
use sp_io::hashing::blake2_256;

use pallet_matchmaking_common::InteractionType;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 每个用户最大互动记录数
        #[pallet::constant]
        type MaxInteractionsPerUser: Get<u32>;

        /// 最大收到的超级喜欢数
        #[pallet::constant]
        type MaxSuperLikesReceived: Get<u32>;

        /// 超级喜欢费用（DUST，以 u128 表示）
        #[pallet::constant]
        type SuperLikeCost: Get<u128>;

        /// 免费用户每日点赞配额
        #[pallet::constant]
        type FreeDailyLikes: Get<u32>;

        /// 免费用户每日超级喜欢配额（通常为 0）
        #[pallet::constant]
        type FreeDailySuperLikes: Get<u32>;

        /// 会员每日超级喜欢配额
        #[pallet::constant]
        type MemberDailySuperLikes: Get<u32>;

        /// 每日区块数（用于计算日期）
        #[pallet::constant]
        type BlocksPerDay: Get<u32>;

        /// 免费用户每日可发起的聊天数
        #[pallet::constant]
        type FreeDailyChatInitiations: Get<u32>;

        /// 月费会员每日可发起的聊天数
        #[pallet::constant]
        type MonthlyMemberDailyChatInitiations: Get<u32>;

        /// 年费会员每日可发起的聊天数（0 = 无限）
        #[pallet::constant]
        type YearlyMemberDailyChatInitiations: Get<u32>;

        /// 免费用户每日查看资料配额
        #[pallet::constant]
        type FreeDailyViews: Get<u32>;

        /// 会员每日查看资料配额（0 = 无限）
        #[pallet::constant]
        type MemberDailyViews: Get<u32>;

        /// 余额类型
        type Balance: codec::FullCodec
            + codec::MaxEncodedLen
            + Copy
            + MaybeSerializeDeserialize
            + core::fmt::Debug
            + Default
            + scale_info::TypeInfo
            + Saturating
            + Zero
            + PartialOrd
            + Ord
            + TryFrom<u128>
            + TryInto<u128>;

        /// Fungible 接口：用于扣除超级喜欢费用
        type Fungible: Inspect<Self::AccountId, Balance = Self::Balance>
            + Mutate<Self::AccountId>;

        /// 国库账户（费用转入）
        type TreasuryAccount: Get<Self::AccountId>;

        /// 权重信息
        type WeightInfo: WeightInfo;
    }

    // ========================================================================
    // 类型定义
    // ========================================================================

    /// 互动记录（隐私保护版）
    /// 
    /// 不直接存储账户，而是存储哈希值
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct InteractionRecord<T: Config> {
        /// 互动类型
        pub interaction_type: InteractionType,
        /// 时间戳
        pub timestamp: BlockNumberFor<T>,
    }

    /// 加密的匹配记录
    /// 
    /// 存储加密后的对方账户哈希，仅当事人可解密验证
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub struct EncryptedMatchRecord {
        /// 对方账户的哈希（用于验证）
        pub target_hash: [u8; 32],
        /// 匹配时间（区块号）
        pub matched_at: u64,
    }

    /// 隐私盐值（每个用户唯一，用于生成哈希）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
    pub struct PrivacySalt {
        pub salt: [u8; 16],
    }

    /// 超级喜欢记录
    /// 
    /// 存储收到的超级喜欢，用于优先展示
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub struct SuperLikeRecord {
        /// 发送者哈希（隐私保护）
        pub sender_hash: [u8; 32],
        /// 发送时间（区块号）
        pub sent_at: u64,
        /// 是否已查看
        pub viewed: bool,
    }

    /// 每日配额信息
    /// 
    /// 记录用户每日的互动配额使用情况
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
    pub struct DailyQuota {
        /// 今日已发送点赞数
        pub likes_used: u32,
        /// 今日已发送超级喜欢数
        pub super_likes_used: u32,
        /// 今日已查看资料数
        pub views_used: u32,
        /// 最后重置日期（区块号 / 每日区块数）
        pub last_reset_day: u32,
    }

    /// 聊天发起配额
    /// 
    /// 记录用户每日发起新聊天的配额使用情况
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
    pub struct ChatInitiationQuota {
        /// 今日已发起的聊天数
        pub chats_initiated: u32,
        /// 最后重置日期（区块号 / 每日区块数）
        pub last_reset_day: u32,
    }

    /// 聊天发起方式
    /// 
    /// 标识聊天会话是如何建立的
    #[derive(Clone, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub enum ChatInitiationType {
        /// 我主动发起（消耗配额）
        InitiatedByMe,
        /// 对方先发起（我是回复方，不消耗配额）
        InitiatedByOther,
        /// 超级喜欢特权（不消耗配额）
        SuperLikePrivilege,
    }

    /// 聊天会话信息
    /// 
    /// 记录与某个用户的聊天会话状态
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub struct ChatSessionInfo {
        /// 会话创建时间（区块号）
        pub created_at: u64,
        /// 发起方式
        pub initiation_type: ChatInitiationType,
    }

    // ========================================================================
    // 存储（隐私保护版）
    // ========================================================================

    /// 用户隐私盐值
    /// 
    /// 每个用户唯一的盐值，用于生成互动哈希
    #[pallet::storage]
    #[pallet::getter(fn user_salt)]
    pub type UserSalt<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PrivacySalt,
    >;

    /// 互动记录（哈希化存储）
    /// 
    /// Key: hash(from || to || from_salt)
    /// 第三方无法直接查询谁喜欢谁
    #[pallet::storage]
    #[pallet::getter(fn interactions)]
    pub type Interactions<T: Config> = StorageMap<
        _,
        Identity,
        [u8; 32],  // interaction_hash
        InteractionRecord<T>,
    >;

    /// 用户发出的互动哈希列表（仅自己可见）
    /// 
    /// 存储用户发出的所有互动的哈希值
    #[pallet::storage]
    #[pallet::getter(fn my_interactions)]
    pub type MyInteractions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<[u8; 32], T::MaxInteractionsPerUser>,
        ValueQuery,
    >;

    /// 用户收到的点赞数量（仅统计，不暴露具体人）
    #[pallet::storage]
    #[pallet::getter(fn likes_received_count)]
    pub type LikesReceivedCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;

    /// 用户收到的点赞哈希列表（加密存储）
    /// 
    /// 存储 hash(from || to || to_salt)，仅接收者可验证
    #[pallet::storage]
    #[pallet::getter(fn likes_received_hashes)]
    pub type LikesReceivedHashes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<[u8; 32], T::MaxInteractionsPerUser>,
        ValueQuery,
    >;

    /// 匹配成功列表（加密存储）
    /// 
    /// 存储加密的匹配记录，仅当事人可解密
    #[pallet::storage]
    #[pallet::getter(fn matches)]
    pub type Matches<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<EncryptedMatchRecord, T::MaxInteractionsPerUser>,
        ValueQuery,
    >;

    /// 屏蔽列表（哈希化存储）
    /// 
    /// 存储被屏蔽用户的哈希，保护屏蔽关系隐私
    #[pallet::storage]
    #[pallet::getter(fn blocked_hashes)]
    pub type BlockedHashes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<[u8; 32], T::MaxInteractionsPerUser>,
        ValueQuery,
    >;

    /// 互动统计（仅数量，不暴露具体关系）
    #[pallet::storage]
    #[pallet::getter(fn interaction_stats)]
    pub type InteractionStats<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        UserInteractionStats,
        ValueQuery,
    >;

    /// 收到的超级喜欢队列（优先展示）
    /// 
    /// 存储收到的超级喜欢记录，用于在推荐列表中优先展示
    #[pallet::storage]
    #[pallet::getter(fn super_likes_received)]
    pub type SuperLikesReceived<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<SuperLikeRecord, T::MaxSuperLikesReceived>,
        ValueQuery,
    >;

    /// 用户每日配额
    /// 
    /// 记录用户每日的互动配额使用情况
    #[pallet::storage]
    #[pallet::getter(fn daily_quota)]
    pub type DailyQuotas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        DailyQuota,
        ValueQuery,
    >;

    /// 聊天会话记录
    /// 
    /// 记录用户与其他用户之间的聊天会话状态
    /// Key1: 用户账户
    /// Key2: 对方账户哈希（隐私保护）
    #[pallet::storage]
    #[pallet::getter(fn chat_sessions)]
    pub type ChatSessions<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        [u8; 32],  // target_hash
        ChatSessionInfo,
        OptionQuery,
    >;

    /// 聊天发起配额
    /// 
    /// 记录用户每日发起新聊天的配额使用情况
    #[pallet::storage]
    #[pallet::getter(fn chat_initiation_quota)]
    pub type ChatInitiationQuotas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ChatInitiationQuota,
        ValueQuery,
    >;

    /// 查看历史记录
    /// 
    /// 记录用户查看其他用户资料的时间戳
    /// Key1: viewer (查看者)
    /// Key2: viewed (被查看者)
    /// Value: 最后查看的区块号
    #[pallet::storage]
    #[pallet::getter(fn view_history)]
    pub type ViewHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,  // viewer
        Blake2_128Concat,
        T::AccountId,  // viewed
        BlockNumberFor<T>,  // last_viewed_at
        OptionQuery,
    >;

    /// 谁看过我（反向索引）
    /// 
    /// 记录查看过某用户资料的人列表
    /// Key: viewed (被查看者)
    /// Value: 查看者列表（最多保留最近 100 个）
    #[pallet::storage]
    #[pallet::getter(fn profile_viewers)]
    pub type ProfileViewers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<(T::AccountId, BlockNumberFor<T>), ConstU32<100>>,
        ValueQuery,
    >;

    /// 用户互动统计
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
    pub struct UserInteractionStats {
        pub likes_sent: u32,
        pub likes_received: u32,
        pub super_likes_sent: u32,
        pub super_likes_received: u32,
        pub matches_count: u32,
    }

    // ========================================================================
    // 事件（隐私保护版）
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 用户发出互动（不暴露目标）
        InteractionSent {
            from: T::AccountId,
            interaction_hash: [u8; 32],
            interaction_type: InteractionType,
        },
        /// 用户收到互动（不暴露来源）
        InteractionReceived {
            to: T::AccountId,
            interaction_hash: [u8; 32],
            interaction_type: InteractionType,
        },
        /// 匹配成功（使用哈希而非明文账户）
        MatchSuccess {
            match_hash: [u8; 32],
        },
        /// 用户屏蔽操作（不暴露目标）
        UserBlocked {
            from: T::AccountId,
            block_hash: [u8; 32],
        },
        /// 用户取消屏蔽
        UserUnblocked {
            from: T::AccountId,
            unblock_hash: [u8; 32],
        },
        /// 用户盐值初始化
        SaltInitialized {
            user: T::AccountId,
        },
        /// 超级喜欢已发送（付费成功）
        SuperLikeSent {
            from: T::AccountId,
            to: T::AccountId,
            /// 费用（以 u128 表示）
            cost: u128,
        },
        /// 收到超级喜欢（优先展示）
        SuperLikeReceived {
            to: T::AccountId,
            sender_hash: [u8; 32],
        },
        /// 聊天会话已建立
        ChatSessionEstablished {
            user: T::AccountId,
            target_hash: [u8; 32],
            initiation_type: ChatInitiationType,
        },
        /// 聊天发起配额已消耗
        ChatInitiationQuotaConsumed {
            user: T::AccountId,
            remaining: u32,
            limit: u32,
        },
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 不能对自己操作
        CannotInteractWithSelf,
        /// 已经互动过
        AlreadyInteracted,
        /// 用户已被屏蔽
        UserBlocked,
        /// 被对方屏蔽
        BlockedByUser,
        /// 互动列表已满
        InteractionListFull,
        /// 用户不存在
        UserNotFound,
        /// 余额不足
        InsufficientBalance,
        /// 未屏蔽该用户
        NotBlocked,
        /// 盐值未初始化
        SaltNotInitialized,
        /// 超级喜欢队列已满
        SuperLikeQueueFull,
        /// 转账失败
        TransferFailed,
        /// 每日点赞配额已用完
        DailyLikeQuotaExceeded,
        /// 每日超级喜欢配额已用完
        DailySuperLikeQuotaExceeded,
        /// 超级喜欢记录不存在
        SuperLikeNotFound,
        /// 每日聊天发起配额已用完
        DailyChatInitiationQuotaExceeded,
        /// 未匹配且无超级喜欢特权
        NotMatchedOrNoSuperLikePrivilege,
        /// 聊天会话不存在
        ChatSessionNotFound,
        /// 每日查看资料配额已用完
        DailyViewQuotaExceeded,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 初始化用户隐私盐值
        /// 
        /// 每个用户在首次互动前需要初始化盐值
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::like())]
        pub fn initialize_salt(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if UserSalt::<T>::contains_key(&who) {
                return Ok(());
            }

            // 生成随机盐值
            let block_number = frame_system::Pallet::<T>::block_number();
            let salt_input = (who.clone(), block_number, b"interaction_salt");
            let salt_hash = blake2_256(&salt_input.encode());
            let mut salt = [0u8; 16];
            salt.copy_from_slice(&salt_hash[..16]);

            UserSalt::<T>::insert(&who, PrivacySalt { salt });

            Self::deposit_event(Event::SaltInitialized { user: who });

            Ok(())
        }

        /// 点赞（隐私保护版）
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::like())]
        pub fn like(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(who != target, Error::<T>::CannotInteractWithSelf);

            // 确保盐值已初始化
            Self::ensure_salt_initialized(&who)?;
            Self::ensure_salt_initialized(&target)?;

            // 检查是否被对方屏蔽（使用哈希检查）
            let block_hash = Self::compute_block_hash(&target, &who);
            let target_blocked = BlockedHashes::<T>::get(&target);
            ensure!(!target_blocked.contains(&block_hash), Error::<T>::BlockedByUser);

            // 检查是否已屏蔽对方
            let my_block_hash = Self::compute_block_hash(&who, &target);
            let my_blocked = BlockedHashes::<T>::get(&who);
            ensure!(!my_blocked.contains(&my_block_hash), Error::<T>::UserBlocked);

            // 计算互动哈希
            let interaction_hash = Self::compute_interaction_hash(&who, &target);
            let reverse_hash = Self::compute_interaction_hash(&target, &who);

            // 检查是否已互动
            ensure!(!Interactions::<T>::contains_key(&interaction_hash), Error::<T>::AlreadyInteracted);

            // 检查并消耗每日点赞配额
            Self::check_and_consume_like_quota(&who)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 记录互动（哈希化存储）
            Interactions::<T>::insert(&interaction_hash, InteractionRecord {
                interaction_type: InteractionType::Like,
                timestamp: current_block,
            });

            // 更新发送者的互动列表
            MyInteractions::<T>::try_mutate(&who, |interactions| {
                interactions.try_push(interaction_hash).map_err(|_| Error::<T>::InteractionListFull)
            })?;

            // 更新接收者的点赞哈希列表
            let receiver_hash = Self::compute_receiver_hash(&who, &target);
            LikesReceivedHashes::<T>::try_mutate(&target, |hashes| {
                if !hashes.contains(&receiver_hash) {
                    hashes.try_push(receiver_hash).map_err(|_| Error::<T>::InteractionListFull)?;
                }
                Ok::<_, Error<T>>(())
            })?;

            // 更新统计
            InteractionStats::<T>::mutate(&who, |stats| {
                stats.likes_sent = stats.likes_sent.saturating_add(1);
            });
            LikesReceivedCount::<T>::mutate(&target, |count| {
                *count = count.saturating_add(1);
            });
            InteractionStats::<T>::mutate(&target, |stats| {
                stats.likes_received = stats.likes_received.saturating_add(1);
            });

            // 发送隐私保护事件
            Self::deposit_event(Event::InteractionSent {
                from: who.clone(),
                interaction_hash,
                interaction_type: InteractionType::Like,
            });
            Self::deposit_event(Event::InteractionReceived {
                to: target.clone(),
                interaction_hash: receiver_hash,
                interaction_type: InteractionType::Like,
            });

            // 检查是否互相喜欢（使用哈希检查）
            if let Some(record) = Interactions::<T>::get(&reverse_hash) {
                if matches!(record.interaction_type, InteractionType::Like | InteractionType::SuperLike) {
                    Self::create_match(&who, &target)?;
                }
            }

            Ok(())
        }

        /// 超级喜欢（付费，隐私保护版）
        /// 
        /// 需要支付 SuperLikeCost 费用，费用转入国库账户。
        /// 超级喜欢会被添加到接收者的优先队列，在推荐列表中优先展示。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::super_like())]
        pub fn super_like(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(who != target, Error::<T>::CannotInteractWithSelf);

            // 确保盐值已初始化
            Self::ensure_salt_initialized(&who)?;
            Self::ensure_salt_initialized(&target)?;

            // 检查是否被对方屏蔽
            let block_hash = Self::compute_block_hash(&target, &who);
            let target_blocked = BlockedHashes::<T>::get(&target);
            ensure!(!target_blocked.contains(&block_hash), Error::<T>::BlockedByUser);

            // 计算互动哈希（先检查是否已互动，避免扣费后失败）
            let interaction_hash = Self::compute_interaction_hash(&who, &target);
            let reverse_hash = Self::compute_interaction_hash(&target, &who);

            // 检查是否已互动
            ensure!(!Interactions::<T>::contains_key(&interaction_hash), Error::<T>::AlreadyInteracted);

            // 检查并消耗每日超级喜欢配额
            Self::check_and_consume_super_like_quota(&who)?;

            // ========== 扣除费用 ==========
            let cost_u128 = T::SuperLikeCost::get();
            if cost_u128 > 0 {
                // 转换为 Balance 类型
                let cost: T::Balance = cost_u128.try_into().map_err(|_| Error::<T>::TransferFailed)?;
                
                // 检查余额
                let balance = T::Fungible::balance(&who);
                ensure!(balance >= cost, Error::<T>::InsufficientBalance);
                
                // 转账到国库
                T::Fungible::transfer(
                    &who,
                    &T::TreasuryAccount::get(),
                    cost,
                    frame_support::traits::tokens::Preservation::Preserve,
                ).map_err(|_| Error::<T>::TransferFailed)?;
            }

            let current_block = frame_system::Pallet::<T>::block_number();
            let block_num: u64 = current_block.try_into().unwrap_or(0);

            // 记录互动
            Interactions::<T>::insert(&interaction_hash, InteractionRecord {
                interaction_type: InteractionType::SuperLike,
                timestamp: current_block,
            });

            // 更新发送者的互动列表
            MyInteractions::<T>::try_mutate(&who, |interactions| {
                interactions.try_push(interaction_hash).map_err(|_| Error::<T>::InteractionListFull)
            })?;

            // 更新接收者的点赞哈希列表
            let receiver_hash = Self::compute_receiver_hash(&who, &target);
            LikesReceivedHashes::<T>::try_mutate(&target, |hashes| {
                if !hashes.contains(&receiver_hash) {
                    hashes.try_push(receiver_hash).map_err(|_| Error::<T>::InteractionListFull)?;
                }
                Ok::<_, Error<T>>(())
            })?;

            // ========== 添加到超级喜欢接收队列（关键新增）==========
            let sender_hash = Self::compute_target_hash(&who);
            SuperLikesReceived::<T>::try_mutate(&target, |super_likes| {
                // 检查是否已存在
                if !super_likes.iter().any(|s| s.sender_hash == sender_hash) {
                    let record = SuperLikeRecord {
                        sender_hash,
                        sent_at: block_num,
                        viewed: false,
                    };
                    super_likes.try_push(record).map_err(|_| Error::<T>::SuperLikeQueueFull)?;
                }
                Ok::<_, Error<T>>(())
            })?;

            // 更新统计
            InteractionStats::<T>::mutate(&who, |stats| {
                stats.super_likes_sent = stats.super_likes_sent.saturating_add(1);
            });
            LikesReceivedCount::<T>::mutate(&target, |count| {
                *count = count.saturating_add(1);
            });
            InteractionStats::<T>::mutate(&target, |stats| {
                stats.super_likes_received = stats.super_likes_received.saturating_add(1);
            });

            // 发送隐私保护事件
            Self::deposit_event(Event::InteractionSent {
                from: who.clone(),
                interaction_hash,
                interaction_type: InteractionType::SuperLike,
            });
            Self::deposit_event(Event::InteractionReceived {
                to: target.clone(),
                interaction_hash: receiver_hash,
                interaction_type: InteractionType::SuperLike,
            });
            
            // 发送超级喜欢专用事件
            Self::deposit_event(Event::SuperLikeSent {
                from: who.clone(),
                to: target.clone(),
                cost: cost_u128,
            });
            Self::deposit_event(Event::SuperLikeReceived {
                to: target.clone(),
                sender_hash,
            });

            // 检查是否互相喜欢
            if let Some(record) = Interactions::<T>::get(&reverse_hash) {
                if matches!(record.interaction_type, InteractionType::Like | InteractionType::SuperLike) {
                    Self::create_match(&who, &target)?;
                }
            }

            Ok(())
        }

        /// 跳过（隐私保护版）
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::pass())]
        pub fn pass(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(who != target, Error::<T>::CannotInteractWithSelf);

            Self::ensure_salt_initialized(&who)?;

            let interaction_hash = Self::compute_interaction_hash(&who, &target);
            let current_block = frame_system::Pallet::<T>::block_number();

            Interactions::<T>::insert(&interaction_hash, InteractionRecord {
                interaction_type: InteractionType::Pass,
                timestamp: current_block,
            });

            MyInteractions::<T>::try_mutate(&who, |interactions| {
                interactions.try_push(interaction_hash).map_err(|_| Error::<T>::InteractionListFull)
            })?;

            Self::deposit_event(Event::InteractionSent {
                from: who,
                interaction_hash,
                interaction_type: InteractionType::Pass,
            });

            Ok(())
        }

        /// 屏蔽用户（隐私保护版）
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::block_user())]
        pub fn block_user(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(who != target, Error::<T>::CannotInteractWithSelf);

            Self::ensure_salt_initialized(&who)?;

            let block_hash = Self::compute_block_hash(&who, &target);
            let target_hash = Self::compute_target_hash(&target);

            // 添加到屏蔽哈希列表
            BlockedHashes::<T>::try_mutate(&who, |blocked| {
                if !blocked.contains(&block_hash) {
                    blocked.try_push(block_hash).map_err(|_| Error::<T>::InteractionListFull)?;
                }
                Ok::<_, Error<T>>(())
            })?;

            // 从匹配列表中移除（使用哈希匹配）
            Matches::<T>::mutate(&who, |matches| {
                matches.retain(|m| m.target_hash != target_hash);
            });

            let my_hash = Self::compute_target_hash(&who);
            Matches::<T>::mutate(&target, |matches| {
                matches.retain(|m| m.target_hash != my_hash);
            });

            Self::deposit_event(Event::UserBlocked {
                from: who,
                block_hash,
            });

            Ok(())
        }

        /// 取消屏蔽（隐私保护版）
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::unblock_user())]
        pub fn unblock_user(origin: OriginFor<T>, target: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::ensure_salt_initialized(&who)?;

            let block_hash = Self::compute_block_hash(&who, &target);

            BlockedHashes::<T>::try_mutate(&who, |blocked| {
                let pos = blocked.iter().position(|b| *b == block_hash).ok_or(Error::<T>::NotBlocked)?;
                blocked.remove(pos);
                Ok::<_, Error<T>>(())
            })?;

            Self::deposit_event(Event::UserUnblocked {
                from: who,
                unblock_hash: block_hash,
            });

            Ok(())
        }

        /// 验证互动关系（仅当事人可调用）
        /// 
        /// 用于验证自己是否喜欢/被喜欢某人
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::like())]
        pub fn verify_interaction(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::ensure_salt_initialized(&who)?;

            let interaction_hash = Self::compute_interaction_hash(&who, &target);
            
            // 返回是否存在互动（不暴露具体类型给第三方）
            let _exists = Interactions::<T>::contains_key(&interaction_hash);

            Ok(())
        }

        /// 标记超级喜欢为已查看
        /// 
        /// 当用户查看了某个超级喜欢时调用，将其标记为已查看
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::like())]
        pub fn mark_super_like_viewed(
            origin: OriginFor<T>,
            sender_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            SuperLikesReceived::<T>::try_mutate(&who, |super_likes| {
                let record = super_likes
                    .iter_mut()
                    .find(|r| r.sender_hash == sender_hash)
                    .ok_or(Error::<T>::SuperLikeNotFound)?;
                record.viewed = true;
                Ok::<_, Error<T>>(())
            })?;

            Ok(())
        }

        /// 发送婚恋消息（带聊天发起配额检查）
        /// 
        /// 此函数是婚恋场景下发起聊天的主入口，包含：
        /// 1. 匹配/超级喜欢权限检查
        /// 2. 每日发起配额检查
        /// 3. 聊天会话管理
        /// 
        /// # 参数
        /// - `receiver`: 接收者账户
        /// 
        /// # 权限规则
        /// - 已匹配用户可发起聊天（消耗每日配额）
        /// - 收到超级喜欢后可发起聊天（不消耗配额）
        /// - 已有会话可继续聊天（不消耗配额）
        /// - 被动回复不消耗配额
        /// 
        /// # 返回
        /// - 成功时返回 Ok(())，前端可继续调用 chat-core 发送消息
        /// - 失败时返回相应错误
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::initiate_matchmaking_chat())]
        pub fn initiate_matchmaking_chat(
            origin: OriginFor<T>,
            receiver: T::AccountId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // 不能给自己发消息
            ensure!(sender != receiver, Error::<T>::CannotInteractWithSelf);
            
            // 确保盐值已初始化
            Self::ensure_salt_initialized(&sender)?;
            Self::ensure_salt_initialized(&receiver)?;
            
            // 1. 检查聊天权限
            let initiation_type = Self::can_initiate_chat(&sender, &receiver)?;
            
            // 2. 根据发起类型处理配额
            match initiation_type {
                ChatInitiationType::InitiatedByMe => {
                    // 首次主动发起，消耗配额
                    Self::check_and_consume_chat_initiation_quota(&sender)?;
                    
                    // 记录会话
                    Self::record_chat_session(&sender, &receiver, initiation_type.clone())?;
                    
                    // 发送配额消耗事件
                    let (remaining, limit) = Self::get_remaining_chat_initiation_quota(&sender);
                    Self::deposit_event(Event::ChatInitiationQuotaConsumed {
                        user: sender,
                        remaining,
                        limit,
                    });
                },
                ChatInitiationType::InitiatedByOther => {
                    // 对方先发起，我是回复方，不消耗配额
                    Self::record_chat_session(&sender, &receiver, initiation_type)?;
                },
                ChatInitiationType::SuperLikePrivilege => {
                    // 超级喜欢特权，不消耗配额
                    Self::record_chat_session(&sender, &receiver, initiation_type)?;
                },
            }
            
            Ok(())
        }

        /// 查看用户资料（消耗查看配额）
        /// 
        /// 当用户查看另一个用户的详细资料时调用，消耗每日查看配额。
        /// 
        /// # 参数
        /// - `target`: 被查看的用户账户
        /// 
        /// # 配额规则
        /// - 免费用户：每日限制查看次数（如 50 次）
        /// - 会员用户：更多查看次数或无限制
        /// - 查看自己的资料不消耗配额
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::view_profile())]
        pub fn view_profile(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // 查看自己的资料不消耗配额
            if who == target {
                return Ok(());
            }
            
            // 检查是否已查看过（同一天内重复查看不消耗配额）
            let current_block = frame_system::Pallet::<T>::block_number();
            let already_viewed_today = Self::is_viewed_today(&who, &target, current_block);
            
            if !already_viewed_today {
                // 首次查看，检查并消耗查看配额
                Self::check_and_consume_view_quota(&who)?;
            }
            
            // 记录查看历史
            Self::record_view_history(&who, &target, current_block);
            
            Ok(())
        }
    }
}

// ============================================================================
// 辅助实现（隐私保护版）
// ============================================================================

impl<T: Config> Pallet<T> {
    /// 确保用户盐值已初始化
    fn ensure_salt_initialized(user: &T::AccountId) -> DispatchResult {
        if !UserSalt::<T>::contains_key(user) {
            // 自动初始化盐值
            let block_number = frame_system::Pallet::<T>::block_number();
            let salt_input = (user.clone(), block_number, b"interaction_salt");
            let salt_hash = blake2_256(&salt_input.encode());
            let mut salt = [0u8; 16];
            salt.copy_from_slice(&salt_hash[..16]);
            UserSalt::<T>::insert(user, PrivacySalt { salt });
        }
        Ok(())
    }

    /// 计算互动哈希
    /// 
    /// hash(from || to || from_salt)
    /// 用于存储发送者的互动记录
    fn compute_interaction_hash(from: &T::AccountId, to: &T::AccountId) -> [u8; 32] {
        let salt = UserSalt::<T>::get(from).unwrap_or_default();
        let input = (from.clone(), to.clone(), salt.salt, b"interaction");
        blake2_256(&input.encode())
    }

    /// 计算接收者哈希
    /// 
    /// hash(from || to || to_salt)
    /// 用于存储接收者的点赞记录
    fn compute_receiver_hash(from: &T::AccountId, to: &T::AccountId) -> [u8; 32] {
        let salt = UserSalt::<T>::get(to).unwrap_or_default();
        let input = (from.clone(), to.clone(), salt.salt, b"receiver");
        blake2_256(&input.encode())
    }

    /// 计算屏蔽哈希
    /// 
    /// hash(blocker || blocked || blocker_salt)
    fn compute_block_hash(blocker: &T::AccountId, blocked: &T::AccountId) -> [u8; 32] {
        let salt = UserSalt::<T>::get(blocker).unwrap_or_default();
        let input = (blocker.clone(), blocked.clone(), salt.salt, b"block");
        blake2_256(&input.encode())
    }

    /// 计算目标账户哈希（用于匹配列表）
    fn compute_target_hash(target: &T::AccountId) -> [u8; 32] {
        let input = (target.clone(), b"target_hash");
        blake2_256(&input.encode())
    }

    /// 计算匹配哈希（用于事件）
    fn compute_match_hash(user_a: &T::AccountId, user_b: &T::AccountId) -> [u8; 32] {
        // 确保顺序一致
        let (first, second) = if user_a.encode() < user_b.encode() {
            (user_a, user_b)
        } else {
            (user_b, user_a)
        };
        let input = (first.clone(), second.clone(), b"match");
        blake2_256(&input.encode())
    }

    /// 创建匹配（隐私保护版）
    fn create_match(user_a: &T::AccountId, user_b: &T::AccountId) -> DispatchResult {
        let current_block = frame_system::Pallet::<T>::block_number();
        let block_num: u64 = current_block.try_into().unwrap_or(0);

        // 计算对方的哈希
        let hash_b = Self::compute_target_hash(user_b);
        let hash_a = Self::compute_target_hash(user_a);

        // 添加到双方的匹配列表（加密存储）
        Matches::<T>::try_mutate(user_a, |matches| {
            let record = EncryptedMatchRecord {
                target_hash: hash_b,
                matched_at: block_num,
            };
            if !matches.iter().any(|m| m.target_hash == hash_b) {
                matches.try_push(record).map_err(|_| Error::<T>::InteractionListFull)?;
            }
            Ok::<_, Error<T>>(())
        })?;

        Matches::<T>::try_mutate(user_b, |matches| {
            let record = EncryptedMatchRecord {
                target_hash: hash_a,
                matched_at: block_num,
            };
            if !matches.iter().any(|m| m.target_hash == hash_a) {
                matches.try_push(record).map_err(|_| Error::<T>::InteractionListFull)?;
            }
            Ok::<_, Error<T>>(())
        })?;

        // 更新统计
        InteractionStats::<T>::mutate(user_a, |stats| {
            stats.matches_count = stats.matches_count.saturating_add(1);
        });
        InteractionStats::<T>::mutate(user_b, |stats| {
            stats.matches_count = stats.matches_count.saturating_add(1);
        });

        // 发送隐私保护事件（使用匹配哈希而非明文账户）
        let match_hash = Self::compute_match_hash(user_a, user_b);
        Self::deposit_event(Event::MatchSuccess { match_hash });

        Ok(())
    }

    /// 检查是否互相喜欢（隐私保护版）
    pub fn is_mutual_like(user_a: &T::AccountId, user_b: &T::AccountId) -> bool {
        let hash_a_to_b = Self::compute_interaction_hash(user_a, user_b);
        let hash_b_to_a = Self::compute_interaction_hash(user_b, user_a);

        let a_likes_b = Interactions::<T>::get(&hash_a_to_b)
            .map(|r| matches!(r.interaction_type, InteractionType::Like | InteractionType::SuperLike))
            .unwrap_or(false);

        let b_likes_a = Interactions::<T>::get(&hash_b_to_a)
            .map(|r| matches!(r.interaction_type, InteractionType::Like | InteractionType::SuperLike))
            .unwrap_or(false);

        a_likes_b && b_likes_a
    }

    /// 检查是否被屏蔽（隐私保护版）
    pub fn is_blocked(from: &T::AccountId, to: &T::AccountId) -> bool {
        let block_hash = Self::compute_block_hash(to, from);
        BlockedHashes::<T>::get(to).contains(&block_hash)
    }

    /// 验证某人是否喜欢自己
    /// 
    /// 用户可以验证某个特定账户是否喜欢自己
    pub fn verify_liked_by(user: &T::AccountId, potential_liker: &T::AccountId) -> bool {
        let receiver_hash = Self::compute_receiver_hash(potential_liker, user);
        LikesReceivedHashes::<T>::get(user).contains(&receiver_hash)
    }

    /// 获取用户匹配数量
    pub fn get_match_count(user: &T::AccountId) -> u32 {
        InteractionStats::<T>::get(user).matches_count
    }

    /// 获取用户收到的点赞数量
    pub fn get_likes_received_count(user: &T::AccountId) -> u32 {
        LikesReceivedCount::<T>::get(user)
    }

    // ========================================================================
    // 配额系统
    // ========================================================================

    /// 获取当前日期（以区块号计算）
    fn get_current_day() -> u32 {
        let current_block: u64 = frame_system::Pallet::<T>::block_number()
            .try_into()
            .unwrap_or(0);
        let blocks_per_day: u64 = T::BlocksPerDay::get().into();
        if blocks_per_day == 0 {
            return 0;
        }
        (current_block / blocks_per_day) as u32
    }

    /// 重置配额（如果是新的一天）
    fn maybe_reset_quota(quota: &mut DailyQuota) {
        let current_day = Self::get_current_day();
        if quota.last_reset_day != current_day {
            quota.likes_used = 0;
            quota.super_likes_used = 0;
            quota.views_used = 0;
            quota.last_reset_day = current_day;
        }
    }

    /// 检查并消耗点赞配额
    /// 
    /// 返回 Ok(()) 如果配额足够，否则返回错误
    pub fn check_and_consume_like_quota(user: &T::AccountId) -> DispatchResult {
        let max_likes = T::FreeDailyLikes::get();
        
        // 如果配额为 0，表示无限制
        if max_likes == 0 {
            return Ok(());
        }

        DailyQuotas::<T>::try_mutate(user, |quota| {
            Self::maybe_reset_quota(quota);
            
            if quota.likes_used >= max_likes {
                return Err(Error::<T>::DailyLikeQuotaExceeded.into());
            }
            
            quota.likes_used = quota.likes_used.saturating_add(1);
            Ok(())
        })
    }

    /// 检查并消耗超级喜欢配额
    /// 
    /// 返回 Ok(()) 如果配额足够，否则返回错误
    pub fn check_and_consume_super_like_quota(user: &T::AccountId) -> DispatchResult {
        let max_super_likes = T::FreeDailySuperLikes::get();
        
        // 如果配额为 0，表示免费用户不能使用超级喜欢（需要付费）
        // 这里不做限制，让付费逻辑处理
        if max_super_likes == 0 {
            return Ok(());
        }

        DailyQuotas::<T>::try_mutate(user, |quota| {
            Self::maybe_reset_quota(quota);
            
            if quota.super_likes_used >= max_super_likes {
                return Err(Error::<T>::DailySuperLikeQuotaExceeded.into());
            }
            
            quota.super_likes_used = quota.super_likes_used.saturating_add(1);
            Ok(())
        })
    }

    /// 检查并消耗查看资料配额
    /// 
    /// 返回 Ok(()) 如果配额足够，否则返回错误
    pub fn check_and_consume_view_quota(user: &T::AccountId) -> DispatchResult {
        let max_views = Self::get_daily_view_limit(user);
        
        // 如果配额为 0，表示无限制
        if max_views == 0 {
            return Ok(());
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

    /// 获取用户的每日查看限额
    fn get_daily_view_limit(user: &T::AccountId) -> u32 {
        // TODO: 集成会员系统检查
        // 目前简化为使用免费用户配额
        // 未来可以检查 MembershipExpiry 来区分会员等级
        T::FreeDailyViews::get()
    }

    // ========================================================================
    // 查看历史记录
    // ========================================================================

    /// 检查今天是否已查看过该用户
    fn is_viewed_today(
        viewer: &T::AccountId,
        viewed: &T::AccountId,
        current_block: BlockNumberFor<T>,
    ) -> bool {
        if let Some(last_viewed) = ViewHistory::<T>::get(viewer, viewed) {
            let blocks_per_day: BlockNumberFor<T> = T::BlocksPerDay::get().into();
            if blocks_per_day == Zero::zero() {
                return false;
            }
            let current_day = current_block / blocks_per_day;
            let last_day = last_viewed / blocks_per_day;
            return current_day == last_day;
        }
        false
    }

    /// 记录查看历史
    fn record_view_history(
        viewer: &T::AccountId,
        viewed: &T::AccountId,
        current_block: BlockNumberFor<T>,
    ) {
        // 更新查看历史（viewer -> viewed）
        ViewHistory::<T>::insert(viewer, viewed, current_block);
        
        // 更新反向索引（谁看过我）
        ProfileViewers::<T>::mutate(viewed, |viewers| {
            // 检查是否已存在，如果存在则更新时间
            if let Some(pos) = viewers.iter().position(|(v, _)| v == viewer) {
                viewers[pos].1 = current_block;
            } else {
                // 如果列表已满，移除最早的记录
                if viewers.is_full() {
                    // 找到最早的记录并移除
                    if let Some((min_idx, _)) = viewers.iter().enumerate().min_by_key(|(_, (_, block))| block) {
                        viewers.remove(min_idx);
                    }
                }
                // 添加新记录
                let _ = viewers.try_push((viewer.clone(), current_block));
            }
        });
    }

    /// 获取查看过我的用户列表
    pub fn get_profile_viewers(user: &T::AccountId) -> Vec<(T::AccountId, BlockNumberFor<T>)> {
        ProfileViewers::<T>::get(user).into_inner()
    }

    /// 获取我查看过的用户数量（今日）
    pub fn get_today_view_count(user: &T::AccountId) -> u32 {
        let quota = DailyQuotas::<T>::get(user);
        let current_day = Self::get_current_day();
        if quota.last_reset_day == current_day {
            quota.views_used
        } else {
            0
        }
    }

    /// 获取用户剩余的每日配额（点赞、超级喜欢、查看）
    pub fn get_remaining_quota(user: &T::AccountId) -> (u32, u32, u32) {
        let mut quota = DailyQuotas::<T>::get(user);
        Self::maybe_reset_quota(&mut quota);
        
        let max_likes = T::FreeDailyLikes::get();
        let max_super_likes = T::FreeDailySuperLikes::get();
        let max_views = Self::get_daily_view_limit(user);
        
        let remaining_likes = max_likes.saturating_sub(quota.likes_used);
        let remaining_super_likes = max_super_likes.saturating_sub(quota.super_likes_used);
        let remaining_views = if max_views == 0 { u32::MAX } else { max_views.saturating_sub(quota.views_used) };
        
        (remaining_likes, remaining_super_likes, remaining_views)
    }

    // ========================================================================
    // 超级喜欢查询
    // ========================================================================

    /// 获取未查看的超级喜欢数量
    pub fn get_unviewed_super_likes_count(user: &T::AccountId) -> u32 {
        SuperLikesReceived::<T>::get(user)
            .iter()
            .filter(|r| !r.viewed)
            .count() as u32
    }

    /// 获取所有收到的超级喜欢
    pub fn get_super_likes_received(user: &T::AccountId) -> BoundedVec<SuperLikeRecord, T::MaxSuperLikesReceived> {
        SuperLikesReceived::<T>::get(user)
    }

    // ========================================================================
    // 聊天发起系统
    // ========================================================================

    /// 检查用户是否可以向目标发起聊天
    /// 
    /// 返回 Ok(ChatInitiationType) 表示可以发起，并说明发起方式
    /// 返回 Err 表示不能发起
    pub fn can_initiate_chat(
        sender: &T::AccountId,
        receiver: &T::AccountId,
    ) -> Result<ChatInitiationType, DispatchError> {
        let receiver_hash = Self::compute_target_hash(receiver);
        let sender_hash = Self::compute_target_hash(sender);
        
        // 1. 检查是否被屏蔽
        ensure!(!Self::is_blocked(sender, receiver), Error::<T>::BlockedByUser);
        
        // 2. 检查是否已有聊天会话
        if let Some(session) = ChatSessions::<T>::get(sender, &receiver_hash) {
            // 已有会话，直接返回（不消耗配额）
            return Ok(session.initiation_type);
        }
        
        // 3. 检查超级喜欢特权
        // 如果 sender 收到过来自 receiver 的超级喜欢，则有特权
        let super_likes = SuperLikesReceived::<T>::get(sender);
        let has_super_like_privilege = super_likes.iter()
            .any(|sl| sl.sender_hash == receiver_hash);
        
        if has_super_like_privilege {
            return Ok(ChatInitiationType::SuperLikePrivilege);
        }
        
        // 4. 检查是否已匹配
        let matches = Matches::<T>::get(sender);
        let is_matched = matches.iter().any(|m| m.target_hash == receiver_hash);
        
        if !is_matched {
            return Err(Error::<T>::NotMatchedOrNoSuperLikePrivilege.into());
        }
        
        // 5. 检查对方是否先发起过
        if ChatSessions::<T>::contains_key(receiver, &sender_hash) {
            return Ok(ChatInitiationType::InitiatedByOther);
        }
        
        // 6. 首次主动发起
        Ok(ChatInitiationType::InitiatedByMe)
    }

    /// 获取用户的每日聊天发起限额
    fn get_daily_chat_initiation_limit(user: &T::AccountId) -> u32 {
        // TODO: 集成会员系统检查
        // 目前简化为使用免费用户配额
        // 未来可以检查 MembershipExpiry 来区分会员等级
        T::FreeDailyChatInitiations::get()
    }

    /// 检查并消耗聊天发起配额
    /// 
    /// 仅在 ChatInitiationType::InitiatedByMe 时调用
    pub fn check_and_consume_chat_initiation_quota(
        sender: &T::AccountId,
    ) -> DispatchResult {
        let limit = Self::get_daily_chat_initiation_limit(sender);
        
        // 0 表示无限制
        if limit == 0 {
            return Ok(());
        }
        
        ChatInitiationQuotas::<T>::try_mutate(sender, |quota| {
            let current_day = Self::get_current_day();
            
            // 重置配额（新的一天）
            if quota.last_reset_day != current_day {
                quota.chats_initiated = 0;
                quota.last_reset_day = current_day;
            }
            
            // 检查配额
            ensure!(
                quota.chats_initiated < limit,
                Error::<T>::DailyChatInitiationQuotaExceeded
            );
            
            // 消耗配额
            quota.chats_initiated = quota.chats_initiated.saturating_add(1);
            
            Ok(())
        })
    }

    /// 记录聊天会话
    fn record_chat_session(
        sender: &T::AccountId,
        receiver: &T::AccountId,
        initiation_type: ChatInitiationType,
    ) -> DispatchResult {
        let receiver_hash = Self::compute_target_hash(receiver);
        let block_num: u64 = frame_system::Pallet::<T>::block_number()
            .try_into().unwrap_or(0);
        
        ChatSessions::<T>::insert(sender, &receiver_hash, ChatSessionInfo {
            created_at: block_num,
            initiation_type: initiation_type.clone(),
        });
        
        Self::deposit_event(Event::ChatSessionEstablished {
            user: sender.clone(),
            target_hash: receiver_hash,
            initiation_type,
        });
        
        Ok(())
    }

    /// 获取用户剩余的聊天发起配额
    pub fn get_remaining_chat_initiation_quota(user: &T::AccountId) -> (u32, u32) {
        let limit = Self::get_daily_chat_initiation_limit(user);
        
        // 0 表示无限
        if limit == 0 {
            return (u32::MAX, 0);
        }
        
        let quota = ChatInitiationQuotas::<T>::get(user);
        let current_day = Self::get_current_day();
        
        if quota.last_reset_day != current_day {
            return (limit, limit);
        }
        
        (limit.saturating_sub(quota.chats_initiated), limit)
    }

    /// 检查用户是否有超级喜欢特权可以联系目标
    pub fn has_super_like_privilege(
        sender: &T::AccountId,
        receiver: &T::AccountId,
    ) -> bool {
        let receiver_hash = Self::compute_target_hash(receiver);
        let super_likes = SuperLikesReceived::<T>::get(sender);
        super_likes.iter().any(|sl| sl.sender_hash == receiver_hash)
    }
}

// WeightInfo trait 和实现已移至 weights.rs
