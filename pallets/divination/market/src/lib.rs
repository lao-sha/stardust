//! # 通用玄学占卜服务市场 Pallet
//!
//! 本模块实现了去中心化的占卜服务交易市场，支持多种玄学系统：
//! - 梅花易数
//! - 八字命理
//! - 六爻占卜
//! - 奇门遁甲
//! - 紫微斗数
//!
//! ## 核心功能
//!
//! 1. **服务提供者**: 注册、认证、等级晋升
//! 2. **服务套餐**: 文字/语音/视频/实时多种形式
//! 3. **订单系统**: 下单、支付、解读、评价完整流程
//! 4. **信誉机制**: 多维度评分、等级制度
//! 5. **收益管理**: 平台抽成、提现申请
//!
//! ## 架构说明
//!
//! 本模块通过 `DivinationProvider` trait 与各玄学核心模块解耦：
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                pallet-divination-market                 │
//! │    (通用服务市场、订单管理、评价系统)                      │
//! └──────────────────────────┬──────────────────────────────┘
//!                            │ DivinationProvider trait
//!                            ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │              Runtime: CombinedDivinationProvider        │
//! └───────┬─────────────────────────────────────┬───────────┘
//!         │                                     │
//!         ▼                                     ▼
//! ┌───────────────┐                     ┌───────────────┐
//! │ pallet-meihua │                     │ pallet-bazi   │
//! └───────────────┘                     └───────────────┘
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod types;

mod helpers;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::types::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use pallet_divination_common::{DivinationProvider, DivinationType};
    use pallet_affiliate::types::AffiliateDistributor;
    use pallet_trading_common::PricingProvider;
    use pallet_chat_permission::{SceneAuthorizationManager, SceneType, SceneId};
    use sp_runtime::traits::{Saturating, Zero, SaturatedConversion};
    // 已移除 L1/L2 归档压缩，不再需要 amount_to_tier 和 block_to_year_month
    use sp_std::prelude::*;

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 占卜结果查询接口
        type DivinationProvider: DivinationProvider<Self::AccountId>;

        /// IPFS 内容注册接口（用于自动 Pin 市场内容）
        type ContentRegistry: pallet_storage_service::ContentRegistry;

        /// 最小保证金（DUST数量）
        #[pallet::constant]
        type MinDeposit: Get<BalanceOf<Self>>;

        /// 最小保证金USD价值（精度10^6，100_000_000 = 100 USDT）
        #[pallet::constant]
        type MinDepositUsd: Get<u64>;

        /// 定价接口（用于换算保证金USD价值）
        type Pricing: pallet_trading_common::PricingProvider<BalanceOf<Self>>;

        /// 最小服务价格
        #[pallet::constant]
        type MinServicePrice: Get<BalanceOf<Self>>;

        /// 最大服务价格（修复 H-13: 防止异常高价）
        #[pallet::constant]
        type MaxServicePrice: Get<BalanceOf<Self>>;

        /// 订单超时时间（区块数）
        #[pallet::constant]
        type OrderTimeout: Get<BlockNumberFor<Self>>;

        /// 接单超时时间（区块数）
        #[pallet::constant]
        type AcceptTimeout: Get<BlockNumberFor<Self>>;

        /// 评价期限（区块数）
        #[pallet::constant]
        type ReviewPeriod: Get<BlockNumberFor<Self>>;

        /// 提现冷却期（区块数）
        #[pallet::constant]
        type WithdrawalCooldown: Get<BlockNumberFor<Self>>;

        /// 最大名称长度
        #[pallet::constant]
        type MaxNameLength: Get<u32>;

        /// 最大简介长度
        #[pallet::constant]
        type MaxBioLength: Get<u32>;

        /// 最大描述长度
        #[pallet::constant]
        type MaxDescriptionLength: Get<u32>;

        /// 最大 CID 长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 每个提供者最大套餐数
        #[pallet::constant]
        type MaxPackagesPerProvider: Get<u32>;

        /// 每个订单最大追问数
        #[pallet::constant]
        type MaxFollowUpsPerOrder: Get<u32>;

        /// 平台收款账户
        #[pallet::constant]
        type PlatformAccount: Get<Self::AccountId>;

        /// 治理权限来源
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 国库账户
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// 🆕 联盟分成接口
        type AffiliateDistributor: pallet_affiliate::types::AffiliateDistributor<
            Self::AccountId,
            u128,
            BlockNumberFor<Self>,
        >;


        /// 🆕 解读修改窗口（区块数，28800 ≈ 2天，按6秒/块计算）
        #[pallet::constant]
        type InterpretationEditWindow: Get<BlockNumberFor<Self>>;

        /// 🆕 聊天权限管理接口（订单创建时自动授权双方聊天）
        type ChatPermission: SceneAuthorizationManager<Self::AccountId, BlockNumberFor<Self>>;

        /// 🆕 订单聊天授权有效期（区块数，432000 ≈ 30天）
        #[pallet::constant]
        type OrderChatDuration: Get<BlockNumberFor<Self>>;
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 提供者类型别名
    pub type ProviderOf<T> = Provider<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxBioLength,
    >;

    /// 服务套餐类型别名
    pub type ServicePackageOf<T> = ServicePackage<BalanceOf<T>, <T as Config>::MaxDescriptionLength>;

    /// 订单类型别名
    pub type OrderOf<T> = Order<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 追问类型别名
    pub type FollowUpOf<T> = FollowUp<BlockNumberFor<T>, <T as Config>::MaxCidLength>;

    /// 评价类型别名
    pub type ReviewOf<T> = Review<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 悬赏问题类型别名
    pub type BountyQuestionOf<T> = BountyQuestion<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 悬赏回答类型别名
    pub type BountyAnswerOf<T> = BountyAnswer<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 投票记录类型别名
    pub type BountyVoteOf<T> = BountyVote<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
    >;

    // ==================== 个人主页类型别名 ====================

    /// 提供者详细资料类型别名
    pub type ProviderProfileOf<T> = ProviderProfile<
        BlockNumberFor<T>,
        <T as Config>::MaxDescriptionLength,
        <T as Config>::MaxCidLength,
    >;

    /// 资质证书类型别名
    pub type CertificateOf<T> = Certificate<
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 作品集类型别名
    pub type PortfolioItemOf<T> = PortfolioItem<
        BlockNumberFor<T>,
        <T as Config>::MaxNameLength,
        <T as Config>::MaxCidLength,
    >;

    /// 技能标签类型别名
    pub type SkillTagOf = SkillTag<ConstU32<32>>;

    // ==================== 信用体系类型别名 ====================

    /// 信用档案类型别名
    pub type CreditProfileOf<T> = CreditProfile<BlockNumberFor<T>>;

    /// 违规记录类型别名
    pub type ViolationRecordOf<T> = ViolationRecord<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxDescriptionLength,
    >;

    /// 信用变更记录类型别名
    pub type CreditChangeRecordOf<T> = CreditChangeRecord<
        BlockNumberFor<T>,
        ConstU32<256>,
    >;

    /// 信用修复任务类型别名
    pub type CreditRepairTaskOf<T> = CreditRepairTask<BlockNumberFor<T>>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 🆕 存储膨胀防护：Hooks ====================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 空闲时归档已完成订单和悬赏（仅移动索引，保留完整数据）
        fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = Weight::from_parts(20_000, 0);
            if remaining_weight.ref_time() < base_weight.ref_time() * 10 {
                return Weight::zero();
            }

            // 1. 归档已完成订单（保留完整订单数据）
            let w1 = Self::archive_completed_orders(5);
            
            // 2. 归档已结束悬赏（保留完整悬赏数据）
            let w2 = Self::archive_completed_bounties(5);
            
            w1.saturating_add(w2)
        }
    }

    // ==================== 存储项 ====================

    /// 下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个提现请求 ID
    #[pallet::storage]
    #[pallet::getter(fn next_withdrawal_id)]
    pub type NextWithdrawalId<T> = StorageValue<_, u64, ValueQuery>;

    /// 提供者下一个套餐 ID
    #[pallet::storage]
    #[pallet::getter(fn next_package_id)]
    pub type NextPackageId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 服务提供者存储
    #[pallet::storage]
    #[pallet::getter(fn providers)]
    pub type Providers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ProviderOf<T>>;

    /// 服务套餐存储（提供者 -> 套餐ID -> 套餐）
    #[pallet::storage]
    #[pallet::getter(fn packages)]
    pub type Packages<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        ServicePackageOf<T>,
    >;

    /// 订单存储
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, u64, OrderOf<T>>;

    /// 订单追问存储
    #[pallet::storage]
    #[pallet::getter(fn follow_ups)]
    pub type FollowUps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<FollowUpOf<T>, T::MaxFollowUpsPerOrder>,
        ValueQuery,
    >;

    /// 评价存储
    #[pallet::storage]
    #[pallet::getter(fn reviews)]
    pub type Reviews<T: Config> = StorageMap<_, Blake2_128Concat, u64, ReviewOf<T>>;

    /// 提供者收入余额
    #[pallet::storage]
    #[pallet::getter(fn provider_balances)]
    pub type ProviderBalances<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    /// 提现请求存储
    #[pallet::storage]
    #[pallet::getter(fn withdrawals)]
    pub type Withdrawals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        WithdrawalRequest<T::AccountId, BalanceOf<T>, BlockNumberFor<T>>,
    >;

    /// 客户订单索引
    /// 上限从200提升到500，配合7天归档窗口可支持每天70+订单
    #[pallet::storage]
    #[pallet::getter(fn customer_orders)]
    pub type CustomerOrders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<500>>, ValueQuery>;

    /// 提供者订单索引
    /// 上限从200提升到1000，热门提供者可能接单量更大
    #[pallet::storage]
    #[pallet::getter(fn provider_orders)]
    pub type ProviderOrders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<1000>>, ValueQuery>;

    /// 市场统计
    #[pallet::storage]
    #[pallet::getter(fn market_stats)]
    pub type MarketStatistics<T: Config> = StorageValue<_, MarketStats<BalanceOf<T>>, ValueQuery>;

    /// 🆕 累计联盟分成金额
    #[pallet::storage]
    #[pallet::getter(fn total_affiliate_distributed)]
    pub type TotalAffiliateDistributed<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // ==================== 🆕 OCW 异步解读存储 ====================

    /// 待处理解读（OCW 异步结算）
    #[pallet::storage]
    #[pallet::getter(fn pending_interpretations)]
    pub type PendingInterpretations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // order_id
        PendingInterpretation<BlockNumberFor<T>, T::MaxCidLength, ConstU32<20>>,
    >;

    /// 待处理解读队列（按提交顺序）
    #[pallet::storage]
    #[pallet::getter(fn pending_interpretation_queue)]
    pub type PendingInterpretationQueue<T: Config> = StorageValue<
        _,
        BoundedVec<u64, ConstU32<1000>>,
        ValueQuery,
    >;

    /// 按占卜类型的市场统计
    #[pallet::storage]
    #[pallet::getter(fn type_stats)]
    pub type TypeStatistics<T: Config> =
        StorageMap<_, Blake2_128Concat, DivinationType, TypeMarketStats<BalanceOf<T>>, ValueQuery>;

    // ==================== 悬赏问答存储项 ====================

    /// 下一个悬赏问题 ID
    #[pallet::storage]
    #[pallet::getter(fn next_bounty_id)]
    pub type NextBountyId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个悬赏回答 ID
    #[pallet::storage]
    #[pallet::getter(fn next_bounty_answer_id)]
    pub type NextBountyAnswerId<T> = StorageValue<_, u64, ValueQuery>;

    /// 悬赏问题存储
    #[pallet::storage]
    #[pallet::getter(fn bounty_questions)]
    pub type BountyQuestions<T: Config> = StorageMap<_, Blake2_128Concat, u64, BountyQuestionOf<T>>;

    /// 悬赏回答存储
    #[pallet::storage]
    #[pallet::getter(fn bounty_answers)]
    pub type BountyAnswers<T: Config> = StorageMap<_, Blake2_128Concat, u64, BountyAnswerOf<T>>;

    /// 悬赏问题的回答列表索引（bounty_id -> answer_ids）
    #[pallet::storage]
    #[pallet::getter(fn bounty_answer_ids)]
    pub type BountyAnswerIds<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u64, ConstU32<100>>, ValueQuery>;

    /// 用户创建的悬赏问题索引
    #[pallet::storage]
    #[pallet::getter(fn user_bounties)]
    pub type UserBounties<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<500>>, ValueQuery>;

    /// 用户提交的悬赏回答索引
    /// 上限从200提升到500，支持活跃回答者
    #[pallet::storage]
    #[pallet::getter(fn user_bounty_answers)]
    pub type UserBountyAnswers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<u64, ConstU32<500>>, ValueQuery>;

    /// 悬赏投票记录（bounty_id -> voter -> vote）
    #[pallet::storage]
    #[pallet::getter(fn bounty_votes)]
    pub type BountyVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        BountyVoteOf<T>,
    >;

    /// 悬赏问答统计
    #[pallet::storage]
    #[pallet::getter(fn bounty_stats)]
    pub type BountyStatistics<T: Config> = StorageValue<_, BountyStats<BalanceOf<T>>, ValueQuery>;

    // ==================== 个人主页存储项 ====================

    /// 提供者详细资料
    #[pallet::storage]
    #[pallet::getter(fn provider_profiles)]
    pub type ProviderProfiles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ProviderProfileOf<T>>;

    /// 提供者资质证书（提供者 -> 证书ID -> 证书）
    #[pallet::storage]
    #[pallet::getter(fn certificates)]
    pub type Certificates<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        CertificateOf<T>,
    >;

    /// 提供者下一个证书 ID
    #[pallet::storage]
    #[pallet::getter(fn next_certificate_id)]
    pub type NextCertificateId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 提供者作品集（提供者 -> 作品ID -> 作品）
    #[pallet::storage]
    #[pallet::getter(fn portfolios)]
    pub type Portfolios<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        PortfolioItemOf<T>,
    >;

    /// 提供者下一个作品 ID
    #[pallet::storage]
    #[pallet::getter(fn next_portfolio_id)]
    pub type NextPortfolioId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// 提供者技能标签
    #[pallet::storage]
    #[pallet::getter(fn skill_tags)]
    pub type SkillTags<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<SkillTagOf, ConstU32<20>>,
        ValueQuery,
    >;

    /// 提供者评价标签统计
    #[pallet::storage]
    #[pallet::getter(fn review_tag_stats)]
    pub type ReviewTagStatistics<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ReviewTagStats, ValueQuery>;

    /// 作品点赞记录（(提供者, 作品ID) -> 用户 -> 是否点赞）
    #[pallet::storage]
    #[pallet::getter(fn portfolio_likes)]
    pub type PortfolioLikes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        (T::AccountId, u32),
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    // ==================== 信用体系存储项 ====================

    /// 提供者信用档案
    #[pallet::storage]
    #[pallet::getter(fn credit_profiles)]
    pub type CreditProfiles<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, CreditProfileOf<T>>;

    /// 违规记录存储
    #[pallet::storage]
    #[pallet::getter(fn violation_records)]
    pub type ViolationRecords<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ViolationRecordOf<T>>;

    /// 提供者违规记录索引
    #[pallet::storage]
    #[pallet::getter(fn provider_violations)]
    pub type ProviderViolations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<200>>,
        ValueQuery,
    >;

    /// 下一个违规记录 ID
    #[pallet::storage]
    #[pallet::getter(fn next_violation_id)]
    pub type NextViolationId<T> = StorageValue<_, u64, ValueQuery>;

    /// 信用变更历史（最近 50 条）
    #[pallet::storage]
    #[pallet::getter(fn credit_history)]
    pub type CreditHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<CreditChangeRecordOf<T>, ConstU32<50>>,
        ValueQuery,
    >;

    /// 信用修复任务
    #[pallet::storage]
    #[pallet::getter(fn repair_tasks)]
    pub type RepairTasks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<CreditRepairTaskOf<T>, ConstU32<5>>,
        ValueQuery,
    >;

    /// 信用黑名单（永久封禁）
    #[pallet::storage]
    #[pallet::getter(fn credit_blacklist)]
    pub type CreditBlacklist<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BlockNumberFor<T>>;

    /// 全局信用统计
    #[pallet::storage]
    #[pallet::getter(fn credit_stats)]
    pub type CreditStatistics<T: Config> = StorageValue<_, GlobalCreditStats, ValueQuery>;

    // ==================== 🆕 存储膨胀防护：归档存储 ====================

    /// 客户已归档订单ID索引（永久保留，用于历史查询）
    /// 订单数据保留在 Orders 存储中，此处仅存储ID列表
    #[pallet::storage]
    #[pallet::getter(fn customer_archived_order_ids)]
    pub type CustomerArchivedOrderIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<10000>>,  // 支持每用户最多10000条历史订单
        ValueQuery,
    >;

    /// 提供者已归档订单ID索引（永久保留）
    #[pallet::storage]
    #[pallet::getter(fn provider_archived_order_ids)]
    pub type ProviderArchivedOrderIds<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<50000>>,  // 提供者可能有更多历史订单
        ValueQuery,
    >;

    /// 归档游标（用于on_idle处理订单）
    #[pallet::storage]
    pub type ArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 悬赏归档游标
    #[pallet::storage]
    pub type BountyArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 用户已归档悬赏问题ID索引（永久保留）
    /// 悬赏数据保留在 BountyQuestions 存储中，此处仅存储ID列表
    #[pallet::storage]
    #[pallet::getter(fn user_archived_bounties)]
    pub type UserArchivedBounties<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<5000>>,  // 支持每用户最多5000条历史悬赏
        ValueQuery,
    >;

    /// 用户已归档悬赏回答ID索引（永久保留）
    #[pallet::storage]
    #[pallet::getter(fn user_archived_bounty_answers)]
    pub type UserArchivedBountyAnswers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<10000>>,  // 活跃回答者可能有更多历史
        ValueQuery,
    >;

    /// 市场永久统计
    #[pallet::storage]
    #[pallet::getter(fn permanent_stats)]
    pub type PermanentStats<T: Config> = StorageValue<_, MarketPermanentStats, ValueQuery>;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 提供者已注册
        ProviderRegistered {
            provider: T::AccountId,
            deposit: BalanceOf<T>,
            supported_types: u8,
        },

        /// 提供者信息已更新
        ProviderUpdated { provider: T::AccountId },

        /// 提供者已暂停
        ProviderPaused { provider: T::AccountId },

        /// 提供者已恢复
        ProviderResumed { provider: T::AccountId },

        /// 提供者已注销
        ProviderDeactivated { provider: T::AccountId },

        /// 提供者已封禁
        ProviderBanned {
            provider: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        },

        /// 提供者保证金已扣除
        ProviderDepositSlashed {
            provider: T::AccountId,
            order_id: u64,
            amount: BalanceOf<T>,
            to_customer: bool,
        },

        /// 提供者保证金已补充
        ProviderDepositToppedUp {
            provider: T::AccountId,
            amount: BalanceOf<T>,
            new_total: BalanceOf<T>,
        },

        /// 提供者保证金不足警告
        ProviderDepositInsufficient {
            provider: T::AccountId,
            current: BalanceOf<T>,
            required: BalanceOf<T>,
        },

        /// 提供者等级已提升
        ProviderTierUpgraded {
            provider: T::AccountId,
            new_tier: ProviderTier,
        },

        /// 服务套餐已创建
        PackageCreated {
            provider: T::AccountId,
            package_id: u32,
            divination_type: DivinationType,
            price: BalanceOf<T>,
        },

        /// 服务套餐已更新
        PackageUpdated {
            provider: T::AccountId,
            package_id: u32,
        },

        /// 服务套餐已删除
        PackageRemoved {
            provider: T::AccountId,
            package_id: u32,
        },

        /// 订单已创建
        OrderCreated {
            order_id: u64,
            customer: T::AccountId,
            provider: T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            amount: BalanceOf<T>,
        },

        /// 订单已支付
        OrderPaid { order_id: u64 },

        /// 订单已接受
        OrderAccepted {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 订单已拒绝
        OrderRejected {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 解读结果已提交（服务提供者完成解读）
        InterpretationSubmitted {
            order_id: u64,
            interpretation_cid: BoundedVec<u8, T::MaxCidLength>,
        },

        /// 订单已完成
        OrderCompleted {
            order_id: u64,
            provider_earnings: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
        },

        /// 🆕 联盟奖励已分配
        AffiliateRewardDistributed {
            order_id: u64,
            customer: T::AccountId,
            total_distributed: BalanceOf<T>,
        },

        // ==================== 🆕 OCW 异步解读事件 ====================

        /// 多媒体解读已提交（等待 OCW 确认）
        InterpretationPending {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 解读已确认（OCW 处理完成，已结算）
        InterpretationConfirmed {
            order_id: u64,
            content_cid: BoundedVec<u8, T::MaxCidLength>,
        },

        /// 解读处理超时
        InterpretationTimeout {
            order_id: u64,
        },

        /// 解读内容已更新
        InterpretationUpdated {
            order_id: u64,
            provider: T::AccountId,
        },

        /// 订单已取消
        OrderCancelled { order_id: u64 },

        /// 订单已退款
        OrderRefunded {
            order_id: u64,
            amount: BalanceOf<T>,
        },

        /// 追问已提交
        FollowUpSubmitted { order_id: u64, index: u32 },

        /// 追问已回复（服务提供者回复追问）
        FollowUpReplied { order_id: u64, index: u32 },

        /// 评价已提交
        ReviewSubmitted {
            order_id: u64,
            divination_type: DivinationType,
            rating: u8,
        },

        /// 提供者已回复评价
        ReviewReplied { order_id: u64 },

        /// 提现已申请
        WithdrawalRequested {
            withdrawal_id: u64,
            provider: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 提现已完成
        WithdrawalCompleted { withdrawal_id: u64 },

        /// 提现已取消
        WithdrawalCancelled { withdrawal_id: u64 },

        // ==================== 悬赏问答事件 ====================

        /// 悬赏问题已创建
        BountyCreated {
            bounty_id: u64,
            creator: T::AccountId,
            divination_type: DivinationType,
            bounty_amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
        },

        /// 悬赏回答已提交
        BountyAnswerSubmitted {
            answer_id: u64,
            bounty_id: u64,
            answerer: T::AccountId,
        },

        /// 悬赏问题已关闭（停止接受回答）
        BountyClosed { bounty_id: u64 },

        /// 悬赏答案已被投票
        BountyAnswerVoted {
            bounty_id: u64,
            answer_id: u64,
            voter: T::AccountId,
        },

        /// 悬赏答案已采纳（选择前三名）
        BountyAnswersAdopted {
            bounty_id: u64,
            first_place: u64,
            second_place: Option<u64>,
            third_place: Option<u64>,
        },

        /// 悬赏已结算（奖励已分配）
        BountySettled {
            bounty_id: u64,
            total_distributed: BalanceOf<T>,
            platform_fee: BalanceOf<T>,
            participant_count: u32,
        },

        /// 悬赏已取消
        BountyCancelled {
            bounty_id: u64,
            refund_amount: BalanceOf<T>,
        },

        /// 悬赏已过期
        BountyExpired {
            bounty_id: u64,
            refund_amount: BalanceOf<T>,
        },

        /// 悬赏奖励已发放
        BountyRewardPaid {
            bounty_id: u64,
            recipient: T::AccountId,
            amount: BalanceOf<T>,
            rank: u8, // 1=第一名, 2=第二名, 3=第三名, 0=参与奖
        },

        // ==================== 个人主页事件 ====================

        /// 个人资料已更新
        ProfileUpdated { provider: T::AccountId },

        /// 资质证书已添加
        CertificateAdded {
            provider: T::AccountId,
            certificate_id: u32,
        },

        /// 资质证书已删除
        CertificateRemoved {
            provider: T::AccountId,
            certificate_id: u32,
        },

        /// 资质证书验证状态已更新
        CertificateVerified {
            provider: T::AccountId,
            certificate_id: u32,
            is_verified: bool,
        },

        /// 作品已发布
        PortfolioPublished {
            provider: T::AccountId,
            portfolio_id: u32,
            divination_type: DivinationType,
        },

        /// 作品已更新
        PortfolioUpdated {
            provider: T::AccountId,
            portfolio_id: u32,
        },

        /// 作品已删除
        PortfolioRemoved {
            provider: T::AccountId,
            portfolio_id: u32,
        },

        /// 作品被点赞
        PortfolioLiked {
            provider: T::AccountId,
            portfolio_id: u32,
            liker: T::AccountId,
        },

        /// 技能标签已更新
        SkillTagsUpdated { provider: T::AccountId },

        // ==================== 信用体系事件 ====================

        /// 信用档案已创建
        CreditProfileCreated { provider: T::AccountId },

        /// 信用评估完成
        CreditEvaluated {
            provider: T::AccountId,
            new_score: u16,
            new_level: CreditLevel,
        },

        /// 信用等级变更
        CreditLevelChanged {
            provider: T::AccountId,
            old_level: CreditLevel,
            new_level: CreditLevel,
        },

        /// 违规记录创建
        ViolationRecorded {
            provider: T::AccountId,
            violation_id: u64,
            violation_type: ViolationType,
            penalty: PenaltyType,
            deduction_points: u16,
        },

        /// 违规申诉提交
        ViolationAppealed {
            provider: T::AccountId,
            violation_id: u64,
        },

        /// 申诉结果处理完成
        AppealResolved {
            provider: T::AccountId,
            violation_id: u64,
            result: AppealResult,
            restored_points: u16,
        },

        /// 信用修复任务申请
        CreditRepairRequested {
            provider: T::AccountId,
            task_type: RepairTaskType,
            target_value: u32,
        },

        /// 投诉裁决后订单退款
        OrderRefundedOnComplaint {
            order_id: u64,
            customer: T::AccountId,
            amount: BalanceOf<T>,
        },

        /// 信用修复任务完成
        CreditRepairCompleted {
            provider: T::AccountId,
            task_type: RepairTaskType,
            restored_points: u16,
        },

        /// 加入信用黑名单
        AddedToBlacklist { provider: T::AccountId },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 提供者已存在
        ProviderAlreadyExists,
        /// 提供者不存在
        ProviderNotFound,
        /// 提供者未激活
        ProviderNotActive,
        /// 保证金不足
        InsufficientDeposit,
        /// 套餐不存在
        PackageNotFound,
        /// 套餐已达上限
        TooManyPackages,
        /// 价格低于最低限制
        PriceTooLow,
        /// 价格高于最高限制（修复 H-13）
        PriceTooHigh,
        /// 订单不存在
        OrderNotFound,
        /// 订单状态无效
        InvalidOrderStatus,
        /// 非订单所有者
        NotOrderOwner,
        /// 非服务提供者
        NotProvider,
        /// 余额不足
        InsufficientBalance,
        /// 无追问次数
        NoFollowUpsRemaining,
        /// 追问不存在
        FollowUpNotFound,
        /// 已评价
        AlreadyReviewed,
        /// 评分无效
        InvalidRating,
        /// 评价期已过
        ReviewPeriodExpired,
        /// 提现金额无效
        InvalidWithdrawalAmount,
        /// 提现请求不存在
        WithdrawalNotFound,
        /// 名称过长
        NameTooLong,
        /// 简介过长
        BioTooLong,
        /// 描述过长
        DescriptionTooLong,
        /// CID 过长
        CidTooLong,
        /// 订单列表已满
        OrderListFull,
        /// 追问列表已满
        FollowUpListFull,
        /// 不能给自己下单
        CannotOrderSelf,
        /// 提供者已被封禁
        ProviderBanned,
        /// 占卜结果不存在
        DivinationResultNotFound,
        /// 不是占卜结果的创建者
        NotResultCreator,
        /// 提供者不支持该占卜类型
        DivinationTypeNotSupported,
        /// 提供者状态无效（非预期的状态转换）
        InvalidProviderStatus,
        /// 加急服务不可用
        UrgentNotAvailable,
        /// 投票功能未启用
        VotingNotAllowed,
        /// 悬赏未被采纳
        BountyNotAdopted,

        // ==================== 悬赏问答错误 ====================

        /// 悬赏问题不存在
        BountyNotFound,
        /// 悬赏问题不是开放状态
        BountyNotOpen,
        /// 悬赏问题已关闭
        BountyAlreadyClosed,
        /// 悬赏回答不存在
        BountyAnswerNotFound,
        /// 不能回答自己的悬赏
        CannotAnswerOwnBounty,
        /// 已经回答过该悬赏
        AlreadyAnswered,
        /// 悬赏回答数已达上限
        BountyAnswerLimitReached,
        /// 不是悬赏创建者
        NotBountyCreator,
        /// 悬赏金额过低
        BountyAmountTooLow,
        /// 悬赏已过截止时间
        BountyDeadlinePassed,
        /// 悬赏截止时间无效
        InvalidBountyDeadline,
        /// 回答数不足以采纳
        NotEnoughAnswers,
        /// 已投票
        AlreadyVoted,
        /// 悬赏已被采纳
        BountyAlreadyAdopted,
        /// 悬赏已结算
        BountyAlreadySettled,
        /// 悬赏不能取消（已有回答）
        BountyCannotCancel,
        /// 悬赏未过期
        BountyNotExpired,
        /// 仅限认证提供者
        CertifiedProviderOnly,
        /// 悬赏列表已满
        BountyListFull,
        /// 奖励分配比例无效
        InvalidRewardDistribution,

        // ==================== 个人主页错误 ====================

        /// 资质证书不存在
        CertificateNotFound,
        /// 证书数量已达上限
        TooManyCertificates,
        /// 作品不存在
        PortfolioNotFound,
        /// 作品数量已达上限
        TooManyPortfolios,
        /// 已点赞
        AlreadyLiked,
        /// 标签数量过多
        TooManyTags,

        // ==================== 信用体系错误 ====================

        /// 信用档案不存在
        CreditProfileNotFound,
        /// 违规记录不存在
        ViolationNotFound,
        /// 不是违规记录所有者
        NotViolationOwner,
        /// 已申诉
        AlreadyAppealed,
        /// 违规已过期
        ViolationExpired,
        /// 未申诉
        NotAppealed,
        /// 信用分过高，无需修复
        CreditTooHighForRepair,
        /// 重复的修复任务
        DuplicateRepairTask,
        /// 活跃任务过多
        TooManyActiveTasks,
        /// 任务数量过多
        TooManyTasks,
        /// 违规记录过多
        TooManyViolations,
        /// 已被列入黑名单
        InBlacklist,
        /// 信用等级不足
        InsufficientCreditLevel,

        // ==================== 🆕 OCW 异步解读错误 ====================

        /// 待处理解读不存在
        PendingInterpretationNotFound,
        /// 待处理解读队列已满
        PendingQueueFull,
        /// 解读已提交，等待确认
        InterpretationAlreadyPending,
        /// 无效的 OCW 提交
        InvalidOcwSubmission,
        /// 媒体数量超过上限
        TooManyMediaItems,
        /// 修改窗口已过期
        EditWindowExpired,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 注册成为服务提供者
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `name`: 显示名称
        /// - `bio`: 个人简介
        /// - `specialties`: 擅长领域位图
        /// - `supported_divination_types`: 支持的占卜类型位图
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn register_provider(
            origin: OriginFor<T>,
            name: Vec<u8>,
            bio: Vec<u8>,
            specialties: u16,
            supported_divination_types: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 确保未注册
            ensure!(
                !Providers::<T>::contains_key(&who),
                Error::<T>::ProviderAlreadyExists
            );

            let name_bounded: BoundedVec<u8, T::MaxNameLength> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let bio_bounded: BoundedVec<u8, T::MaxBioLength> =
                BoundedVec::try_from(bio).map_err(|_| Error::<T>::BioTooLong)?;

            // 计算保证金：使用pricing换算，确保不低于100 USDT价值
            let min_deposit_dust = T::MinDeposit::get();
            let min_deposit_usd = T::MinDepositUsd::get(); // 100_000_000 (100 USDT)
            
            // 使用pricing模块换算100 USDT对应的DUST数量
            let deposit = if let Some(price) = T::Pricing::get_dust_to_usd_rate() {
                let price_u128: u128 = price.saturated_into();
                if price_u128 > 0u128 {
                    // DUST数量 = USD金额 * 精度 / 价格
                    let required_dust_u128 = (min_deposit_usd as u128).saturating_mul(1_000_000u128) / price_u128;
                    let required_dust: BalanceOf<T> = required_dust_u128.saturated_into();
                    // 取pricing换算值和最小值中的较大者
                    if required_dust > min_deposit_dust {
                        required_dust
                    } else {
                        min_deposit_dust
                    }
                } else {
                    min_deposit_dust
                }
            } else {
                min_deposit_dust
            };
            
            // 锁定保证金
            T::Currency::reserve(&who, deposit)?;

            let block_number = <frame_system::Pallet<T>>::block_number();

            let provider = Provider {
                account: who.clone(),
                name: name_bounded,
                bio: bio_bounded,
                avatar_cid: None,
                tier: ProviderTier::Novice,
                status: ProviderStatus::Active,
                deposit,
                registered_at: block_number,
                total_orders: 0,
                completed_orders: 0,
                cancelled_orders: 0,
                total_ratings: 0,
                rating_sum: 0,
                total_earnings: Zero::zero(),
                specialties,
                supported_divination_types,
                accepts_urgent: false,
                last_active_at: block_number,
            };

            Providers::<T>::insert(&who, provider);

            // 更新统计
            MarketStatistics::<T>::mutate(|s| s.active_providers += 1);

            Self::deposit_event(Event::ProviderRegistered {
                provider: who,
                deposit,
                supported_types: supported_divination_types,
            });

            Ok(())
        }

        /// 更新提供者信息
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_provider(
            origin: OriginFor<T>,
            name: Option<Vec<u8>>,
            bio: Option<Vec<u8>>,
            avatar_cid: Option<Vec<u8>>,
            specialties: Option<u16>,
            supported_divination_types: Option<u8>,
            accepts_urgent: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 🆕 如果有头像 CID，先 Pin 到 IPFS (Standard 层级)
            if let Some(ref cid) = avatar_cid {
                // 使用 provider 账户地址编码的前8字节作为 subject_id
                let subject_id = who.using_encoded(|bytes| {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
                    u64::from_le_bytes(arr)
                });

                <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-market".to_vec(),
                    subject_id,
                    cid.clone(),
                    pallet_storage_service::PinTier::Standard,
                )?;
            }

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;

                if let Some(n) = name {
                    provider.name =
                        BoundedVec::try_from(n).map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(b) = bio {
                    provider.bio = BoundedVec::try_from(b).map_err(|_| Error::<T>::BioTooLong)?;
                }
                if let Some(cid) = avatar_cid {
                    provider.avatar_cid =
                        Some(BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?);
                }
                if let Some(s) = specialties {
                    provider.specialties = s;
                }
                if let Some(types) = supported_divination_types {
                    provider.supported_divination_types = types;
                }
                if let Some(u) = accepts_urgent {
                    provider.accepts_urgent = u;
                }

                provider.last_active_at = <frame_system::Pallet<T>>::block_number();

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ProviderUpdated { provider: who });

            Ok(())
        }

        /// 暂停接单
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn pause_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;
                ensure!(
                    provider.status == ProviderStatus::Active,
                    Error::<T>::ProviderNotActive
                );
                provider.status = ProviderStatus::Paused;
                Ok::<_, DispatchError>(())
            })?;

            MarketStatistics::<T>::mutate(|s| {
                s.active_providers = s.active_providers.saturating_sub(1)
            });

            Self::deposit_event(Event::ProviderPaused { provider: who });

            Ok(())
        }

        /// 恢复接单
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn resume_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::ProviderNotFound)?;
                ensure!(
                    provider.status == ProviderStatus::Paused,
                    Error::<T>::InvalidProviderStatus
                );
                
                // 检查保证金是否达到最低要求
                let min_deposit = T::MinDeposit::get();
                ensure!(
                    provider.deposit >= min_deposit,
                    Error::<T>::InsufficientDeposit
                );
                
                provider.status = ProviderStatus::Active;
                provider.last_active_at = <frame_system::Pallet<T>>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            MarketStatistics::<T>::mutate(|s| s.active_providers += 1);

            Self::deposit_event(Event::ProviderResumed { provider: who });

            Ok(())
        }

        /// 补充保证金
        /// 
        /// 当保证金因违规被扣除后，提供者可以补充保证金以恢复正常接单
        #[pallet::call_index(41)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn top_up_deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;

            // 不能是已封禁状态
            ensure!(
                provider.status != ProviderStatus::Banned,
                Error::<T>::ProviderBanned
            );

            // 锁定保证金
            T::Currency::reserve(&who, amount)?;

            // 更新保证金
            provider.deposit = provider.deposit.saturating_add(amount);
            let new_total = provider.deposit;

            // 检查是否达到最低要求，如果达到且之前是暂停状态，可以恢复
            let min_deposit = T::MinDeposit::get();
            if provider.deposit >= min_deposit && provider.status == ProviderStatus::Paused {
                // 保证金已达标，可以恢复接单（需要手动调用 resume_provider）
            }

            Providers::<T>::insert(&who, provider);

            Self::deposit_event(Event::ProviderDepositToppedUp {
                provider: who,
                amount,
                new_total,
            });

            Ok(())
        }

        /// 注销提供者（需要无进行中订单）
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn deactivate_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;

            // 退还保证金
            T::Currency::unreserve(&who, provider.deposit);

            // 退还余额
            let balance = ProviderBalances::<T>::take(&who);
            if !balance.is_zero() {
                T::Currency::transfer(
                    &T::PlatformAccount::get(),
                    &who,
                    balance,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            Providers::<T>::remove(&who);

            MarketStatistics::<T>::mutate(|s| {
                if provider.status == ProviderStatus::Active {
                    s.active_providers = s.active_providers.saturating_sub(1);
                }
            });

            Self::deposit_event(Event::ProviderDeactivated { provider: who });

            Ok(())
        }

        /// 创建服务套餐
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn create_package(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            service_type: ServiceType,
            name: Vec<u8>,
            description: Vec<u8>,
            price: BalanceOf<T>,
            duration: u32,
            follow_up_count: u8,
            urgent_available: bool,
            urgent_surcharge: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证提供者
            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;
            ensure!(
                provider.supports_divination_type(divination_type),
                Error::<T>::DivinationTypeNotSupported
            );
            ensure!(price >= T::MinServicePrice::get(), Error::<T>::PriceTooLow);
            ensure!(price <= T::MaxServicePrice::get(), Error::<T>::PriceTooHigh);
            
            // 🆕 P1修复: 验证组合价格（基础价 + 加急加价）不超过限制
            if urgent_available && urgent_surcharge > 0 {
                let surcharge = price.saturating_mul(urgent_surcharge.into()) / 10000u32.into();
                let max_price = price.saturating_add(surcharge);
                ensure!(max_price <= T::MaxServicePrice::get(), Error::<T>::PriceTooHigh);
            }

            let name_bounded: BoundedVec<u8, ConstU32<64>> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let desc_bounded: BoundedVec<u8, T::MaxDescriptionLength> =
                BoundedVec::try_from(description).map_err(|_| Error::<T>::DescriptionTooLong)?;

            let package_id = NextPackageId::<T>::get(&who);
            ensure!(
                package_id < T::MaxPackagesPerProvider::get(),
                Error::<T>::TooManyPackages
            );

            let package = ServicePackage {
                id: package_id,
                divination_type,
                service_type,
                name: name_bounded,
                description: desc_bounded,
                price,
                duration,
                follow_up_count,
                urgent_available,
                urgent_surcharge,
                is_active: true,
                sales_count: 0,
            };

            Packages::<T>::insert(&who, package_id, package);
            NextPackageId::<T>::insert(&who, package_id.saturating_add(1));

            Self::deposit_event(Event::PackageCreated {
                provider: who,
                package_id,
                divination_type,
                price,
            });

            Ok(())
        }

        /// 更新服务套餐
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_package(
            origin: OriginFor<T>,
            package_id: u32,
            price: Option<BalanceOf<T>>,
            description: Option<Vec<u8>>,
            is_active: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Packages::<T>::try_mutate(&who, package_id, |maybe_package| {
                let package = maybe_package.as_mut().ok_or(Error::<T>::PackageNotFound)?;

                if let Some(p) = price {
                    ensure!(p >= T::MinServicePrice::get(), Error::<T>::PriceTooLow);
                    ensure!(p <= T::MaxServicePrice::get(), Error::<T>::PriceTooHigh);
                    
                    // 🆕 P1修复: 验证新价格与现有加急加价组合后不超过限制
                    if package.urgent_available && package.urgent_surcharge > 0 {
                        let surcharge = p.saturating_mul(package.urgent_surcharge.into()) / 10000u32.into();
                        let max_price = p.saturating_add(surcharge);
                        ensure!(max_price <= T::MaxServicePrice::get(), Error::<T>::PriceTooHigh);
                    }
                    
                    package.price = p;
                }
                if let Some(d) = description {
                    package.description =
                        BoundedVec::try_from(d).map_err(|_| Error::<T>::DescriptionTooLong)?;
                }
                if let Some(a) = is_active {
                    package.is_active = a;
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::PackageUpdated {
                provider: who,
                package_id,
            });

            Ok(())
        }

        /// 删除服务套餐
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_package(origin: OriginFor<T>, package_id: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Packages::<T>::contains_key(&who, package_id),
                Error::<T>::PackageNotFound
            );
            Packages::<T>::remove(&who, package_id);

            Self::deposit_event(Event::PackageRemoved {
                provider: who,
                package_id,
            });

            Ok(())
        }

        /// 创建订单
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn create_order(
            origin: OriginFor<T>,
            provider_account: T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            package_id: u32,
            question_cid: Vec<u8>,
            is_urgent: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 不能给自己下单
            ensure!(who != provider_account, Error::<T>::CannotOrderSelf);

            // 验证占卜结果存在
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证提供者
            let provider =
                Providers::<T>::get(&provider_account).ok_or(Error::<T>::ProviderNotFound)?;
            ensure!(
                provider.status == ProviderStatus::Active,
                Error::<T>::ProviderNotActive
            );
            ensure!(
                provider.status != ProviderStatus::Banned,
                Error::<T>::ProviderBanned
            );
            ensure!(
                provider.supports_divination_type(divination_type),
                Error::<T>::DivinationTypeNotSupported
            );

            // 验证套餐
            let package = Packages::<T>::get(&provider_account, package_id)
                .ok_or(Error::<T>::PackageNotFound)?;
            ensure!(package.is_active, Error::<T>::PackageNotFound);
            ensure!(
                package.divination_type == divination_type,
                Error::<T>::DivinationTypeNotSupported
            );

            // 验证加急
            if is_urgent {
                ensure!(
                    package.urgent_available && provider.accepts_urgent,
                    Error::<T>::UrgentNotAvailable
                );
            }

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            // 计算价格
            let mut amount = package.price;
            if is_urgent {
                let surcharge =
                    amount.saturating_mul(package.urgent_surcharge.into()) / 10000u32.into();
                amount = amount.saturating_add(surcharge);
            }

            // 🆕 P0修复: 验证最终价格不超过限制
            ensure!(amount <= T::MaxServicePrice::get(), Error::<T>::PriceTooHigh);

            // 计算平台手续费
            let platform_fee_rate = provider.tier.platform_fee_rate();
            let platform_fee =
                amount.saturating_mul(platform_fee_rate.into()) / 10000u32.into();

            // 扣款到平台账户（托管）
            T::Currency::transfer(
                &who,
                &T::PlatformAccount::get(),
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let order_id = NextOrderId::<T>::get();
            NextOrderId::<T>::put(order_id.saturating_add(1));

            // 🆕 自动 Pin 问题描述到 IPFS (Temporary 层级)
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                order_id,
                question_cid,
                pallet_storage_service::PinTier::Temporary,
            )?;

            let block_number = <frame_system::Pallet<T>>::block_number();

            let order = Order {
                id: order_id,
                customer: who.clone(),
                provider: provider_account.clone(),
                divination_type,
                result_id,
                package_id,
                amount,
                platform_fee,
                is_urgent,
                status: OrderStatus::Paid,
                question_cid: question_cid_bounded,
                interpretation_cid: None,
                created_at: block_number,
                paid_at: Some(block_number),
                accepted_at: None,
                completed_at: None,
                follow_ups_remaining: package.follow_up_count,
                rating: None,
                review_cid: None,
            };

            Orders::<T>::insert(order_id, order);

            // 更新索引
            CustomerOrders::<T>::try_mutate(&who, |list| {
                list.try_push(order_id)
                    .map_err(|_| Error::<T>::OrderListFull)
            })?;
            ProviderOrders::<T>::try_mutate(&provider_account, |list| {
                list.try_push(order_id)
                    .map_err(|_| Error::<T>::OrderListFull)
            })?;

            // 更新套餐销量
            Packages::<T>::mutate(&provider_account, package_id, |maybe_package| {
                if let Some(p) = maybe_package {
                    p.sales_count += 1;
                }
            });

            // 更新统计
            MarketStatistics::<T>::mutate(|s| {
                s.total_orders += 1;
                s.total_volume = s.total_volume.saturating_add(amount);
            });
            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.order_count += 1;
                s.volume = s.volume.saturating_add(amount);
            });

            // 🆕 自动授权双方聊天（订单场景）
            // 允许命主和命理师在订单期间相互发送消息
            let chat_duration = T::OrderChatDuration::get();
            let metadata = sp_std::vec![]; // 可扩展：添加订单金额等信息
            let _ = T::ChatPermission::grant_bidirectional_scene_authorization(
                *b"div_mrkt",  // 来源标识：divination-market
                &who,
                &provider_account,
                SceneType::Order,
                SceneId::Numeric(order_id),
                Some(chat_duration),
                metadata,
            );
            // 注意：聊天授权失败不应阻止订单创建，因此使用 let _ 忽略错误

            Self::deposit_event(Event::OrderCreated {
                order_id,
                customer: who,
                provider: provider_account,
                divination_type,
                result_id,
                amount,
            });

            Self::deposit_event(Event::OrderPaid { order_id });

            Ok(())
        }

        /// 接受订单
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn accept_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.provider == who, Error::<T>::NotProvider);
                ensure!(
                    order.status == OrderStatus::Paid,
                    Error::<T>::InvalidOrderStatus
                );

                order.status = OrderStatus::Accepted;
                order.accepted_at = Some(<frame_system::Pallet<T>>::block_number());

                Ok::<_, DispatchError>(())
            })?;

            // 更新提供者活跃时间
            Providers::<T>::mutate(&who, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            Self::deposit_event(Event::OrderAccepted {
                order_id,
                provider: who,
            });

            Ok(())
        }

        /// 拒绝订单（退款给客户）
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn reject_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.provider == who, Error::<T>::NotProvider);
            ensure!(
                order.status == OrderStatus::Paid,
                Error::<T>::InvalidOrderStatus
            );

            // 退款给客户
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &order.customer,
                order.amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Cancelled;
                }
            });

            // 🆕 订单被拒绝时撤销聊天授权
            let _ = T::ChatPermission::revoke_scene_authorization(
                *b"div_mrkt",
                &order.customer,
                &who,
                SceneType::Order,
                SceneId::Numeric(order_id),
            );

            Self::deposit_event(Event::OrderRejected {
                order_id,
                provider: who,
            });

            Ok(())
        }

        // ==================== 🆕 OCW 异步解读 Extrinsics ====================

        /// 提交解读结果（多媒体异步结算版本）
        /// 
        /// 支持图片、视频、文档等多媒体内容
        /// 提交后由 OCW 构建 JSON 清单并上传 IPFS，确认后结算
        /// 2天修改窗口内可调用 update_interpretation 修改
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn submit_interpretation(
            origin: OriginFor<T>,
            order_id: u64,
            text_cid: Vec<u8>,
            imgs: Vec<Vec<u8>>,
            vids: Vec<Vec<u8>>,
            docs: Vec<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // 1. 验证媒体数量
            ensure!(imgs.len() <= 20, Error::<T>::TooManyMediaItems);
            ensure!(vids.len() <= 5, Error::<T>::TooManyMediaItems);
            ensure!(docs.len() <= 10, Error::<T>::TooManyMediaItems);
            
            // 2. 转换 CID
            let text_cid_bounded: BoundedVec<u8, T::MaxCidLength> = 
                text_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
            
            let imgs_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                imgs.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            let vids_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                vids.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            let docs_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                docs.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            // 3. 验证订单状态
            Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.provider == who, Error::<T>::NotProvider);
                ensure!(order.status == OrderStatus::Accepted, Error::<T>::InvalidOrderStatus);
                
                // 4. 状态变更为"解读已提交"
                order.status = OrderStatus::InterpretationSubmitted;
                
                Ok::<_, DispatchError>(())
            })?;
            
            // 5. 确保没有重复提交
            ensure!(
                !PendingInterpretations::<T>::contains_key(order_id),
                Error::<T>::InterpretationAlreadyPending
            );
            
            // 6. 创建待处理解读
            let now = <frame_system::Pallet<T>>::block_number();
            let pending = PendingInterpretation {
                order_id,
                text_cid: text_cid_bounded,
                imgs: imgs_bounded,
                vids: vids_bounded,
                docs: docs_bounded,
                submitted_at: now,
                status: InterpretationProcessStatus::Pending,
                retry_count: 0,
            };
            
            PendingInterpretations::<T>::insert(order_id, pending);
            
            // 7. 添加到队列
            PendingInterpretationQueue::<T>::try_mutate(|queue| {
                queue.try_push(order_id).map_err(|_| Error::<T>::PendingQueueFull)
            })?;
            
            // 8. 发送事件
            Self::deposit_event(Event::InterpretationPending {
                order_id,
                provider: who,
            });
            
            Ok(())
        }

        /// 确认解读（由 OCW 或管理员调用）
        /// 
        /// OCW 处理完成后调用此方法完成结算
        #[pallet::call_index(51)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn confirm_interpretation(
            origin: OriginFor<T>,
            order_id: u64,
            content_cid: Vec<u8>,
        ) -> DispatchResult {
            // 允许 Root 或 OCW 签名者调用
            let _ = ensure_root(origin.clone()).or_else(|_| {
                let _who = ensure_signed(origin)?;
                // TODO: 验证是否是授权的 OCW 签名者
                Ok::<_, DispatchError>(())
            })?;
            
            let content_cid_bounded: BoundedVec<u8, T::MaxCidLength> = 
                content_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
            
            // 1. 获取待处理解读
            let _pending = PendingInterpretations::<T>::get(order_id)
                .ok_or(Error::<T>::PendingInterpretationNotFound)?;
            
            // 2. 更新订单并提取结算信息
            let (divination_type, provider, customer, amount, platform_fee) = 
                Orders::<T>::try_mutate(order_id, |maybe_order| {
                    let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                    ensure!(
                        order.status == OrderStatus::InterpretationSubmitted,
                        Error::<T>::InvalidOrderStatus
                    );
                    
                    order.interpretation_cid = Some(content_cid_bounded.clone());
                    order.status = OrderStatus::Completed;
                    order.completed_at = Some(<frame_system::Pallet<T>>::block_number());
                    
                    Ok::<_, DispatchError>((
                        order.divination_type,
                        order.provider.clone(),
                        order.customer.clone(),
                        order.amount,
                        order.platform_fee,
                    ))
                })?;
            
            // 3. 执行结算
            let provider_earnings = amount.saturating_sub(platform_fee);
            
            ProviderBalances::<T>::mutate(&provider, |balance| {
                *balance = balance.saturating_add(provider_earnings);
            });
            
            Providers::<T>::mutate(&provider, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.total_orders += 1;
                    p.completed_orders += 1;
                    p.total_earnings = p.total_earnings.saturating_add(provider_earnings);
                    p.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });
            
            MarketStatistics::<T>::mutate(|s| {
                s.completed_orders += 1;
                s.platform_earnings = s.platform_earnings.saturating_add(platform_fee);
            });
            TypeStatistics::<T>::mutate(divination_type, |s| {
                s.completed_count += 1;
            });
            
            // 4. 平台抽成全部通过联盟分成资金流向处理
            // 资金流向：销毁 5% + 国库 2% + 存储 3% + 推荐链 90%
            if !platform_fee.is_zero() {
                let platform_fee_u128: u128 = platform_fee.saturated_into();
                
                if let Ok(distributed_u128) = T::AffiliateDistributor::distribute_rewards(
                    &customer,
                    platform_fee_u128,
                    Some((15, order_id)),
                ) {
                    let distributed: BalanceOf<T> = distributed_u128.saturated_into();
                    
                    TotalAffiliateDistributed::<T>::mutate(|total| {
                        *total = total.saturating_add(distributed);
                    });
                    
                    Self::deposit_event(Event::AffiliateRewardDistributed {
                        order_id,
                        customer,
                        total_distributed: distributed,
                    });
                }
            }
            
            // 5. 清理待处理
            PendingInterpretations::<T>::remove(order_id);
            PendingInterpretationQueue::<T>::mutate(|queue| {
                queue.retain(|id| *id != order_id);
            });

            // 🆕 订单完成后撤销聊天授权（可选：保留一段时间供追问）
            // 注意：这里不立即撤销，让授权自然过期，以便用户可以追问
            // 如需立即撤销，取消下面的注释：
            // let _ = T::ChatPermission::revoke_scene_authorization(
            //     *b"div_mrkt",
            //     &customer,
            //     &provider,
            //     SceneType::Order,
            //     SceneId::Numeric(order_id),
            // );
            
            // 6. 发送事件
            Self::deposit_event(Event::InterpretationConfirmed {
                order_id,
                content_cid: content_cid_bounded,
            });
            
            Self::deposit_event(Event::OrderCompleted {
                order_id,
                provider_earnings,
                platform_fee,
            });
            
            Ok(())
        }

        /// 修改待处理解读（在修改窗口内可任意修改）
        /// 
        /// 允许提供者在 InterpretationEditWindow 内修改已提交的解读内容
        #[pallet::call_index(50)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn update_interpretation(
            origin: OriginFor<T>,
            order_id: u64,
            text_cid: Vec<u8>,
            imgs: Vec<Vec<u8>>,
            vids: Vec<Vec<u8>>,
            docs: Vec<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // 1. 验证媒体数量
            ensure!(imgs.len() <= 20, Error::<T>::TooManyMediaItems);
            ensure!(vids.len() <= 5, Error::<T>::TooManyMediaItems);
            ensure!(docs.len() <= 10, Error::<T>::TooManyMediaItems);
            
            // 2. 获取并验证待处理解读
            let pending = PendingInterpretations::<T>::get(order_id)
                .ok_or(Error::<T>::PendingInterpretationNotFound)?;
            
            // 3. 验证修改窗口
            let now = <frame_system::Pallet<T>>::block_number();
            let edit_window = T::InterpretationEditWindow::get();
            ensure!(
                now <= pending.submitted_at.saturating_add(edit_window),
                Error::<T>::EditWindowExpired
            );
            
            // 4. 验证订单和权限
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.provider == who, Error::<T>::NotProvider);
            ensure!(
                order.status == OrderStatus::InterpretationSubmitted,
                Error::<T>::InvalidOrderStatus
            );
            
            // 5. 转换 CID
            let text_cid_bounded: BoundedVec<u8, T::MaxCidLength> = 
                text_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;
            
            let imgs_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                imgs.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            let vids_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                vids.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            let docs_bounded: BoundedVec<BoundedVec<u8, T::MaxCidLength>, ConstU32<20>> = 
                docs.into_iter()
                    .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map_err(|_| Error::<T>::TooManyMediaItems)?;
            
            // 6. 更新待处理解读（保持原提交时间，不重置修改窗口）
            let updated_pending = PendingInterpretation {
                order_id,
                text_cid: text_cid_bounded,
                imgs: imgs_bounded,
                vids: vids_bounded,
                docs: docs_bounded,
                submitted_at: pending.submitted_at, // 保持原提交时间
                status: InterpretationProcessStatus::Pending,
                retry_count: 0,
            };
            
            PendingInterpretations::<T>::insert(order_id, updated_pending);
            
            // 7. 发送事件
            Self::deposit_event(Event::InterpretationUpdated {
                order_id,
                provider: who,
            });
            
            Ok(())
        }

        /// 提交追问
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn submit_follow_up(
            origin: OriginFor<T>,
            order_id: u64,
            question_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            // 验证订单
            Orders::<T>::try_mutate(order_id, |maybe_order| {
                let order = maybe_order.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(order.customer == who, Error::<T>::NotOrderOwner);
                ensure!(
                    order.status == OrderStatus::Completed,
                    Error::<T>::InvalidOrderStatus
                );
                ensure!(
                    order.follow_ups_remaining > 0,
                    Error::<T>::NoFollowUpsRemaining
                );

                order.follow_ups_remaining -= 1;

                Ok::<_, DispatchError>(())
            })?;

            // 🆕 自动 Pin 追问内容到 IPFS (Temporary 层级)
            // 使用 order_id + follow_up_index 作为唯一标识
            let follow_up_count = FollowUps::<T>::get(order_id).len() as u64;
            let subject_id = order_id.saturating_mul(1000).saturating_add(follow_up_count);

            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                subject_id,
                question_cid,
                pallet_storage_service::PinTier::Temporary,
            )?;

            let follow_up = FollowUp {
                question_cid: question_cid_bounded,
                reply_cid: None,
                asked_at: <frame_system::Pallet<T>>::block_number(),
                replied_at: None,
            };

            let index = FollowUps::<T>::try_mutate(order_id, |list| {
                let idx = list.len() as u32;
                list.try_push(follow_up)
                    .map_err(|_| Error::<T>::FollowUpListFull)?;
                Ok::<u32, DispatchError>(idx)
            })?;

            Self::deposit_event(Event::FollowUpSubmitted { order_id, index });

            Ok(())
        }

        /// 回复追问
        ///
        /// 服务提供者对客户追问进行回复
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn reply_follow_up(
            origin: OriginFor<T>,
            order_id: u64,
            follow_up_index: u32,
            reply_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reply_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(reply_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            // 验证订单
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.provider == who, Error::<T>::NotProvider);

            // 🆕 自动 Pin 追问回复到 IPFS (Temporary 层级)
            let subject_id = order_id.saturating_mul(1000).saturating_add(follow_up_index as u64).saturating_add(500);

            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                subject_id,
                reply_cid,
                pallet_storage_service::PinTier::Temporary,
            )?;

            FollowUps::<T>::try_mutate(order_id, |list| {
                let follow_up = list
                    .get_mut(follow_up_index as usize)
                    .ok_or(Error::<T>::FollowUpNotFound)?;
                follow_up.reply_cid = Some(reply_cid_bounded);
                follow_up.replied_at = Some(<frame_system::Pallet<T>>::block_number());
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::FollowUpReplied {
                order_id,
                index: follow_up_index,
            });

            Ok(())
        }

        /// 提交评价
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn submit_review(
            origin: OriginFor<T>,
            order_id: u64,
            overall_rating: u8,
            accuracy_rating: u8,
            attitude_rating: u8,
            response_rating: u8,
            content_cid: Option<Vec<u8>>,
            is_anonymous: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证评分
            ensure!(
                overall_rating >= 1
                    && overall_rating <= 5
                    && accuracy_rating >= 1
                    && accuracy_rating <= 5
                    && attitude_rating >= 1
                    && attitude_rating <= 5
                    && response_rating >= 1
                    && response_rating <= 5,
                Error::<T>::InvalidRating
            );

            // 验证订单
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.customer == who, Error::<T>::NotOrderOwner);
            ensure!(
                order.status == OrderStatus::Completed,
                Error::<T>::InvalidOrderStatus
            );

            // 检查是否已评价
            ensure!(
                !Reviews::<T>::contains_key(order_id),
                Error::<T>::AlreadyReviewed
            );

            // 检查评价期限
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(completed_at) = order.completed_at {
                ensure!(
                    current_block <= completed_at + T::ReviewPeriod::get(),
                    Error::<T>::ReviewPeriodExpired
                );
            }

            let content_cid_bounded = content_cid
                .clone()
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 🆕 如果有评价内容 CID，Pin 到 IPFS (Temporary 层级)
            if let Some(ref cid) = content_cid {
                <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-market".to_vec(),
                    order_id,
                    cid.clone(),
                    pallet_storage_service::PinTier::Temporary,
                )?;
            }

            let review = Review {
                order_id,
                reviewer: who.clone(),
                reviewee: order.provider.clone(),
                divination_type: order.divination_type,
                overall_rating,
                accuracy_rating,
                attitude_rating,
                response_rating,
                content_cid: content_cid_bounded,
                created_at: current_block,
                is_anonymous,
                provider_reply_cid: None,
            };

            Reviews::<T>::insert(order_id, review);

            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Reviewed;
                    o.rating = Some(overall_rating);
                }
            });

            // 更新提供者评分
            Providers::<T>::mutate(&order.provider, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.total_ratings += 1;
                    p.rating_sum += overall_rating as u64;

                    // 检查是否可以升级
                    Self::try_upgrade_tier(p);
                }
            });

            // 更新市场统计
            MarketStatistics::<T>::mutate(|s| {
                s.total_reviews += 1;
                // 简单计算平均评分
                let total =
                    s.average_rating as u64 * (s.total_reviews - 1) + overall_rating as u64 * 100;
                s.average_rating = (total / s.total_reviews) as u16;
            });

            Self::deposit_event(Event::ReviewSubmitted {
                order_id,
                divination_type: order.divination_type,
                rating: overall_rating,
            });

            Ok(())
        }

        /// 提供者回复评价
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn reply_review(
            origin: OriginFor<T>,
            order_id: u64,
            reply_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reply_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(reply_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            // 🆕 Pin 评价回复到 IPFS (Temporary 层级)
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                order_id,
                reply_cid,
                pallet_storage_service::PinTier::Temporary,
            )?;

            Reviews::<T>::try_mutate(order_id, |maybe_review| {
                let review = maybe_review.as_mut().ok_or(Error::<T>::OrderNotFound)?;
                ensure!(review.reviewee == who, Error::<T>::NotProvider);

                review.provider_reply_cid = Some(reply_cid_bounded);

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ReviewReplied { order_id });

            Ok(())
        }

        /// 申请提现
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn request_withdrawal(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let balance = ProviderBalances::<T>::get(&who);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);
            ensure!(!amount.is_zero(), Error::<T>::InvalidWithdrawalAmount);

            // 检查平台账户余额是否充足
            let platform_account = T::PlatformAccount::get();
            let platform_balance = T::Currency::free_balance(&platform_account);
            ensure!(platform_balance >= amount, Error::<T>::InsufficientBalance);

            // 先转账给提供者（失败则整个交易回滚）
            T::Currency::transfer(
                &platform_account,
                &who,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 转账成功后再扣除账面余额
            ProviderBalances::<T>::mutate(&who, |b| {
                *b = b.saturating_sub(amount);
            });

            let withdrawal_id = NextWithdrawalId::<T>::get();
            NextWithdrawalId::<T>::put(withdrawal_id.saturating_add(1));

            let withdrawal = WithdrawalRequest {
                id: withdrawal_id,
                provider: who.clone(),
                amount,
                status: WithdrawalStatus::Completed,
                requested_at: <frame_system::Pallet<T>>::block_number(),
                processed_at: Some(<frame_system::Pallet<T>>::block_number()),
            };

            Withdrawals::<T>::insert(withdrawal_id, withdrawal);

            Self::deposit_event(Event::WithdrawalRequested {
                withdrawal_id,
                provider: who,
                amount,
            });

            Self::deposit_event(Event::WithdrawalCompleted { withdrawal_id });

            Ok(())
        }

        /// 取消订单（仅限未接单状态）
        #[pallet::call_index(17)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn cancel_order(origin: OriginFor<T>, order_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.customer == who, Error::<T>::NotOrderOwner);
            ensure!(
                order.status == OrderStatus::Paid,
                Error::<T>::InvalidOrderStatus
            );

            // 退款
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &who,
                order.amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Cancelled;
                }
            });

            Self::deposit_event(Event::OrderCancelled { order_id });

            Ok(())
        }

        // ==================== 悬赏问答可调用函数 ====================

        /// 创建悬赏问题
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 关联的占卜结果 ID（可选）
        /// - `question_cid`: 问题描述 IPFS CID
        /// - `bounty_amount`: 悬赏金额
        /// - `deadline`: 截止区块
        /// - `min_answers`: 最小回答数
        /// - `max_answers`: 最大回答数
        /// - `specialty`: 擅长领域（可选）
        /// - `certified_only`: 是否仅限认证提供者回答
        /// - `allow_voting`: 是否允许社区投票
        #[pallet::call_index(18)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn create_bounty(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            question_cid: Vec<u8>,
            bounty_amount: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
            min_answers: u8,
            max_answers: u8,
            specialty: Option<Specialty>,
            certified_only: bool,
            allow_voting: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证悬赏金额
            ensure!(
                bounty_amount >= T::MinServicePrice::get(),
                Error::<T>::BountyAmountTooLow
            );

            // 验证截止时间
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(deadline > current_block, Error::<T>::InvalidBountyDeadline);

            // 验证占卜结果存在（悬赏必须基于已存在的占卜结果）
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证调用者是占卜结果的创建者
            let result_creator = T::DivinationProvider::result_creator(divination_type, result_id)
                .ok_or(Error::<T>::DivinationResultNotFound)?;
            ensure!(result_creator == who, Error::<T>::NotResultCreator);

            let question_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(question_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            // 转账悬赏金到平台账户托管
            T::Currency::transfer(
                &who,
                &T::PlatformAccount::get(),
                bounty_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            let bounty_id = NextBountyId::<T>::get();
            NextBountyId::<T>::put(bounty_id.saturating_add(1));

            // 🆕 自动 Pin 悬赏问题到 IPFS (Temporary 层级)
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                bounty_id,
                question_cid,
                pallet_storage_service::PinTier::Temporary,
            )?;

            let bounty = BountyQuestion {
                id: bounty_id,
                creator: who.clone(),
                divination_type,
                result_id,
                question_cid: question_cid_bounded,
                bounty_amount,
                deadline,
                min_answers,
                max_answers,
                status: BountyStatus::Open,
                adopted_answer_id: None,
                second_place_id: None,
                third_place_id: None,
                answer_count: 0,
                reward_distribution: RewardDistribution::default(),
                created_at: current_block,
                closed_at: None,
                settled_at: None,
                specialty,
                certified_only,
                allow_voting,
                total_votes: 0,
            };

            BountyQuestions::<T>::insert(bounty_id, bounty);

            // 更新用户悬赏索引
            UserBounties::<T>::try_mutate(&who, |list| {
                list.try_push(bounty_id)
                    .map_err(|_| Error::<T>::BountyListFull)
            })?;

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.total_bounties += 1;
                s.active_bounties += 1;
                s.total_bounty_amount = s.total_bounty_amount.saturating_add(bounty_amount);
            });

            Self::deposit_event(Event::BountyCreated {
                bounty_id,
                creator: who,
                divination_type,
                bounty_amount,
                deadline,
            });

            Ok(())
        }

        /// 提交悬赏回答
        ///
        /// # 参数
        /// - `bounty_id`: 悬赏问题 ID
        /// - `answer_cid`: 回答内容 IPFS CID
        #[pallet::call_index(19)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn submit_bounty_answer(
            origin: OriginFor<T>,
            bounty_id: u64,
            answer_cid: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            // 验证状态
            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyNotOpen);

            // 验证截止时间
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(
                current_block <= bounty.deadline,
                Error::<T>::BountyDeadlinePassed
            );

            // 不能回答自己的悬赏
            ensure!(who != bounty.creator, Error::<T>::CannotAnswerOwnBounty);

            // 验证回答数量限制
            ensure!(
                bounty.answer_count < bounty.max_answers as u32,
                Error::<T>::BountyAnswerLimitReached
            );

            // 检查是否已回答
            let answer_ids = BountyAnswerIds::<T>::get(bounty_id);
            for answer_id in answer_ids.iter() {
                if let Some(ans) = BountyAnswers::<T>::get(answer_id) {
                    ensure!(ans.answerer != who, Error::<T>::AlreadyAnswered);
                }
            }

            // 检查认证要求
            let (is_certified, provider_tier) = if bounty.certified_only {
                let provider =
                    Providers::<T>::get(&who).ok_or(Error::<T>::CertifiedProviderOnly)?;
                ensure!(
                    provider.tier as u8 >= ProviderTier::Certified as u8,
                    Error::<T>::CertifiedProviderOnly
                );
                (true, Some(provider.tier))
            } else {
                // 非强制认证时，检查是否为提供者
                if let Some(provider) = Providers::<T>::get(&who) {
                    (provider.tier as u8 >= ProviderTier::Certified as u8, Some(provider.tier))
                } else {
                    (false, None)
                }
            };

            let answer_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(answer_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            let answer_id = NextBountyAnswerId::<T>::get();
            NextBountyAnswerId::<T>::put(answer_id.saturating_add(1));

            // 🆕 自动 Pin 悬赏回答到 IPFS (Standard 层级)
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-market".to_vec(),
                answer_id,
                answer_cid,
                pallet_storage_service::PinTier::Standard,
            )?;

            let answer = BountyAnswer {
                id: answer_id,
                bounty_id,
                answerer: who.clone(),
                answer_cid: answer_cid_bounded,
                status: BountyAnswerStatus::Pending,
                votes: 0,
                reward_amount: Zero::zero(),
                submitted_at: current_block,
                is_certified,
                provider_tier,
            };

            BountyAnswers::<T>::insert(answer_id, answer);

            // 更新悬赏回答索引
            BountyAnswerIds::<T>::try_mutate(bounty_id, |list| {
                list.try_push(answer_id)
                    .map_err(|_| Error::<T>::BountyAnswerLimitReached)
            })?;

            // 更新用户回答索引
            UserBountyAnswers::<T>::try_mutate(&who, |list| {
                list.try_push(answer_id)
                    .map_err(|_| Error::<T>::BountyListFull)
            })?;

            // 更新悬赏回答数
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.answer_count += 1;
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.total_answers += 1;
            });

            Self::deposit_event(Event::BountyAnswerSubmitted {
                answer_id,
                bounty_id,
                answerer: who,
            });

            Ok(())
        }

        /// 关闭悬赏（停止接受回答）
        ///
        /// 仅悬赏创建者可调用，需要达到最小回答数
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn close_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            BountyQuestions::<T>::try_mutate(bounty_id, |maybe_bounty| {
                let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

                ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
                ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);
                ensure!(
                    bounty.answer_count >= bounty.min_answers as u32,
                    Error::<T>::NotEnoughAnswers
                );

                bounty.status = BountyStatus::Closed;
                bounty.closed_at = Some(<frame_system::Pallet<T>>::block_number());

                Ok::<_, DispatchError>(())
            })?;

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.active_bounties = s.active_bounties.saturating_sub(1);
            });

            Self::deposit_event(Event::BountyClosed { bounty_id });

            Ok(())
        }

        /// 投票支持回答
        ///
        /// 任何人都可以投票（如果悬赏允许投票）
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn vote_bounty_answer(
            origin: OriginFor<T>,
            bounty_id: u64,
            answer_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            // 验证投票功能已开启
            ensure!(bounty.allow_voting, Error::<T>::VotingNotAllowed);

            // 验证状态：Open 或 Closed 时可投票
            ensure!(
                bounty.status == BountyStatus::Open || bounty.status == BountyStatus::Closed,
                Error::<T>::BountyAlreadyAdopted
            );

            // 验证答案存在且属于该悬赏
            let answer = BountyAnswers::<T>::get(answer_id).ok_or(Error::<T>::BountyAnswerNotFound)?;
            ensure!(answer.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);

            // 检查是否已投票
            ensure!(
                !BountyVotes::<T>::contains_key(bounty_id, &who),
                Error::<T>::AlreadyVoted
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            // 记录投票
            let vote = BountyVote {
                voter: who.clone(),
                answer_id,
                voted_at: current_block,
            };
            BountyVotes::<T>::insert(bounty_id, &who, vote);

            // 更新答案票数
            BountyAnswers::<T>::mutate(answer_id, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.votes += 1;
                }
            });

            // 更新悬赏总票数
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.total_votes += 1;
                }
            });

            Self::deposit_event(Event::BountyAnswerVoted {
                bounty_id,
                answer_id,
                voter: who,
            });

            Ok(())
        }

        /// 采纳答案（选择前三名）
        ///
        /// 仅悬赏创建者可调用
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn adopt_bounty_answers(
            origin: OriginFor<T>,
            bounty_id: u64,
            first_place: u64,
            second_place: Option<u64>,
            third_place: Option<u64>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            BountyQuestions::<T>::try_mutate(bounty_id, |maybe_bounty| {
                let bounty = maybe_bounty.as_mut().ok_or(Error::<T>::BountyNotFound)?;

                ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
                ensure!(
                    bounty.status == BountyStatus::Open || bounty.status == BountyStatus::Closed,
                    Error::<T>::BountyAlreadyAdopted
                );
                ensure!(bounty.answer_count >= 1, Error::<T>::NotEnoughAnswers);

                // 验证第一名答案
                let first_ans = BountyAnswers::<T>::get(first_place)
                    .ok_or(Error::<T>::BountyAnswerNotFound)?;
                ensure!(first_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);

                // 验证第二名答案（如果提供）
                if let Some(second_id) = second_place {
                    let second_ans = BountyAnswers::<T>::get(second_id)
                        .ok_or(Error::<T>::BountyAnswerNotFound)?;
                    ensure!(second_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);
                }

                // 验证第三名答案（如果提供）
                if let Some(third_id) = third_place {
                    let third_ans = BountyAnswers::<T>::get(third_id)
                        .ok_or(Error::<T>::BountyAnswerNotFound)?;
                    ensure!(third_ans.bounty_id == bounty_id, Error::<T>::BountyAnswerNotFound);
                }

                bounty.status = BountyStatus::Adopted;
                bounty.adopted_answer_id = Some(first_place);
                bounty.second_place_id = second_place;
                bounty.third_place_id = third_place;

                Ok::<_, DispatchError>(())
            })?;

            // 更新答案状态
            BountyAnswers::<T>::mutate(first_place, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.status = BountyAnswerStatus::Adopted;
                }
            });

            if let Some(second_id) = second_place {
                BountyAnswers::<T>::mutate(second_id, |maybe_answer| {
                    if let Some(a) = maybe_answer {
                        a.status = BountyAnswerStatus::Selected;
                    }
                });
            }

            if let Some(third_id) = third_place {
                BountyAnswers::<T>::mutate(third_id, |maybe_answer| {
                    if let Some(a) = maybe_answer {
                        a.status = BountyAnswerStatus::Selected;
                    }
                });
            }

            Self::deposit_event(Event::BountyAnswersAdopted {
                bounty_id,
                first_place,
                second_place,
                third_place,
            });

            Ok(())
        }

        /// 结算悬赏奖励（方案B - 多人奖励）
        ///
        /// 采纳后由任何人调用执行奖励分配
        #[pallet::call_index(23)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn settle_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(
                bounty.status == BountyStatus::Adopted,
                Error::<T>::BountyNotAdopted
            );

            let first_place_id = bounty.adopted_answer_id.ok_or(Error::<T>::NotEnoughAnswers)?;

            // 计算奖励金额
            let dist = bounty.reward_distribution;
            let total = bounty.bounty_amount;
            let answer_count = bounty.answer_count;

            // 计算各名次奖励
            let first_reward = total.saturating_mul(dist.first_place.into()) / 10000u32.into();
            let second_reward = total.saturating_mul(dist.second_place.into()) / 10000u32.into();
            let third_reward = total.saturating_mul(dist.third_place.into()) / 10000u32.into();
            let platform_fee = total.saturating_mul(dist.platform_fee.into()) / 10000u32.into();
            let participation_pool =
                total.saturating_mul(dist.participation_pool.into()) / 10000u32.into();

            // 发放第一名奖励
            let first_ans =
                BountyAnswers::<T>::get(first_place_id).ok_or(Error::<T>::BountyAnswerNotFound)?;
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &first_ans.answerer,
                first_reward,
                ExistenceRequirement::KeepAlive,
            )?;
            BountyAnswers::<T>::mutate(first_place_id, |maybe_answer| {
                if let Some(a) = maybe_answer {
                    a.reward_amount = first_reward;
                }
            });
            Self::deposit_event(Event::BountyRewardPaid {
                bounty_id,
                recipient: first_ans.answerer.clone(),
                amount: first_reward,
                rank: 1,
            });

            let mut distributed = first_reward;

            // 发放第二名奖励
            if let Some(second_id) = bounty.second_place_id {
                if let Some(second_ans) = BountyAnswers::<T>::get(second_id) {
                    T::Currency::transfer(
                        &T::PlatformAccount::get(),
                        &second_ans.answerer,
                        second_reward,
                        ExistenceRequirement::KeepAlive,
                    )?;
                    BountyAnswers::<T>::mutate(second_id, |maybe_answer| {
                        if let Some(a) = maybe_answer {
                            a.reward_amount = second_reward;
                        }
                    });
                    Self::deposit_event(Event::BountyRewardPaid {
                        bounty_id,
                        recipient: second_ans.answerer,
                        amount: second_reward,
                        rank: 2,
                    });
                    distributed = distributed.saturating_add(second_reward);
                }
            }

            // 发放第三名奖励
            if let Some(third_id) = bounty.third_place_id {
                if let Some(third_ans) = BountyAnswers::<T>::get(third_id) {
                    T::Currency::transfer(
                        &T::PlatformAccount::get(),
                        &third_ans.answerer,
                        third_reward,
                        ExistenceRequirement::KeepAlive,
                    )?;
                    BountyAnswers::<T>::mutate(third_id, |maybe_answer| {
                        if let Some(a) = maybe_answer {
                            a.reward_amount = third_reward;
                        }
                    });
                    Self::deposit_event(Event::BountyRewardPaid {
                        bounty_id,
                        recipient: third_ans.answerer,
                        amount: third_reward,
                        rank: 3,
                    });
                    distributed = distributed.saturating_add(third_reward);
                }
            }

            // 计算并发放参与奖
            let top_three = [
                bounty.adopted_answer_id,
                bounty.second_place_id,
                bounty.third_place_id,
            ];
            let answer_ids = BountyAnswerIds::<T>::get(bounty_id);
            let other_participants: Vec<_> = answer_ids
                .iter()
                .filter(|id| !top_three.contains(&Some(**id)))
                .collect();

            let other_count = other_participants.len() as u32;
            if other_count > 0 {
                let per_participant = participation_pool / other_count.into();
                for answer_id in other_participants {
                    if let Some(ans) = BountyAnswers::<T>::get(answer_id) {
                        T::Currency::transfer(
                            &T::PlatformAccount::get(),
                            &ans.answerer,
                            per_participant,
                            ExistenceRequirement::KeepAlive,
                        )?;
                        BountyAnswers::<T>::mutate(answer_id, |maybe_answer| {
                            if let Some(a) = maybe_answer {
                                a.status = BountyAnswerStatus::Participated;
                                a.reward_amount = per_participant;
                            }
                        });
                        Self::deposit_event(Event::BountyRewardPaid {
                            bounty_id,
                            recipient: ans.answerer,
                            amount: per_participant,
                            rank: 0,
                        });
                        distributed = distributed.saturating_add(per_participant);
                    }
                }
            }

            // 平台手续费保留在平台账户（无需转账）
            distributed = distributed.saturating_add(platform_fee);

            // 更新悬赏状态
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.status = BountyStatus::Settled;
                    b.settled_at = Some(<frame_system::Pallet<T>>::block_number());
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.settled_bounties += 1;
                s.total_rewards_paid = s.total_rewards_paid.saturating_add(distributed);
                // 更新平均回答数
                if s.settled_bounties > 0 {
                    s.avg_answers_per_bounty =
                        ((s.total_answers as u64 * 100) / s.settled_bounties) as u16;
                }
            });

            Self::deposit_event(Event::BountySettled {
                bounty_id,
                total_distributed: distributed,
                platform_fee,
                participant_count: answer_count,
            });

            Ok(())
        }

        /// 取消悬赏（仅限无回答时）
        #[pallet::call_index(24)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn cancel_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.creator == who, Error::<T>::NotBountyCreator);
            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);
            ensure!(bounty.answer_count == 0, Error::<T>::BountyCannotCancel);

            // 退款
            T::Currency::transfer(
                &T::PlatformAccount::get(),
                &who,
                bounty.bounty_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 更新状态
            BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                if let Some(b) = maybe_bounty {
                    b.status = BountyStatus::Cancelled;
                }
            });

            // 更新统计
            BountyStatistics::<T>::mutate(|s| {
                s.active_bounties = s.active_bounties.saturating_sub(1);
            });

            Self::deposit_event(Event::BountyCancelled {
                bounty_id,
                refund_amount: bounty.bounty_amount,
            });

            Ok(())
        }

        /// 处理过期悬赏（任何人可调用）
        ///
        /// 超过截止时间且无人回答的悬赏可退款
        #[pallet::call_index(25)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn expire_bounty(origin: OriginFor<T>, bounty_id: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let bounty = BountyQuestions::<T>::get(bounty_id).ok_or(Error::<T>::BountyNotFound)?;

            ensure!(bounty.status == BountyStatus::Open, Error::<T>::BountyAlreadyClosed);

            // 验证已过期
            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block > bounty.deadline, Error::<T>::BountyNotExpired);

            // 如果有回答，不能简单过期处理，需要创建者采纳
            if bounty.answer_count > 0 {
                // 仅关闭，等待创建者采纳
                BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                    if let Some(b) = maybe_bounty {
                        b.status = BountyStatus::Closed;
                        b.closed_at = Some(current_block);
                    }
                });

                BountyStatistics::<T>::mutate(|s| {
                    s.active_bounties = s.active_bounties.saturating_sub(1);
                });

                Self::deposit_event(Event::BountyClosed { bounty_id });
            } else {
                // 无回答，退款并标记过期
                T::Currency::transfer(
                    &T::PlatformAccount::get(),
                    &bounty.creator,
                    bounty.bounty_amount,
                    ExistenceRequirement::KeepAlive,
                )?;

                BountyQuestions::<T>::mutate(bounty_id, |maybe_bounty| {
                    if let Some(b) = maybe_bounty {
                        b.status = BountyStatus::Expired;
                    }
                });

                BountyStatistics::<T>::mutate(|s| {
                    s.active_bounties = s.active_bounties.saturating_sub(1);
                });

                Self::deposit_event(Event::BountyExpired {
                    bounty_id,
                    refund_amount: bounty.bounty_amount,
                });
            }

            Ok(())
        }

        // ==================== 个人主页管理函数 ====================

        /// 更新提供者详细资料
        ///
        /// # 参数
        /// - `introduction_cid`: 详细自我介绍 IPFS CID
        /// - `experience_years`: 从业年限
        /// - `background`: 师承/学习背景
        /// - `motto`: 服务理念/座右铭
        /// - `expertise_description`: 擅长问题类型描述
        /// - `working_hours`: 工作时间说明
        /// - `avg_response_time`: 平均响应时间（分钟）
        /// - `accepts_appointment`: 是否接受预约
        /// - `banner_cid`: 主页背景图 CID
        #[pallet::call_index(26)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn update_profile(
            origin: OriginFor<T>,
            introduction_cid: Option<Vec<u8>>,
            experience_years: Option<u8>,
            background: Option<Vec<u8>>,
            motto: Option<Vec<u8>>,
            expertise_description: Option<Vec<u8>>,
            working_hours: Option<Vec<u8>>,
            avg_response_time: Option<u32>,
            accepts_appointment: Option<bool>,
            banner_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是注册的提供者
            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            // 🆕 如果有详细介绍 CID，先 Pin 到 IPFS (Standard 层级)
            if let Some(ref cid) = introduction_cid {
                // 使用 provider 账户地址编码的前8字节作为 subject_id
                let subject_id = who.using_encoded(|bytes| {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
                    u64::from_le_bytes(arr)
                });

                <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-market".to_vec(),
                    subject_id,
                    cid.clone(),
                    pallet_storage_service::PinTier::Standard,
                )?;
            }

            // 🆕 如果有背景图 CID，也 Pin 到 IPFS (Standard 层级)
            if let Some(ref cid) = banner_cid {
                let subject_id = who.using_encoded(|bytes| {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes[..8.min(bytes.len())]);
                    u64::from_le_bytes(arr)
                });

                <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-market".to_vec(),
                    subject_id,
                    cid.clone(),
                    pallet_storage_service::PinTier::Standard,
                )?;
            }

            let current_block = <frame_system::Pallet<T>>::block_number();

            ProviderProfiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = maybe_profile.get_or_insert_with(|| ProviderProfile {
                    introduction_cid: None,
                    experience_years: 0,
                    background: None,
                    motto: None,
                    expertise_description: None,
                    working_hours: None,
                    avg_response_time: None,
                    accepts_appointment: false,
                    banner_cid: None,
                    updated_at: current_block,
                });

                if let Some(cid) = introduction_cid {
                    profile.introduction_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }
                if let Some(years) = experience_years {
                    profile.experience_years = years;
                }
                if let Some(bg) = background {
                    profile.background = Some(
                        BoundedVec::try_from(bg).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(m) = motto {
                    profile.motto = Some(
                        BoundedVec::try_from(m).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(exp) = expertise_description {
                    profile.expertise_description = Some(
                        BoundedVec::try_from(exp).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(wh) = working_hours {
                    profile.working_hours = Some(
                        BoundedVec::try_from(wh).map_err(|_| Error::<T>::DescriptionTooLong)?
                    );
                }
                if let Some(time) = avg_response_time {
                    profile.avg_response_time = Some(time);
                }
                if let Some(accepts) = accepts_appointment {
                    profile.accepts_appointment = accepts;
                }
                if let Some(cid) = banner_cid {
                    profile.banner_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }

                profile.updated_at = current_block;

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ProfileUpdated { provider: who });

            Ok(())
        }

        /// 添加资质证书
        #[pallet::call_index(27)]
        #[pallet::weight(Weight::from_parts(35_000_000, 0))]
        pub fn add_certificate(
            origin: OriginFor<T>,
            name: Vec<u8>,
            cert_type: CertificateType,
            issuer: Option<Vec<u8>>,
            image_cid: Vec<u8>,
            issued_at: Option<BlockNumberFor<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let cert_id = NextCertificateId::<T>::get(&who);
            // 限制每个提供者最多 20 个证书
            ensure!(cert_id < 20, Error::<T>::TooManyCertificates);

            let name_bounded = BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;
            let image_cid_bounded = BoundedVec::try_from(image_cid).map_err(|_| Error::<T>::CidTooLong)?;
            let issuer_bounded = issuer
                .map(|i| BoundedVec::try_from(i).map_err(|_| Error::<T>::NameTooLong))
                .transpose()?;

            let certificate = Certificate {
                id: cert_id,
                name: name_bounded,
                cert_type,
                issuer: issuer_bounded,
                image_cid: image_cid_bounded,
                issued_at,
                is_verified: false,
                uploaded_at: <frame_system::Pallet<T>>::block_number(),
            };

            Certificates::<T>::insert(&who, cert_id, certificate);
            NextCertificateId::<T>::insert(&who, cert_id.saturating_add(1));

            Self::deposit_event(Event::CertificateAdded {
                provider: who,
                certificate_id: cert_id,
            });

            Ok(())
        }

        /// 删除资质证书
        #[pallet::call_index(28)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_certificate(
            origin: OriginFor<T>,
            certificate_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Certificates::<T>::contains_key(&who, certificate_id),
                Error::<T>::CertificateNotFound
            );

            Certificates::<T>::remove(&who, certificate_id);

            Self::deposit_event(Event::CertificateRemoved {
                provider: who,
                certificate_id,
            });

            Ok(())
        }

        /// 验证资质证书（治理权限）
        #[pallet::call_index(29)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn verify_certificate(
            origin: OriginFor<T>,
            provider: T::AccountId,
            certificate_id: u32,
            is_verified: bool,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Certificates::<T>::try_mutate(&provider, certificate_id, |maybe_cert| {
                let cert = maybe_cert.as_mut().ok_or(Error::<T>::CertificateNotFound)?;
                cert.is_verified = is_verified;
                Ok::<_, DispatchError>(())
            })?;

            // 更新信用档案中的认证数
            if is_verified {
                CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.certification_count = profile.certification_count.saturating_add(1);
                    }
                });
            }

            Self::deposit_event(Event::CertificateVerified {
                provider,
                certificate_id,
                is_verified,
            });

            Ok(())
        }

        /// 发布作品/案例
        #[pallet::call_index(30)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn publish_portfolio(
            origin: OriginFor<T>,
            title: Vec<u8>,
            divination_type: DivinationType,
            case_type: PortfolioCaseType,
            content_cid: Vec<u8>,
            cover_cid: Option<Vec<u8>>,
            is_featured: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let portfolio_id = NextPortfolioId::<T>::get(&who);
            // 限制每个提供者最多 100 个作品
            ensure!(portfolio_id < 100, Error::<T>::TooManyPortfolios);

            let title_bounded = BoundedVec::try_from(title).map_err(|_| Error::<T>::NameTooLong)?;
            let content_cid_bounded = BoundedVec::try_from(content_cid).map_err(|_| Error::<T>::CidTooLong)?;
            let cover_cid_bounded = cover_cid
                .map(|c| BoundedVec::try_from(c).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let portfolio = PortfolioItem {
                id: portfolio_id,
                title: title_bounded,
                divination_type,
                case_type,
                content_cid: content_cid_bounded,
                cover_cid: cover_cid_bounded,
                is_featured,
                view_count: 0,
                like_count: 0,
                published_at: <frame_system::Pallet<T>>::block_number(),
            };

            Portfolios::<T>::insert(&who, portfolio_id, portfolio);
            NextPortfolioId::<T>::insert(&who, portfolio_id.saturating_add(1));

            Self::deposit_event(Event::PortfolioPublished {
                provider: who,
                portfolio_id,
                divination_type,
            });

            Ok(())
        }

        /// 更新作品
        #[pallet::call_index(31)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_portfolio(
            origin: OriginFor<T>,
            portfolio_id: u32,
            title: Option<Vec<u8>>,
            content_cid: Option<Vec<u8>>,
            cover_cid: Option<Vec<u8>>,
            is_featured: Option<bool>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Portfolios::<T>::try_mutate(&who, portfolio_id, |maybe_portfolio| {
                let portfolio = maybe_portfolio.as_mut().ok_or(Error::<T>::PortfolioNotFound)?;

                if let Some(t) = title {
                    portfolio.title = BoundedVec::try_from(t).map_err(|_| Error::<T>::NameTooLong)?;
                }
                if let Some(cid) = content_cid {
                    portfolio.content_cid = BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?;
                }
                if let Some(cid) = cover_cid {
                    portfolio.cover_cid = Some(
                        BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong)?
                    );
                }
                if let Some(f) = is_featured {
                    portfolio.is_featured = f;
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::PortfolioUpdated {
                provider: who,
                portfolio_id,
            });

            Ok(())
        }

        /// 删除作品
        #[pallet::call_index(32)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn remove_portfolio(
            origin: OriginFor<T>,
            portfolio_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Portfolios::<T>::contains_key(&who, portfolio_id),
                Error::<T>::PortfolioNotFound
            );

            Portfolios::<T>::remove(&who, portfolio_id);

            Self::deposit_event(Event::PortfolioRemoved {
                provider: who,
                portfolio_id,
            });

            Ok(())
        }

        /// 点赞作品
        #[pallet::call_index(33)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn like_portfolio(
            origin: OriginFor<T>,
            provider: T::AccountId,
            portfolio_id: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证作品存在
            ensure!(
                Portfolios::<T>::contains_key(&provider, portfolio_id),
                Error::<T>::PortfolioNotFound
            );

            // 检查是否已点赞
            let key = (provider.clone(), portfolio_id);
            ensure!(
                !PortfolioLikes::<T>::get(&key, &who),
                Error::<T>::AlreadyLiked
            );

            // 记录点赞
            PortfolioLikes::<T>::insert(&key, &who, true);

            // 更新点赞数
            Portfolios::<T>::mutate(&provider, portfolio_id, |maybe_portfolio| {
                if let Some(p) = maybe_portfolio {
                    p.like_count = p.like_count.saturating_add(1);
                }
            });

            Self::deposit_event(Event::PortfolioLiked {
                provider,
                portfolio_id,
                liker: who,
            });

            Ok(())
        }

        /// 设置技能标签
        #[pallet::call_index(34)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn set_skill_tags(
            origin: OriginFor<T>,
            tags: Vec<(Vec<u8>, SkillTagType, u8)>, // (label, type, proficiency)
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            let mut skill_tags: BoundedVec<SkillTagOf, ConstU32<20>> = BoundedVec::new();

            for (label, tag_type, proficiency) in tags {
                ensure!(proficiency >= 1 && proficiency <= 5, Error::<T>::InvalidRating);

                let label_bounded = BoundedVec::try_from(label)
                    .map_err(|_| Error::<T>::NameTooLong)?;

                skill_tags.try_push(SkillTag {
                    label: label_bounded,
                    tag_type,
                    proficiency,
                }).map_err(|_| Error::<T>::TooManyTags)?;
            }

            SkillTags::<T>::insert(&who, skill_tags);

            Self::deposit_event(Event::SkillTagsUpdated { provider: who });

            Ok(())
        }

        // ==================== 信用体系管理函数 ====================

        /// 初始化提供者信用档案
        ///
        /// 在提供者注册时自动调用，也可手动为老用户创建
        #[pallet::call_index(35)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn init_credit_profile(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Providers::<T>::contains_key(&who),
                Error::<T>::ProviderNotFound
            );

            // 检查是否已有信用档案
            ensure!(
                !CreditProfiles::<T>::contains_key(&who),
                Error::<T>::ProviderAlreadyExists
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            let provider = Providers::<T>::get(&who).ok_or(Error::<T>::ProviderNotFound)?;

            // 创建初始信用档案，基础分 650
            let initial_score: u16 = 650;
            let profile = CreditProfile {
                score: initial_score,
                level: CreditLevel::from_score(initial_score),
                highest_score: initial_score,
                lowest_score: initial_score,
                service_quality_score: 0,
                avg_overall_rating: 0,
                avg_accuracy_rating: 0,
                avg_attitude_rating: 0,
                avg_response_rating: 0,
                five_star_count: 0,
                one_star_count: 0,
                behavior_score: 250, // 满分
                violation_count: 0,
                warning_count: 0,
                complaint_count: 0,
                complaint_upheld_count: 0,
                active_violations: 0,
                fulfillment_score: 0,
                completion_rate: 10000, // 100%
                on_time_rate: 10000,
                cancellation_rate: 0,
                timeout_count: 0,
                active_cancel_count: 0,
                avg_response_blocks: 0,
                bonus_score: 0,
                bounty_adoption_count: 0,
                certification_count: 0,
                consecutive_positive_days: 0,
                is_verified: false,
                has_deposit: !provider.deposit.is_zero(),
                total_deductions: 0,
                last_deduction_reason: None,
                last_deduction_at: None,
                total_orders: provider.total_orders,
                completed_orders: provider.completed_orders,
                total_reviews: provider.total_ratings,
                created_at: current_block,
                updated_at: current_block,
                last_evaluated_at: current_block,
            };

            CreditProfiles::<T>::insert(&who, profile);

            // 更新全局统计
            CreditStatistics::<T>::mutate(|stats| {
                stats.total_providers = stats.total_providers.saturating_add(1);
                stats.fair_count = stats.fair_count.saturating_add(1);
            });

            Self::deposit_event(Event::CreditProfileCreated { provider: who });

            Ok(())
        }

        /// 记录违规（治理权限）
        #[pallet::call_index(36)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn record_violation(
            origin: OriginFor<T>,
            provider: T::AccountId,
            violation_type: ViolationType,
            reason: Vec<u8>,
            related_order_id: Option<u64>,
            penalty: PenaltyType,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            ensure!(
                Providers::<T>::contains_key(&provider),
                Error::<T>::ProviderNotFound
            );

            // 检查是否在黑名单中
            ensure!(
                !CreditBlacklist::<T>::contains_key(&provider),
                Error::<T>::InBlacklist
            );

            let reason_bounded = BoundedVec::try_from(reason)
                .map_err(|_| Error::<T>::DescriptionTooLong)?;

            let violation_id = NextViolationId::<T>::get();
            NextViolationId::<T>::put(violation_id.saturating_add(1));

            let current_block = <frame_system::Pallet<T>>::block_number();
            let duration = violation_type.record_duration();
            let expires_at = if duration > 0 {
                Some(current_block + duration.into())
            } else {
                None
            };

            // 计算扣分
            let base_deduction: u16 = match &penalty {
                PenaltyType::DeductionOnly => 20,
                PenaltyType::Warning => 30,
                PenaltyType::OrderRestriction => 50,
                PenaltyType::ServiceSuspension => 100,
                PenaltyType::PermanentBan => 500,
            };
            let deduction_points = (base_deduction as u32 * violation_type.penalty_multiplier() as u32 / 100) as u16;

            let record = ViolationRecord {
                id: violation_id,
                provider: provider.clone(),
                violation_type,
                reason: reason_bounded.clone(),
                related_order_id,
                deduction_points,
                penalty,
                penalty_duration: duration,
                is_appealed: false,
                appeal_result: None,
                recorded_at: current_block,
                expires_at,
                is_active: true,
            };

            ViolationRecords::<T>::insert(violation_id, record);

            // 更新提供者违规索引
            ProviderViolations::<T>::try_mutate(&provider, |list| {
                list.try_push(violation_id)
                    .map_err(|_| Error::<T>::TooManyViolations)
            })?;

            // 更新信用档案
            CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.violation_count = profile.violation_count.saturating_add(1);
                    profile.active_violations = profile.active_violations.saturating_add(1);
                    profile.total_deductions = profile.total_deductions.saturating_add(deduction_points);
                    profile.last_deduction_reason = Some(DeductionReason::Violation);
                    profile.last_deduction_at = Some(current_block);

                    // 重新计算分数
                    let new_score = profile.score.saturating_sub(deduction_points);
                    let old_level = profile.level;
                    let new_level = CreditLevel::from_score(new_score);

                    profile.score = new_score;
                    profile.level = new_level;
                    if new_score < profile.lowest_score {
                        profile.lowest_score = new_score;
                    }
                    profile.updated_at = current_block;

                    // 如果等级变更，发送事件
                    if old_level != new_level {
                        Self::deposit_event(Event::CreditLevelChanged {
                            provider: provider.clone(),
                            old_level,
                            new_level,
                        });
                    }
                }
            });

            // 根据处罚类型扣除保证金（非封禁情况）
            let deposit_slash_bps: u16 = match &penalty {
                PenaltyType::DeductionOnly => 0,      // 0%
                PenaltyType::Warning => 500,          // 5%
                PenaltyType::OrderRestriction => 1000, // 10%
                PenaltyType::ServiceSuspension => 2000, // 20%
                PenaltyType::PermanentBan => 10000,   // 100% (在下面单独处理)
            };

            if deposit_slash_bps > 0 && penalty != PenaltyType::PermanentBan {
                if let Some(p) = Providers::<T>::get(&provider) {
                    if !p.deposit.is_zero() {
                        // 计算扣除金额
                        let slash_amount = p.deposit
                            .saturating_mul(deposit_slash_bps.into())
                            / 10000u32.into();
                        
                        if !slash_amount.is_zero() {
                            // 解除锁定
                            T::Currency::unreserve(&provider, slash_amount);
                            
                            // 根据是否有关联订单决定资金流向
                            let (to_customer, target) = if let Some(order_id) = related_order_id {
                                if let Some(order) = Orders::<T>::get(order_id) {
                                    (true, order.customer)
                                } else {
                                    (false, T::TreasuryAccount::get())
                                }
                            } else {
                                (false, T::TreasuryAccount::get())
                            };
                            
                            let _ = T::Currency::transfer(
                                &provider,
                                &target,
                                slash_amount,
                                ExistenceRequirement::AllowDeath,
                            );
                            
                            // 更新提供者保证金
                            let new_deposit = p.deposit.saturating_sub(slash_amount);
                            Providers::<T>::mutate(&provider, |maybe_p| {
                                if let Some(prov) = maybe_p {
                                    prov.deposit = new_deposit;
                                    
                                    // 如果保证金低于最低要求，自动暂停服务
                                    let min_deposit = T::MinDeposit::get();
                                    if new_deposit < min_deposit && prov.status == ProviderStatus::Active {
                                        prov.status = ProviderStatus::Paused;
                                    }
                                }
                            });
                            
                            Self::deposit_event(Event::ProviderDepositSlashed {
                                provider: provider.clone(),
                                order_id: related_order_id.unwrap_or(0),
                                amount: slash_amount,
                                to_customer,
                            });
                            
                            // 检查保证金是否不足并发出警告
                            let min_deposit = T::MinDeposit::get();
                            if new_deposit < min_deposit {
                                Self::deposit_event(Event::ProviderDepositInsufficient {
                                    provider: provider.clone(),
                                    current: new_deposit,
                                    required: min_deposit,
                                });
                                
                                // 更新统计
                                MarketStatistics::<T>::mutate(|s| {
                                    s.active_providers = s.active_providers.saturating_sub(1);
                                });
                                
                                Self::deposit_event(Event::ProviderPaused { provider: provider.clone() });
                            }
                        }
                    }
                }
            }

            // 处理永久封禁
            if penalty == PenaltyType::PermanentBan {
                CreditBlacklist::<T>::insert(&provider, current_block);

                // 扣除保证金并转入国库
                if let Some(p) = Providers::<T>::get(&provider) {
                    if !p.deposit.is_zero() {
                        // 解除锁定
                        T::Currency::unreserve(&provider, p.deposit);
                        // 转入国库
                        let treasury = T::TreasuryAccount::get();
                        let _ = T::Currency::transfer(
                            &provider,
                            &treasury,
                            p.deposit,
                            ExistenceRequirement::AllowDeath,
                        );
                        
                        Self::deposit_event(Event::ProviderDepositSlashed {
                            provider: provider.clone(),
                            order_id: 0, // 封禁时无关联订单
                            amount: p.deposit,
                            to_customer: false,
                        });
                    }
                }

                // 更新提供者状态
                Providers::<T>::mutate(&provider, |maybe_p| {
                    if let Some(p) = maybe_p {
                        p.status = ProviderStatus::Banned;
                        p.deposit = Zero::zero();
                    }
                });

                CreditStatistics::<T>::mutate(|stats| {
                    stats.blacklisted_count = stats.blacklisted_count.saturating_add(1);
                });

                // 转换 reason 类型
                let ban_reason: BoundedVec<u8, ConstU32<128>> = reason_bounded
                    .clone()
                    .into_inner()
                    .try_into()
                    .unwrap_or_default();
                Self::deposit_event(Event::ProviderBanned {
                    provider: provider.clone(),
                    reason: ban_reason,
                });
                Self::deposit_event(Event::AddedToBlacklist { provider: provider.clone() });
            }

            Self::deposit_event(Event::ViolationRecorded {
                provider,
                violation_id,
                violation_type,
                penalty,
                deduction_points,
            });

            Ok(())
        }

        /// 申诉违规（提供者调用）
        #[pallet::call_index(37)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn appeal_violation(
            origin: OriginFor<T>,
            violation_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ViolationRecords::<T>::try_mutate(violation_id, |maybe_record| {
                let record = maybe_record.as_mut()
                    .ok_or(Error::<T>::ViolationNotFound)?;

                ensure!(record.provider == who, Error::<T>::NotViolationOwner);
                ensure!(!record.is_appealed, Error::<T>::AlreadyAppealed);
                ensure!(record.is_active, Error::<T>::ViolationExpired);

                record.is_appealed = true;

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ViolationAppealed {
                provider: who,
                violation_id,
            });

            Ok(())
        }

        /// 处理申诉（治理权限）
        #[pallet::call_index(38)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn resolve_appeal(
            origin: OriginFor<T>,
            violation_id: u64,
            result: AppealResult,
            restore_points: Option<u16>,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let record = ViolationRecords::<T>::get(violation_id)
                .ok_or(Error::<T>::ViolationNotFound)?;

            ensure!(record.is_appealed, Error::<T>::NotAppealed);

            let provider = record.provider.clone();
            let original_deduction = record.deduction_points;

            // 更新违规记录
            ViolationRecords::<T>::mutate(violation_id, |maybe_record| {
                if let Some(r) = maybe_record {
                    r.appeal_result = Some(result);
                    if result == AppealResult::Upheld {
                        r.is_active = false;
                    }
                }
            });

            // 根据申诉结果恢复信用分
            let points_to_restore = match result {
                AppealResult::Upheld => original_deduction,
                AppealResult::PartiallyUpheld => restore_points.unwrap_or(original_deduction / 2),
                AppealResult::Rejected => 0,
            };

            if points_to_restore > 0 {
                CreditProfiles::<T>::mutate(&provider, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.total_deductions = profile.total_deductions.saturating_sub(points_to_restore);

                        if result == AppealResult::Upheld {
                            profile.violation_count = profile.violation_count.saturating_sub(1);
                            profile.active_violations = profile.active_violations.saturating_sub(1);
                        }

                        let new_score = profile.score.saturating_add(points_to_restore).min(1000);
                        let old_level = profile.level;
                        let new_level = CreditLevel::from_score(new_score);

                        profile.score = new_score;
                        profile.level = new_level;
                        if new_score > profile.highest_score {
                            profile.highest_score = new_score;
                        }
                        profile.updated_at = <frame_system::Pallet<T>>::block_number();

                        if old_level != new_level {
                            Self::deposit_event(Event::CreditLevelChanged {
                                provider: provider.clone(),
                                old_level,
                                new_level,
                            });
                        }
                    }
                });
            }

            Self::deposit_event(Event::AppealResolved {
                provider,
                violation_id,
                result,
                restored_points: points_to_restore,
            });

            Ok(())
        }

        /// 申请信用修复任务
        #[pallet::call_index(39)]
        #[pallet::weight(Weight::from_parts(40_000_000, 0))]
        pub fn request_credit_repair(
            origin: OriginFor<T>,
            task_type: RepairTaskType,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = CreditProfiles::<T>::get(&who)
                .ok_or(Error::<T>::CreditProfileNotFound)?;

            // 只有信用分低于 750 的用户才能申请修复
            ensure!(profile.score < 750, Error::<T>::CreditTooHighForRepair);

            // 检查是否已有相同类型的进行中任务
            let tasks = RepairTasks::<T>::get(&who);
            ensure!(
                !tasks.iter().any(|t| t.task_type == task_type && !t.is_completed),
                Error::<T>::DuplicateRepairTask
            );

            // 检查活跃任务数量上限
            ensure!(
                tasks.iter().filter(|t| !t.is_completed).count() < 3,
                Error::<T>::TooManyActiveTasks
            );

            let current_block = <frame_system::Pallet<T>>::block_number();
            let target_value = task_type.default_target();
            let duration = task_type.default_duration();

            let task_id = tasks.len() as u32;

            let task = CreditRepairTask {
                id: task_id,
                task_type,
                reward_points: task_type.default_reward(),
                target_value,
                current_progress: 0,
                is_completed: false,
                started_at: current_block,
                deadline: current_block + duration.into(),
                completed_at: None,
            };

            RepairTasks::<T>::try_mutate(&who, |tasks| {
                tasks.try_push(task)
                    .map_err(|_| Error::<T>::TooManyTasks)
            })?;

            Self::deposit_event(Event::CreditRepairRequested {
                provider: who,
                task_type,
                target_value,
            });

            Ok(())
        }

        // 注：举报功能已迁移到统一仲裁模块 (pallet-arbitration)
        // 使用 arbitration.file_complaint 替代原有的 submit_report 等函数
    }

    // ==================== 🆕 仲裁集成：保证金扣除接口 ====================

    impl<T: Config> Pallet<T> {
        /// 投诉裁决后扣除服务提供者保证金
        /// 
        /// ## 参数
        /// - `order_id`: 订单ID
        /// - `slash_bps`: 扣除比例（基点，5000 = 50%）
        /// - `to_customer`: 是否赔付给客户（true=赔付客户，false=进入国库）
        /// 
        /// ## 返回
        /// - `Ok(slashed_amount)`: 实际扣除金额
        /// - `Err(...)`: 订单不存在或提供者不存在
        pub fn slash_provider_deposit(
            order_id: u64,
            slash_bps: u16,
            to_customer: bool,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            let provider_account = order.provider.clone();
            let customer_account = order.customer.clone();
            
            // 获取提供者信息
            let provider = Providers::<T>::get(&provider_account)
                .ok_or(Error::<T>::ProviderNotFound)?;
            
            // 计算扣除金额
            let slash_amount = sp_runtime::Permill::from_parts((slash_bps as u32) * 100)
                .mul_floor(provider.deposit);
            
            if slash_amount.is_zero() {
                return Ok(Zero::zero());
            }
            
            // 从提供者保证金中扣除（unreserve 后转移）
            let actually_slashed = T::Currency::unreserve(&provider_account, slash_amount);
            
            if to_customer && !actually_slashed.is_zero() {
                // 赔付给客户
                let _ = T::Currency::transfer(
                    &provider_account,
                    &customer_account,
                    actually_slashed,
                    ExistenceRequirement::AllowDeath,
                );
            }
            // 如果不赔付客户，资金留在提供者账户（可由治理决定如何处理）
            
            // 更新提供者保证金记录
            Providers::<T>::mutate(&provider_account, |maybe_provider| {
                if let Some(p) = maybe_provider {
                    p.deposit = p.deposit.saturating_sub(actually_slashed);
                }
            });
            
            // 更新信用档案
            CreditProfiles::<T>::mutate(&provider_account, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    profile.complaint_count = profile.complaint_count.saturating_add(1);
                    profile.complaint_upheld_count = profile.complaint_upheld_count.saturating_add(1);
                    profile.total_deductions = profile.total_deductions.saturating_add(50); // 扣50分
                    profile.last_deduction_reason = Some(DeductionReason::Violation);
                    profile.last_deduction_at = Some(<frame_system::Pallet<T>>::block_number());
                    profile.score = profile.score.saturating_sub(50);
                }
            });
            
            Self::deposit_event(Event::ProviderDepositSlashed {
                provider: provider_account,
                order_id,
                amount: actually_slashed,
                to_customer,
            });
            
            Ok(actually_slashed)
        }
        
        /// 投诉裁决后退款给客户（从托管或提供者余额）
        /// 
        /// ## 参数
        /// - `order_id`: 订单ID
        /// - `refund_bps`: 退款比例（基点，10000 = 100%）
        pub fn refund_customer_on_complaint(
            order_id: u64,
            refund_bps: u16,
        ) -> DispatchResult {
            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            
            // 计算退款金额
            let refund_amount = sp_runtime::Permill::from_parts((refund_bps as u32) * 100)
                .mul_floor(order.amount);
            
            if refund_amount.is_zero() {
                return Ok(());
            }
            
            // 从提供者余额退款
            let provider_balance = ProviderBalances::<T>::get(&order.provider);
            let actual_refund = provider_balance.min(refund_amount);
            
            if !actual_refund.is_zero() {
                ProviderBalances::<T>::mutate(&order.provider, |balance| {
                    *balance = balance.saturating_sub(actual_refund);
                });
                
                T::Currency::transfer(
                    &order.provider,
                    &order.customer,
                    actual_refund,
                    ExistenceRequirement::AllowDeath,
                )?;
            }
            
            // 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.status = OrderStatus::Refunded;
                }
            });
            
            Self::deposit_event(Event::OrderRefundedOnComplaint {
                order_id,
                customer: order.customer,
                amount: actual_refund,
            });
            
            Ok(())
        }
    }

    // ==================== 🆕 存储膨胀防护：归档函数 ====================

    impl<T: Config> Pallet<T> {
        /// 归档已完成订单（保留完整订单数据，仅移动索引）
        /// 
        /// 新方案：订单数据永久保留在 Orders 存储中，仅将订单ID从活跃索引
        /// (CustomerOrders/ProviderOrders) 移至归档索引 
        /// (CustomerArchivedOrderIds/ProviderArchivedOrderIds)
        fn archive_completed_orders(max_count: u32) -> Weight {
            let mut cursor = ArchiveCursor::<T>::get();
            let next_id = NextOrderId::<T>::get();
            let mut processed = 0u32;

            // 7天后归档（区块数，假设6秒/块）
            const ARCHIVE_DELAY_BLOCKS: u32 = 7 * 24 * 60 * 10;
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(order) = Orders::<T>::get(cursor) {
                    // 检查是否为可归档状态（终态）
                    let is_final_state = matches!(
                        order.status,
                        OrderStatus::Completed | OrderStatus::Reviewed |
                        OrderStatus::Cancelled | OrderStatus::Refunded
                    );

                    if !is_final_state {
                        continue;
                    }

                    // 检查完成时间是否超过归档延迟
                    let completed_block: u32 = order.completed_at
                        .unwrap_or(order.created_at)
                        .saturated_into();
                    if current_block.saturating_sub(completed_block) < ARCHIVE_DELAY_BLOCKS {
                        continue;
                    }

                    // ========== 新方案：保留订单数据，仅移动索引 ==========
                    
                    // 1. 从活跃客户订单列表移除
                    CustomerOrders::<T>::mutate(&order.customer, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    // 2. 添加到客户归档订单列表（忽略溢出错误，继续处理）
                    let _ = CustomerArchivedOrderIds::<T>::try_mutate(&order.customer, |ids| {
                        ids.try_push(cursor)
                    });

                    // 3. 从活跃提供者订单列表移除
                    ProviderOrders::<T>::mutate(&order.provider, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    // 4. 添加到提供者归档订单列表
                    let _ = ProviderArchivedOrderIds::<T>::try_mutate(&order.provider, |ids| {
                        ids.try_push(cursor)
                    });

                    // 5. 更新永久统计
                    PermanentStats::<T>::mutate(|stats| {
                        stats.total_archived_orders = stats.total_archived_orders.saturating_add(1);
                        if matches!(order.status, OrderStatus::Completed | OrderStatus::Reviewed) {
                            stats.completed_orders = stats.completed_orders.saturating_add(1);
                            stats.total_volume = stats.total_volume.saturating_add(
                                order.amount.saturated_into::<u64>()
                            );
                        }
                        if let Some(rating) = order.rating {
                            stats.total_ratings = stats.total_ratings.saturating_add(rating as u64);
                            stats.rating_count = stats.rating_count.saturating_add(1);
                        }
                    });

                    // 注意：不删除 Orders::<T>::remove(cursor)，保留完整订单数据！

                    processed = processed.saturating_add(1);
                }
            }

            ArchiveCursor::<T>::put(cursor);
            Weight::from_parts(30_000 * processed as u64, 0)
        }

        /// 归档已结束悬赏（保留完整数据，仅移动索引）
        /// 
        /// 悬赏数据永久保留在 BountyQuestions/BountyAnswers 存储中，
        /// 仅将ID从活跃索引移至归档索引
        fn archive_completed_bounties(max_count: u32) -> Weight {
            let mut cursor = BountyArchiveCursor::<T>::get();
            let next_id = NextBountyId::<T>::get();
            let mut processed = 0u32;

            // 7天后归档（区块数，假设6秒/块）
            const ARCHIVE_DELAY_BLOCKS: u32 = 7 * 24 * 60 * 10;
            let current_block: u32 = <frame_system::Pallet<T>>::block_number().saturated_into();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(bounty) = BountyQuestions::<T>::get(cursor) {
                    // 检查是否为可归档状态（终态）
                    let is_final_state = matches!(
                        bounty.status,
                        BountyStatus::Settled | BountyStatus::Cancelled | BountyStatus::Expired
                    );

                    if !is_final_state {
                        continue;
                    }

                    // 检查结束时间是否超过归档延迟
                    let ended_block: u32 = bounty.deadline.saturated_into();
                    if current_block.saturating_sub(ended_block) < ARCHIVE_DELAY_BLOCKS {
                        continue;
                    }

                    // ========== 保留悬赏数据，仅移动索引 ==========
                    
                    // 1. 从活跃悬赏列表移除
                    UserBounties::<T>::mutate(&bounty.creator, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    // 2. 添加到归档悬赏列表
                    let _ = UserArchivedBounties::<T>::try_mutate(&bounty.creator, |ids| {
                        ids.try_push(cursor)
                    });

                    // 3. 归档该悬赏的所有回答
                    let answer_ids = BountyAnswerIds::<T>::get(cursor);
                    for answer_id in answer_ids.iter() {
                        if let Some(answer) = BountyAnswers::<T>::get(answer_id) {
                            // 从活跃回答列表移除
                            UserBountyAnswers::<T>::mutate(&answer.answerer, |ids| {
                                ids.retain(|&id| id != *answer_id);
                            });

                            // 添加到归档回答列表
                            let _ = UserArchivedBountyAnswers::<T>::try_mutate(&answer.answerer, |ids| {
                                ids.try_push(*answer_id)
                            });
                        }
                    }

                    // 注意：不删除 BountyQuestions/BountyAnswers，保留完整数据！

                    processed = processed.saturating_add(1);
                }
            }

            BountyArchiveCursor::<T>::put(cursor);
            Weight::from_parts(35_000 * processed as u64, 0)
        }
    }
}
