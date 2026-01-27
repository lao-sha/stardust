#![cfg_attr(not(feature = "std"), no_std)]

//! # Pallet Credit (信用管理整合模块)
//!
//! ## 函数级详细中文注释：统一的信用管理系统
//!
//! ### 概述
//!
//! 本模块整合了买家信用（Buyer Credit）和做市商信用（Maker Credit）两个子系统，
//! 提供统一的信用管理、评分计算和风控机制。
//!
//! ### 核心功能
//!
//! 1. **买家信用管理** (buyer.rs)
//!    - 多维度信任评估（资产、账户年龄、活跃度、社交）
//!    - 新用户分层冷启动（Premium/Standard/Basic/Restricted）
//!    - 信用等级体系（Newbie/Bronze/Silver/Gold/Diamond）
//!    - 快速学习机制（前3笔5x权重）
//!    - 社交信任网络（邀请人、推荐）
//!
//! 2. **做市商信用管理** (maker.rs)
//!    - 信用评分体系（800-1000分）
//!    - 五个等级（钻石/白金/黄金/白银/青铜）
//!    - 履约率追踪（完成率、及时释放率、超时率）
//!    - 违约惩罚机制
//!    - 动态保证金（信用高 → 保证金减50%）
//!    - 服务质量评价（买家评分）
//!    - 自动降级/禁用（< 750分 → 暂停）
//!
//! 3. **公共功能** (common.rs)
//!    - 信用分计算工具
//!    - 风险评估函数
//!    - 数据验证和校验

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

// 🆕 2026-01-18: 统一使用 pallet-trading-common 中的 MakerCreditInterface
// 旧的 MakerCreditInterface<AccountId> 定义已移除，统一到 common 模块
pub use pallet_trading_common::MakerCreditInterface;

// TODO: 测试文件待完善 mock 配置
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// 子模块
pub mod buyer;
pub mod maker;
pub mod common;
pub mod quota; // 🆕 方案C+：买家额度管理模块

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ReservableCurrency, Get},
        BoundedVec,
        weights::Weight,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::DispatchResult;
    use sp_runtime::traits::{Zero, Saturating, SaturatedConversion};
    
    // 导入子模块类型
    pub use crate::buyer;
    pub use crate::maker;
    pub use crate::common;
    pub use crate::quota; // 🆕 方案C+：买家额度管理

    // ===== 类型别名 =====
    
    /// 函数级详细中文注释：余额类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
    /// 函数级详细中文注释：时间戳类型别名
    pub type MomentOf<T> = <T as pallet_timestamp::Config>::Moment;

    // ===== 权重信息 Trait =====
    
    /// 函数级详细中文注释：Credit Pallet 权重信息 Trait
    pub trait CreditWeightInfo {
        // Buyer 模块权重
        fn initialize_buyer_credit() -> Weight;
        fn record_buyer_order_completed() -> Weight;
        fn record_buyer_order_failed() -> Weight;
        fn set_referrer() -> Weight;
        fn endorse_buyer() -> Weight;
        
        // Maker 模块权重
        fn initialize_maker_credit() -> Weight;
        fn record_maker_order_completed() -> Weight;
        fn record_maker_order_timeout() -> Weight;
        fn record_dispute_result() -> Weight;
        fn rate_maker() -> Weight;
    }
    
    impl CreditWeightInfo for () {
        fn initialize_buyer_credit() -> Weight { Weight::from_parts(10_000, 0) }
        fn record_buyer_order_completed() -> Weight { Weight::from_parts(20_000, 0) }
        fn record_buyer_order_failed() -> Weight { Weight::from_parts(20_000, 0) }
        fn set_referrer() -> Weight { Weight::from_parts(10_000, 0) }
        fn endorse_buyer() -> Weight { Weight::from_parts(15_000, 0) }
        fn initialize_maker_credit() -> Weight { Weight::from_parts(10_000, 0) }
        fn record_maker_order_completed() -> Weight { Weight::from_parts(20_000, 0) }
        fn record_maker_order_timeout() -> Weight { Weight::from_parts(25_000, 0) }
        fn record_dispute_result() -> Weight { Weight::from_parts(25_000, 0) }
        fn rate_maker() -> Weight { Weight::from_parts(15_000, 0) }
    }

    // ===== Config Trait =====
    
    /// 函数级详细中文注释：Credit Pallet 配置 Trait
    /// 
    /// 统一配置买家信用和做市商信用系统所需的参数
    #[pallet::config]
    /// 函数级中文注释：Credit Pallet 配置 trait
    /// - 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_timestamp::Config {
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        
        // 买家信用配置
        /// 买家初始信用分（0-1000，建议500）
        #[pallet::constant]
        type InitialBuyerCreditScore: Get<u16>;
        
        /// 订单完成信用分增加（建议10）
        #[pallet::constant]
        type OrderCompletedBonus: Get<u16>;
        
        /// 订单违约信用分扣除（建议50）
        #[pallet::constant]
        type OrderDefaultPenalty: Get<u16>;
        
        /// 每日区块数（用于计算日限额）
        #[pallet::constant]
        type BlocksPerDay: Get<BlockNumberFor<Self>>;
        
        /// 最小持仓量（用于计算资产信任）
        #[pallet::constant]
        type MinimumBalance: Get<BalanceOf<Self>>;
        
        // 做市商信用配置
        /// 做市商初始信用分（800-1000，建议820）
        #[pallet::constant]
        type InitialMakerCreditScore: Get<u16>;
        
        /// 订单按时完成信用分增加（建议2）
        #[pallet::constant]
        type MakerOrderCompletedBonus: Get<u16>;
        
        /// 订单超时信用分扣除（建议10）
        #[pallet::constant]
        type MakerOrderTimeoutPenalty: Get<u16>;
        
        /// 争议败诉信用分扣除（建议20）
        #[pallet::constant]
        type MakerDisputeLossPenalty: Get<u16>;
        
        /// 做市商服务暂停阈值（建议750）
        #[pallet::constant]
        type MakerSuspensionThreshold: Get<u16>;
        
        /// 做市商警告阈值（建议800）
        #[pallet::constant]
        type MakerWarningThreshold: Get<u16>;
        
        // 权重信息
        type CreditWeightInfo: CreditWeightInfo;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ===== 存储 =====
    
    // ===== Buyer 模块存储 =====
    
    /// 函数级详细中文注释：买家信用记录
    #[pallet::storage]
    #[pallet::getter(fn buyer_credit)]
    pub type BuyerCredits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        buyer::CreditScore<T>,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：买家每日交易量（用于限额控制）
    #[pallet::storage]
    pub type BuyerDailyVolume<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, u32, // 日期（天数）
        u64, // 当日累计金额（USDT，单位：美分）
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：买家订单历史（用于行为分析）
    #[pallet::storage]
    pub type BuyerOrderHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<buyer::OrderRecord, ConstU32<20>>, // 最近20笔订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：买家推荐人
    #[pallet::storage]
    pub type BuyerReferrer<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        T::AccountId,
        OptionQuery,
    >;
    
    /// 函数级详细中文注释：买家背书记录
    #[pallet::storage]
    pub type BuyerEndorsements<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<buyer::Endorsement<T>, ConstU32<10>>, // 最多10个背书人
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：转账计数（用于活跃度评估）
    #[pallet::storage]
    pub type TransferCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：违约历史记录（用于连续违约检测，最多保留50条）
    #[pallet::storage]
    pub type DefaultHistory<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<BlockNumberFor<T>, ConstU32<50>>,
        ValueQuery,
    >;
    
    // ===== Maker 模块存储 =====
    
    /// 函数级详细中文注释：做市商信用记录
    #[pallet::storage]
    #[pallet::getter(fn maker_credit)]
    pub type MakerCredits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // maker_id
        maker::CreditRecord<BlockNumberFor<T>>,
        OptionQuery,
    >;
    
    /// 函数级详细中文注释：做市商买家评分记录
    #[pallet::storage]
    pub type MakerRatings<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64, // maker_id
        Blake2_128Concat, u64, // order_id
        maker::Rating<T::AccountId>,
        OptionQuery,
    >;
    
    /// 函数级详细中文注释：做市商违约历史
    #[pallet::storage]
    pub type MakerDefaultHistory<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat, u64, // maker_id
        Blake2_128Concat, u64, // order_id
        maker::DefaultRecord<BlockNumberFor<T>>,
        OptionQuery,
    >;
    
    /// 函数级详细中文注释：做市商动态保证金要求
    #[pallet::storage]
    pub type MakerDynamicDeposit<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // maker_id
        BalanceOf<T>,
        ValueQuery,
    >;

    // ===== 🆕 方案C+：买家额度管理存储 =====

    /// 函数级详细中文注释：买家额度配置记录
    #[pallet::storage]
    #[pallet::getter(fn buyer_quota)]
    pub type BuyerQuotas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        quota::BuyerQuotaProfile<T>,
        ValueQuery,
    >;

    /// 函数级详细中文注释：买家违约记录历史（最多保留20条）
    #[pallet::storage]
    pub type BuyerViolations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<quota::ViolationRecord<T>, ConstU32<20>>,
        ValueQuery,
    >;

    /// 函数级详细中文注释：买家当前活跃订单列表
    #[pallet::storage]
    pub type BuyerActiveOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<10>>, // 最多10个并发订单
        ValueQuery,
    >;

    // ===== Event =====
    
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // ===== Buyer 模块事件 =====
        
        /// 函数级详细中文注释：新用户初始化
        /// [账户, 新用户等级代码(0=Premium,1=Standard,2=Basic,3=Restricted), 初始风险分]
        NewUserInitialized {
            account: T::AccountId,
            tier_code: u8,
            risk_score: u16,
        },
        
        /// 函数级详细中文注释：买家完成订单，信用分增加
        /// [账户, 新风险分, 新等级代码(0=Newbie,1=Bronze,2=Silver,3=Gold,4=Diamond)]
        BuyerCreditUpdated {
            account: T::AccountId,
            new_risk_score: u16,
            new_level_code: u8,
        },
        
        /// 函数级详细中文注释：买家等级升级
        /// [账户, 旧等级代码, 新等级代码(0=Newbie,1=Bronze,2=Silver,3=Gold,4=Diamond)]
        BuyerLevelUpgraded {
            account: T::AccountId,
            old_level_code: u8,
            new_level_code: u8,
        },
        
        /// 函数级详细中文注释：买家违约惩罚
        BuyerDefaultPenalty {
            account: T::AccountId,
            penalty: u16,
            consecutive_defaults: u32,
            new_risk_score: u16,
        },
        
        /// 函数级详细中文注释：连续违约检测到
        ConsecutiveDefaultDetected {
            account: T::AccountId,
            consecutive_count: u32,
            within_days: u32,
        },
        
        /// 函数级详细中文注释：用户被封禁
        UserBanned {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        },
        
        /// 函数级详细中文注释：用户推荐
        UserEndorsed {
            endorser: T::AccountId,
            endorsee: T::AccountId,
        },
        
        /// 函数级详细中文注释：设置邀请人
        ReferrerSet {
            invitee: T::AccountId,
            referrer: T::AccountId,
        },
        
        /// 函数级详细中文注释：行为模式识别
        /// [账户, 模式代码(0=HighQuality,1=Good,2=Normal,3=Suspicious,4=Insufficient), 调整分数]
        BehaviorPatternDetected {
            account: T::AccountId,
            pattern_code: u8,
            adjustment: i16,
        },
        
        /// 函数级详细中文注释：风险分自然衰减
        RiskScoreDecayed {
            account: T::AccountId,
            decay_amount: u16,
            new_risk_score: u16,
        },
        
        // ===== Maker 模块事件 =====
        
        /// 函数级详细中文注释：初始化做市商信用记录
        MakerCreditInitialized { maker_id: u64, initial_score: u16 },

        /// 函数级详细中文注释：订单完成，信用分增加
        MakerOrderCompleted {
            maker_id: u64,
            order_id: u64,
            new_score: u16,
            bonus: u16,
        },

        /// 函数级详细中文注释：订单超时，信用分减少
        MakerOrderTimeout {
            maker_id: u64,
            order_id: u64,
            new_score: u16,
            penalty: u16,
        },

        /// 函数级详细中文注释：争议解决，影响信用分
        MakerDisputeResolved {
            maker_id: u64,
            order_id: u64,
            maker_win: bool,
            new_score: u16,
        },

        /// 函数级详细中文注释：买家评价做市商
        MakerRated {
            maker_id: u64,
            order_id: u64,
            buyer: T::AccountId,
            stars: u8,
            new_score: u16,
        },

        /// 函数级详细中文注释：服务状态变更（0=Active, 1=Warning, 2=Suspended）
        MakerStatusChanged {
            maker_id: u64,
            old_status_code: u8,
            new_status_code: u8,
            credit_score: u16,
        },

        /// 函数级详细中文注释：信用等级变更（0=Diamond, 1=Platinum, 2=Gold, 3=Silver, 4=Bronze）
        MakerLevelChanged {
            maker_id: u64,
            old_level_code: u8,
            new_level_code: u8,
            credit_score: u16,
        },

        // ===== 🆕 方案C+：买家额度管理事件 =====

        /// 函数级详细中文注释：买家额度初始化
        BuyerQuotaInitialized {
            account: T::AccountId,
            initial_quota_usd: u64,
            credit_score: u16,
        },

        /// 函数级详细中文注释：占用额度（创建订单）
        QuotaOccupied {
            account: T::AccountId,
            order_id: u64,
            amount_usd: u64,
            remaining_quota: u64,
        },

        /// 函数级详细中文注释：释放额度（订单完成/取消）
        QuotaReleased {
            account: T::AccountId,
            order_id: u64,
            amount_usd: u64,
            new_available_quota: u64,
        },

        /// 函数级详细中文注释：额度提升（信用分提升或订单完成）
        QuotaIncreased {
            account: T::AccountId,
            old_max_quota: u64,
            new_max_quota: u64,
            reason: BoundedVec<u8, ConstU32<64>>,
        },

        /// 函数级详细中文注释：额度降低（违约惩罚）
        QuotaDecreased {
            account: T::AccountId,
            old_max_quota: u64,
            new_max_quota: u64,
            reduction_bps: u16,
            duration_days: u32,
        },

        /// 函数级详细中文注释：买家违约记录
        BuyerViolationRecorded {
            account: T::AccountId,
            violation_type: u8, // 0=Timeout, 1=DisputeLoss, 2=Malicious
            score_penalty: u16,
            new_credit_score: u16,
        },

        /// 函数级详细中文注释：买家服务暂停
        BuyerSuspended {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
            suspension_until: BlockNumberFor<T>,
        },

        /// 函数级详细中文注释：买家服务恢复
        BuyerReinstated {
            account: T::AccountId,
            new_credit_score: u16,
            new_max_quota: u64,
        },

        /// 函数级详细中文注释：买家被永久拉黑
        BuyerBlacklisted {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
            total_violations: u32,
        },

        /// 函数级详细中文注释：信用恢复（30天无违约或10单奖励）
        CreditRecovered {
            account: T::AccountId,
            recovery_points: u16,
            new_credit_score: u16,
            recovery_reason: u8, // 0=30DaysClean, 1=10OrdersBonus
        },

        /// 🆕 信用记录已清理（on_idle自动清理）
        CreditRecordsCleanedUp {
            processed_accounts: u32,
            cleaned_records: u32,
        },
    }

    // ===== Error =====
    
    #[pallet::error]
    pub enum Error<T> {
        // ===== Buyer 模块错误 =====
        
        /// 函数级详细中文注释：信用分过低（风险分 > 800）
        CreditScoreTooLow,
        /// 函数级详细中文注释：超过单笔限额
        ExceedSingleLimit,
        /// 函数级详细中文注释：超过每日限额
        ExceedDailyLimit,
        /// 函数级详细中文注释：新用户冷却期内不能交易
        InCooldownPeriod,
        /// 函数级详细中文注释：违约冷却期内不能交易
        InDefaultCooldown,
        /// 函数级详细中文注释：推荐人信用不足
        InsufficientCreditToEndorse,
        /// 函数级详细中文注释：不能推荐自己
        CannotEndorseSelf,
        /// 函数级详细中文注释：已经被推荐过
        AlreadyEndorsed,
        /// 函数级详细中文注释：邀请人已设置
        ReferrerAlreadySet,
        /// 函数级详细中文注释：不能邀请自己
        CannotReferSelf,
        
        // ===== Maker 模块错误 =====
        
        /// 函数级详细中文注释：做市商不存在
        MakerNotFound,
        /// 函数级详细中文注释：信用记录不存在
        CreditRecordNotFound,
        /// 函数级详细中文注释：评分超出范围（必须1-5）
        InvalidRating,
        /// 函数级详细中文注释：已评价过此订单
        AlreadyRated,
        /// 函数级详细中文注释：不是订单买家，无权评价
        NotOrderBuyer,
        /// 函数级详细中文注释：订单未完成，无法评价
        OrderNotCompleted,
        /// 函数级详细中文注释：服务已暂停
        ServiceSuspended,
        /// 函数级详细中文注释：信用分计算溢出
        ScoreOverflow,

        // ===== 🆕 方案C+：买家额度管理错误 =====

        /// 函数级详细中文注释：可用额度不足
        InsufficientQuota,
        /// 函数级详细中文注释：超过并发订单数限制
        ExceedConcurrentLimit,
        /// 函数级详细中文注释：买家已被暂停服务
        BuyerSuspended,
        /// 函数级详细中文注释：买家已被拉黑
        BuyerBlacklisted,
        /// 函数级详细中文注释：订单未找到（无法释放额度）
        OrderNotFoundForQuotaRelease,
        /// 函数级详细中文注释：额度配置未初始化
        QuotaProfileNotInitialized,
        /// 函数级详细中文注释：违约记录过多（达到上限20条）
        TooManyViolationRecords,
        /// 函数级详细中文注释：活跃订单列表已满（达到上限10个）
        ActiveOrderListFull,
    }

    // ===== Hooks =====
    
    /// 🆕 清理游标：用于追踪上次清理到哪个账户
    #[pallet::storage]
    pub type CleanupCursor<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 🆕 空闲时自动清理过期的订单历史和违约记录
        ///
        /// 清理策略：
        /// - BuyerOrderHistory：保留最近20条，删除超过90天的记录
        /// - BuyerViolations：保留最近20条，删除超过180天的记录
        /// - DefaultHistory：保留最近50条，删除超过365天的记录
        fn on_idle(now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = Weight::from_parts(30_000, 0);
            
            // 确保有足够权重执行清理
            if remaining_weight.ref_time() < base_weight.ref_time() * 5 {
                return Weight::zero();
            }
            
            Self::cleanup_expired_records(now, 5)
        }
    }

    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ===== Buyer 模块函数 =====
        
        /// 函数级详细中文注释：推荐用户（老用户为新用户担保）
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::CreditWeightInfo::endorse_buyer())]
        pub fn endorse_user(
            origin: OriginFor<T>,
            endorsee: T::AccountId,
        ) -> DispatchResult {
            let endorser = ensure_signed(origin)?;

            // 不能推荐自己
            ensure!(endorser != endorsee, Error::<T>::CannotEndorseSelf);

            // 检查推荐人信用
            let endorser_credit = BuyerCredits::<T>::get(&endorser);
            ensure!(
                endorser_credit.risk_score <= 300, // 风险分300以下才能推荐
                Error::<T>::InsufficientCreditToEndorse
            );

            // 检查是否已推荐
            let mut endorsements = BuyerEndorsements::<T>::get(&endorsee);
            ensure!(
                !endorsements.iter().any(|e| e.endorser == endorser),
                Error::<T>::AlreadyEndorsed
            );

            // 添加推荐记录
            let endorsement = buyer::Endorsement {
                endorser: endorser.clone(),
                endorsed_at: <frame_system::Pallet<T>>::block_number(),
                is_active: true,
            };

            endorsements.try_push(endorsement)
                .map_err(|_| Error::<T>::AlreadyEndorsed)?;

            BuyerEndorsements::<T>::insert(&endorsee, endorsements);

            Self::deposit_event(Event::UserEndorsed {
                endorser,
                endorsee,
            });

            Ok(())
        }

        /// 函数级详细中文注释：设置邀请人（仅能设置一次）
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::CreditWeightInfo::set_referrer())]
        pub fn set_referrer(
            origin: OriginFor<T>,
            referrer: T::AccountId,
        ) -> DispatchResult {
            let invitee = ensure_signed(origin)?;

            // 不能邀请自己
            ensure!(invitee != referrer, Error::<T>::CannotReferSelf);

            // 检查是否已设置
            ensure!(
                !BuyerReferrer::<T>::contains_key(&invitee),
                Error::<T>::ReferrerAlreadySet
            );

            BuyerReferrer::<T>::insert(&invitee, &referrer);

            Self::deposit_event(Event::ReferrerSet {
                invitee,
                referrer,
            });

            Ok(())
        }
        
        // ===== Maker 模块函数 =====
        
        /// 函数级详细中文注释：买家评价做市商
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::CreditWeightInfo::rate_maker())]
        pub fn rate_maker(
            origin: OriginFor<T>,
            maker_id: u64,
            order_id: u64,
            stars: u8,
            tags_codes: BoundedVec<u8, ConstU32<5>>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            // 验证评分范围
            ensure!(stars >= 1 && stars <= 5, Error::<T>::InvalidRating);

            // 检查是否已评价
            ensure!(
                !MakerRatings::<T>::contains_key(maker_id, order_id),
                Error::<T>::AlreadyRated
            );

            // 获取当前区块号
            let current_block = <frame_system::Pallet<T>>::block_number();
            let block_u32: u32 = current_block.saturated_into();

            // 存储评价记录
            let rating = maker::Rating {
                buyer: buyer.clone(),
                stars,
                tags_codes,
                rated_at: block_u32,
            };
            MakerRatings::<T>::insert(maker_id, order_id, rating);

            // 更新信用分
            let score_change = match stars {
                5 => 5i16,
                4 => 2i16,
                3 => 0i16,
                1 | 2 => -5i16,
                _ => 0i16,
            };

            let new_score = Self::update_maker_credit_score(maker_id, score_change)?;

            // 触发事件
            Self::deposit_event(Event::MakerRated {
                maker_id,
                order_id,
                buyer,
                stars,
                new_score,
            });

            Ok(())
        }
    }
    
    // ===== 内部辅助函数 =====
    
    impl<T: Config> Pallet<T> {
        // ===== Buyer 模块辅助函数 =====
        
        /// 函数级详细中文注释：计算资产信任度（0-100）
        pub fn calculate_asset_trust(account: &T::AccountId) -> u8 {
            let balance = T::Currency::free_balance(account);
            let min_balance = T::MinimumBalance::get();

            // 计算余额倍数
            let balance_multiplier = if min_balance.is_zero() {
                0u128
            } else {
                let balance_u128: u128 = balance.saturated_into();
                let min_u128: u128 = min_balance.saturated_into();
                balance_u128.checked_div(min_u128).unwrap_or(0)
            };

            // DUST 余额信任分
            let balance_score = if balance_multiplier >= 10000 {
                50  // >= 10000倍：高信任
            } else if balance_multiplier >= 1000 {
                30  // >= 1000倍：中等信任
            } else if balance_multiplier >= 100 {
                15  // >= 100倍：基础信任
            } else {
                0
            };

            // 预留余额加分
            let reserved = T::Currency::reserved_balance(account);
            let reserved_u128: u128 = reserved.saturated_into();
            let min_u128: u128 = min_balance.saturated_into();
            let reserved_score = if reserved_u128 > 0 && min_u128 > 0 {
                core::cmp::min(20, (reserved_u128 / min_u128) as u8 / 100)
            } else {
                0
            };

            balance_score + reserved_score
        }

        /// 函数级详细中文注释：计算账户年龄信任度（0-100）
        pub fn calculate_age_trust(account: &T::AccountId) -> u8 {
            let credit = BuyerCredits::<T>::get(account);
            if credit.account_created_at.is_zero() {
                return 0;
            }

            let current_block = <frame_system::Pallet<T>>::block_number();
            let age_blocks = current_block.saturating_sub(credit.account_created_at);
            
            let blocks_per_day = T::BlocksPerDay::get();
            let age_days = if blocks_per_day.is_zero() {
                0u32
            } else {
                let age_blocks_u32: u32 = age_blocks.saturated_into();
                let blocks_per_day_u32: u32 = blocks_per_day.saturated_into();
                age_blocks_u32 / blocks_per_day_u32
            };

            // 年龄信任分曲线
            if age_days >= 180 {
                100
            } else if age_days >= 90 {
                80
            } else if age_days >= 30 {
                50
            } else if age_days >= 7 {
                25
            } else {
                0
            }
        }

        /// 函数级详细中文注释：计算活跃度信任（0-100）
        pub fn calculate_activity_trust(account: &T::AccountId) -> u8 {
            let transfer_count = TransferCount::<T>::get(account);
            let score = core::cmp::min(40, transfer_count as u8 * 2);
            core::cmp::min(100, score)
        }

        /// 函数级详细中文注释：计算社交信任度（0-100）
        pub fn calculate_social_trust(account: &T::AccountId) -> u8 {
            let mut score = 0u8;

            // 1. 邀请人信誉
            if let Some(referrer) = BuyerReferrer::<T>::get(account) {
                let referrer_credit = BuyerCredits::<T>::get(&referrer);
                let referrer_risk = referrer_credit.risk_score;

                score += if referrer_risk <= 200 {
                    40
                } else if referrer_risk <= 400 {
                    25
                } else if referrer_risk <= 600 {
                    10
                } else {
                    0
                };
            }

            // 2. 被推荐次数
            let endorsements = BuyerEndorsements::<T>::get(account);
            let active_endorsements = endorsements.iter().filter(|e| e.is_active).count();
            score += core::cmp::min(30, active_endorsements as u8 * 10);

            core::cmp::min(100, score)
        }

        /// 函数级详细中文注释：计算新用户综合风险分（0-1000）
        pub fn calculate_new_user_risk_score(account: &T::AccountId) -> u16 {
            // 五个维度的信任分（每个 0-100）
            let asset_trust = Self::calculate_asset_trust(account);
            let age_trust = Self::calculate_age_trust(account);
            let activity_trust = Self::calculate_activity_trust(account);
            let social_trust = Self::calculate_social_trust(account);
            let identity_trust = 0u8;

            // 加权计算综合信任分（0-100）
            let weighted_trust = (
                asset_trust as u16 * 25 +      // 资产权重 25%
                age_trust as u16 * 20 +        // 年龄权重 20%
                activity_trust as u16 * 20 +   // 活跃度权重 20%
                social_trust as u16 * 20 +     // 社交权重 20%
                identity_trust as u16 * 15     // 身份权重 15%
            ) / 100;

            // 风险分 = 1000 - 综合信任分 * 10
            1000u16.saturating_sub(weighted_trust * 10)
        }

        /// 函数级详细中文注释：初始化新用户信用记录
        pub fn initialize_new_user_credit(account: &T::AccountId) -> buyer::NewUserTier {
            let risk_score = Self::calculate_new_user_risk_score(account);
            let tier = buyer::NewUserTier::from_risk_score(risk_score);

            let credit = buyer::CreditScore {
                level: buyer::CreditLevel::Newbie,
                new_user_tier: Some(tier.clone()),
                completed_orders: 0,
                total_volume: Zero::zero(),
                default_count: 0,
                dispute_count: 0,
                last_purchase_at: <frame_system::Pallet<T>>::block_number(),
                risk_score,
                account_created_at: <frame_system::Pallet<T>>::block_number(),
            };

            BuyerCredits::<T>::insert(account, credit);

            Self::deposit_event(Event::NewUserInitialized {
                account: account.clone(),
                tier_code: Self::new_user_tier_to_code(&tier),
                risk_score,
            });

            tier
        }
        
        /// 函数级详细中文注释：将 NewUserTier 转换为 u8
        fn new_user_tier_to_code(tier: &buyer::NewUserTier) -> u8 {
            match tier {
                buyer::NewUserTier::Premium => 0,
                buyer::NewUserTier::Standard => 1,
                buyer::NewUserTier::Basic => 2,
                buyer::NewUserTier::Restricted => 3,
            }
        }
        
        /// 函数级详细中文注释：将买家 CreditLevel 转换为 u8
        fn buyer_level_to_code(level: &buyer::CreditLevel) -> u8 {
            match level {
                buyer::CreditLevel::Newbie => 0,
                buyer::CreditLevel::Bronze => 1,
                buyer::CreditLevel::Silver => 2,
                buyer::CreditLevel::Gold => 3,
                buyer::CreditLevel::Diamond => 4,
            }
        }
        
        /// 函数级详细中文注释：将 BehaviorPattern 转换为 u8
        fn behavior_pattern_to_code(pattern: &buyer::BehaviorPattern) -> u8 {
            match pattern {
                buyer::BehaviorPattern::HighQuality => 0,
                buyer::BehaviorPattern::Good => 1,
                buyer::BehaviorPattern::Normal => 2,
                buyer::BehaviorPattern::Suspicious => 3,
                buyer::BehaviorPattern::Insufficient => 4,
            }
        }

        /// 函数级详细中文注释：获取当前日期key
        fn current_day_key() -> u32 {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let blocks_per_day = T::BlocksPerDay::get();
            if blocks_per_day.is_zero() {
                0
            } else {
                let current_u32: u32 = current_block.saturated_into();
                let day_u32: u32 = blocks_per_day.saturated_into();
                current_u32 / day_u32
            }
        }

        /// 函数级详细中文注释：检查买家是否可以创建订单
        pub fn check_buyer_limit(
            buyer: &T::AccountId,
            amount_usdt: u64,
        ) -> Result<(), Error<T>> {
            let mut credit = BuyerCredits::<T>::get(buyer);

            // 如果是新用户，先初始化
            if credit.account_created_at.is_zero() {
                let _tier = Self::initialize_new_user_credit(buyer);
                credit = BuyerCredits::<T>::get(buyer);
            }

            // 应用风险分自然衰减
            let decay_amount = Self::calculate_risk_decay(buyer);
            if decay_amount > 0 {
                let initial_risk = Self::get_initial_risk_score(buyer);
                let old_risk_score = credit.risk_score;
                
                credit.risk_score = credit.risk_score
                    .saturating_sub(decay_amount)
                    .max(initial_risk);
                
                if credit.risk_score != old_risk_score {
                    BuyerCredits::<T>::insert(buyer, &credit);
                    
                    Self::deposit_event(Event::RiskScoreDecayed {
                        account: buyer.clone(),
                        decay_amount: old_risk_score.saturating_sub(credit.risk_score),
                        new_risk_score: credit.risk_score,
                    });
                }
            }

            // 检查信用分
            ensure!(credit.risk_score <= 800, Error::<T>::CreditScoreTooLow);

            // 获取限额
            let (single_limit, daily_limit) = Self::get_effective_limits(&credit);

            // 首笔订单分层折扣
            let effective_single_limit = if credit.completed_orders == 0 {
                let discounted = single_limit / 10;
                core::cmp::max(discounted, 10)
            } else {
                single_limit
            };

            // 检查单笔限额
            ensure!(amount_usdt <= effective_single_limit, Error::<T>::ExceedSingleLimit);

            // 检查每日限额
            if daily_limit > 0 {
                let day_key = Self::current_day_key();
                let today_volume = BuyerDailyVolume::<T>::get(buyer, day_key);
                let new_volume = today_volume.saturating_add(amount_usdt);
                ensure!(new_volume <= daily_limit, Error::<T>::ExceedDailyLimit);
            }

            // 检查冷却期
            if let Some(ref tier) = credit.new_user_tier {
                let (_, _, cooldown_hours) = tier.get_limits();
                if cooldown_hours > 0 {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    let cooldown_blocks = T::BlocksPerDay::get().saturating_mul(cooldown_hours.into()) / 24u32.into();
                    let required_block = credit.last_purchase_at.saturating_add(cooldown_blocks);
                    ensure!(current_block >= required_block, Error::<T>::InCooldownPeriod);
                }
            }

            // 检查违约冷却期
            if credit.default_count > 0 {
                let cooldown_blocks = Self::calculate_cooldown_period(buyer);
                if !cooldown_blocks.is_zero() {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    let last_default_block = DefaultHistory::<T>::get(buyer)
                        .last()
                        .copied()
                        .unwrap_or(Zero::zero());
                    let required_block = last_default_block.saturating_add(cooldown_blocks);
                    
                    ensure!(current_block >= required_block, Error::<T>::InDefaultCooldown);
                }
            }

            Ok(())
        }

        /// 函数级详细中文注释：获取有效限额
        fn get_effective_limits(credit: &buyer::CreditScore<T>) -> (u64, u64) {
            if credit.completed_orders < 20 {
                if let Some(ref tier) = credit.new_user_tier {
                    let (single, daily, _) = tier.get_limits();
                    return (single, daily);
                }
            }
            credit.level.get_base_limits()
        }

        /// 函数级详细中文注释：统计近期违约次数
        fn count_recent_defaults(buyer: &T::AccountId, within_days: u32) -> u32 {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let blocks_per_day = T::BlocksPerDay::get();
            let within_blocks = blocks_per_day.saturating_mul(within_days.into());
            let cutoff_block = current_block.saturating_sub(within_blocks);
            
            DefaultHistory::<T>::get(buyer)
                .iter()
                .filter(|&&block| block >= cutoff_block)
                .count() as u32
        }

        /// 函数级详细中文注释：计算违约冷却期
        fn calculate_cooldown_period(buyer: &T::AccountId) -> BlockNumberFor<T> {
            let recent_defaults = Self::count_recent_defaults(buyer, 30);
            
            let cooldown_days: u32 = match recent_defaults {
                0 => 0,
                1 => 1,
                2 => 3,
                3 => 7,
                4 => 14,
                _ => 30,
            };
            
            T::BlocksPerDay::get().saturating_mul(cooldown_days.into())
        }

        /// 函数级详细中文注释：获取用户初始风险分
        fn get_initial_risk_score(buyer: &T::AccountId) -> u16 {
            Self::calculate_new_user_risk_score(buyer)
        }

        /// 函数级详细中文注释：计算风险分自然衰减量
        fn calculate_risk_decay(buyer: &T::AccountId) -> u16 {
            let credit = BuyerCredits::<T>::get(buyer);
            
            if credit.default_count == 0 {
                return 0;
            }
            
            let current_block = <frame_system::Pallet<T>>::block_number();
            let last_default_block = DefaultHistory::<T>::get(buyer)
                .last()
                .copied()
                .unwrap_or(Zero::zero());
            
            let blocks_since_last_default = current_block.saturating_sub(last_default_block);
            let blocks_per_30_days = T::BlocksPerDay::get().saturating_mul(30u32.into());
            
            let decay_cycles: u32 = (blocks_since_last_default / blocks_per_30_days).saturated_into();
            
            (decay_cycles as u16).saturating_mul(50)
        }

        /// 函数级详细中文注释：订单完成后更新信用
        pub fn update_credit_on_success(
            buyer: &T::AccountId,
            amount_usdt: u64,
            payment_time_seconds: u64,
        ) {
            BuyerCredits::<T>::mutate(buyer, |credit| {
                credit.completed_orders += 1;
                let order_index = credit.completed_orders;

                // 基础加分
                let base_score = 10u16;

                // 快速付款奖励
                let speed_bonus = if payment_time_seconds < 300 {
                    10
                } else if payment_time_seconds < 600 {
                    5
                } else {
                    0
                };

                // 大额交易奖励
                let amount_bonus = if amount_usdt > 1000 {
                    5
                } else {
                    0
                };

                // 应用权重系数
                let weight = buyer::get_order_weight(order_index);
                let weighted_score = (base_score + speed_bonus + amount_bonus) * (weight as u16) / 10;

                // 降低风险分
                credit.risk_score = credit.risk_score.saturating_sub(weighted_score);

                // 更新交易时间
                credit.last_purchase_at = <frame_system::Pallet<T>>::block_number();

                // 更新等级
                let old_level = credit.level.clone();
                credit.level = buyer::CreditLevel::from_completed_orders(credit.completed_orders);
                
                if credit.level != old_level {
                    Self::deposit_event(Event::BuyerLevelUpgraded {
                        account: buyer.clone(),
                        old_level_code: Self::buyer_level_to_code(&old_level),
                        new_level_code: Self::buyer_level_to_code(&credit.level),
                    });
                }

                // 超过20笔后移除新用户标记
                if credit.completed_orders > 20 {
                    credit.new_user_tier = None;
                }

                Self::deposit_event(Event::BuyerCreditUpdated {
                    account: buyer.clone(),
                    new_risk_score: credit.risk_score,
                    new_level_code: Self::buyer_level_to_code(&credit.level),
                });
            });

            // 更新每日购买量
            let day_key = Self::current_day_key();
            BuyerDailyVolume::<T>::mutate(buyer, day_key, |volume| {
                *volume = volume.saturating_add(amount_usdt);
            });

            // 记录订单历史
            let order_record = buyer::OrderRecord {
                amount_usdt,
                payment_time_seconds,
                created_at_block: Self::current_day_key(),
            };

            BuyerOrderHistory::<T>::mutate(buyer, |history| {
                if history.len() >= 20 {
                    history.remove(0);
                }
                let _ = history.try_push(order_record);
            });

            // 每5笔分析一次行为模式
            let credit = BuyerCredits::<T>::get(buyer);
            if credit.completed_orders % 5 == 0 && credit.completed_orders <= 20 {
                Self::analyze_and_adjust_behavior(buyer);
            }
        }

        /// 函数级详细中文注释：违约惩罚
        pub fn penalize_default(buyer: &T::AccountId) {
            let current_block = <frame_system::Pallet<T>>::block_number();
            DefaultHistory::<T>::mutate(buyer, |history| {
                if history.len() >= 50 {
                    history.remove(0);
                }
                let _ = history.try_push(current_block);
            });

            let consecutive_defaults = Self::count_recent_defaults(buyer, 7);

            BuyerCredits::<T>::mutate(buyer, |credit| {
                credit.default_count += 1;

                // 基础惩罚
                let base_penalty = match credit.level {
                    buyer::CreditLevel::Newbie => 50,
                    buyer::CreditLevel::Bronze => 30,
                    buyer::CreditLevel::Silver => 20,
                    buyer::CreditLevel::Gold => 10,
                    buyer::CreditLevel::Diamond => 5,
                };

                // 连续违约指数级惩罚
                let multiplier = match consecutive_defaults {
                    1 => 1,
                    2 => 2,
                    3 => 4,
                    4 => 8,
                    _ => 16,
                };

                let penalty = base_penalty.saturating_mul(multiplier);
                credit.risk_score = credit.risk_score.saturating_add(penalty);

                // 7天内连续违约 >= 3次，直接封禁
                if consecutive_defaults >= 3 {
                    credit.risk_score = 1000;

                    let reason: BoundedVec<u8, ConstU32<128>> = 
                        b"7 days consecutive 3 defaults".to_vec().try_into().unwrap_or_default();
                    Self::deposit_event(Event::UserBanned {
                        account: buyer.clone(),
                        reason,
                    });
                }

                Self::deposit_event(Event::BuyerDefaultPenalty {
                    account: buyer.clone(),
                    penalty,
                    consecutive_defaults,
                    new_risk_score: credit.risk_score,
                });
            });

            if consecutive_defaults >= 2 {
                Self::deposit_event(Event::ConsecutiveDefaultDetected {
                    account: buyer.clone(),
                    consecutive_count: consecutive_defaults,
                    within_days: 7,
                });
            }

            // 使所有推荐失效
            BuyerEndorsements::<T>::mutate(buyer, |endorsements| {
                for endorsement in endorsements.iter_mut() {
                    endorsement.is_active = false;
                    
                    // 推荐人也受连带责任
                    BuyerCredits::<T>::mutate(&endorsement.endorser, |endorser_credit| {
                        endorser_credit.risk_score = endorser_credit.risk_score.saturating_add(50);
                    });
                }
            });
        }

        /// 函数级详细中文注释：分析行为模式并调整风险分
        fn analyze_and_adjust_behavior(account: &T::AccountId) {
            let history = BuyerOrderHistory::<T>::get(account);
            
            if history.len() < 3 {
                return;
            }

            // 检查付款速度
            let avg_payment_time: u64 = history.iter()
                .map(|o| o.payment_time_seconds)
                .sum::<u64>() / history.len() as u64;
            
            let fast_payment = avg_payment_time < 600;

            // 检查金额稳定性
            let amounts: sp_std::vec::Vec<_> = history.iter().map(|o| o.amount_usdt).collect();
            let max_amount = *amounts.iter().max().unwrap_or(&0);
            let min_amount = *amounts.iter().min().unwrap_or(&1);
            let min_safe = if min_amount == 0 { 1 } else { min_amount };
            let is_consistent = max_amount / min_safe < 3;

            // 综合判断
            let (pattern, adjustment) = match (fast_payment, is_consistent) {
                (true, true) => (buyer::BehaviorPattern::HighQuality, -200i16),
                (true, false) | (false, true) => (buyer::BehaviorPattern::Good, -100i16),
                (false, false) => (buyer::BehaviorPattern::Normal, 0i16),
            };

            // 应用调整
            BuyerCredits::<T>::mutate(account, |credit| {
                if adjustment < 0 {
                    credit.risk_score = credit.risk_score.saturating_sub(adjustment.abs() as u16);
                } else {
                    credit.risk_score = credit.risk_score.saturating_add(adjustment as u16);
                }
            });

            Self::deposit_event(Event::BehaviorPatternDetected {
                account: account.clone(),
                pattern_code: Self::behavior_pattern_to_code(&pattern),
                adjustment,
            });
        }

        /// 函数级详细中文注释：记录转账（用于活跃度统计）
        pub fn record_transfer(account: &T::AccountId) {
            TransferCount::<T>::mutate(account, |count| {
                *count = count.saturating_add(1);
            });
        }
        
        // ===== Maker 模块辅助函数 =====
        
        /// 函数级详细中文注释：初始化做市商信用记录
        pub fn initialize_maker_credit(maker_id: u64) -> DispatchResult {
            let current_block = <frame_system::Pallet<T>>::block_number();

            let record = maker::CreditRecord {
                credit_score: T::InitialMakerCreditScore::get(),
                level: maker::CreditLevel::Bronze,
                status: maker::ServiceStatus::Active,
                total_orders: 0,
                completed_orders: 0,
                timeout_orders: 0,
                cancelled_orders: 0,
                timely_release_orders: 0,
                rating_sum: 0,
                rating_count: 0,
                avg_response_time: 0,
                default_count: 0,
                dispute_loss_count: 0,
                last_default_block: None,
                last_order_block: current_block,
                consecutive_days: 0,
            };

            MakerCredits::<T>::insert(maker_id, record);

            Self::deposit_event(Event::MakerCreditInitialized {
                maker_id,
                initial_score: T::InitialMakerCreditScore::get(),
            });

            Ok(())
        }

        /// 函数级详细中文注释：记录订单完成
        pub fn record_maker_order_completed(
            maker_id: u64,
            order_id: u64,
            response_time_seconds: u32,
        ) -> DispatchResult {
            MakerCredits::<T>::mutate(maker_id, |record_opt| {
                if let Some(record) = record_opt {
                    let current_block = <frame_system::Pallet<T>>::block_number();

                    // 更新履约数据
                    record.total_orders = record.total_orders.saturating_add(1);
                    record.completed_orders = record.completed_orders.saturating_add(1);
                    record.last_order_block = current_block;

                    // 计算奖励分数
                    let mut bonus: u16 = T::MakerOrderCompletedBonus::get();

                    // 及时释放奖励（< 24小时）
                    if response_time_seconds < 86400 {
                        record.timely_release_orders =
                            record.timely_release_orders.saturating_add(1);
                        bonus = bonus.saturating_add(1);
                    }

                    // 更新信用分
                    record.credit_score = record.credit_score.saturating_add(bonus);
                    if record.credit_score > 1000 {
                        record.credit_score = 1000;
                    }

                    // 更新等级和状态
                    let old_level = record.level.clone();
                    let old_status = record.status.clone();
                    Self::update_maker_level_and_status(record);

                    let new_score = record.credit_score;

                    // 触发等级变更事件
                    if old_level != record.level {
                        Self::deposit_event(Event::MakerLevelChanged {
                            maker_id,
                            old_level_code: Self::maker_level_to_code(&old_level),
                            new_level_code: Self::maker_level_to_code(&record.level),
                            credit_score: new_score,
                        });
                    }

                    // 触发状态变更事件
                    if old_status != record.status {
                        Self::deposit_event(Event::MakerStatusChanged {
                            maker_id,
                            old_status_code: Self::maker_status_to_code(&old_status),
                            new_status_code: Self::maker_status_to_code(&record.status),
                            credit_score: new_score,
                        });
                    }

                    // 触发订单完成事件
                    Self::deposit_event(Event::MakerOrderCompleted {
                        maker_id,
                        order_id,
                        new_score,
                        bonus,
                    });

                    Ok(())
                } else {
                    Err(Error::<T>::CreditRecordNotFound.into())
                }
            })
        }

        /// 函数级详细中文注释：记录订单超时
        pub fn record_maker_order_timeout(maker_id: u64, order_id: u64) -> DispatchResult {
            MakerCredits::<T>::mutate(maker_id, |record_opt| {
                if let Some(record) = record_opt {
                    let current_block = <frame_system::Pallet<T>>::block_number();

                    // 更新违约数据
                    record.total_orders = record.total_orders.saturating_add(1);
                    record.timeout_orders = record.timeout_orders.saturating_add(1);
                    record.default_count = record.default_count.saturating_add(1);
                    record.last_default_block = Some(current_block);

                    // 惩罚信用分
                    let penalty: u16 = T::MakerOrderTimeoutPenalty::get();
                    record.credit_score = record.credit_score.saturating_sub(penalty);

                    // 更新等级和状态
                    let old_level = record.level.clone();
                    let old_status = record.status.clone();
                    Self::update_maker_level_and_status(record);

                    let new_score = record.credit_score;

                    // 记录违约历史
                    let default_record = maker::DefaultRecord {
                        default_type: maker::DefaultType::Timeout,
                        block: current_block,
                        penalty_score: penalty,
                        recovered: false,
                    };
                    MakerDefaultHistory::<T>::insert(maker_id, order_id, default_record);

                    // 触发等级变更事件
                    if old_level != record.level {
                        Self::deposit_event(Event::MakerLevelChanged {
                            maker_id,
                            old_level_code: Self::maker_level_to_code(&old_level),
                            new_level_code: Self::maker_level_to_code(&record.level),
                            credit_score: new_score,
                        });
                    }

                    // 触发状态变更事件
                    if old_status != record.status {
                        Self::deposit_event(Event::MakerStatusChanged {
                            maker_id,
                            old_status_code: Self::maker_status_to_code(&old_status),
                            new_status_code: Self::maker_status_to_code(&record.status),
                            credit_score: new_score,
                        });
                    }

                    // 触发订单超时事件
                    Self::deposit_event(Event::MakerOrderTimeout {
                        maker_id,
                        order_id,
                        new_score,
                        penalty,
                    });

                    Ok(())
                } else {
                    Err(Error::<T>::CreditRecordNotFound.into())
                }
            })
        }

        /// 函数级详细中文注释：记录争议结果
        pub fn record_maker_dispute_result(
            maker_id: u64,
            order_id: u64,
            maker_win: bool,
        ) -> DispatchResult {
            if maker_win {
                // 做市商胜诉，无惩罚
                Self::deposit_event(Event::MakerDisputeResolved {
                    maker_id,
                    order_id,
                    maker_win: true,
                    new_score: Self::query_maker_credit_score(maker_id).unwrap_or(820),
                });
                return Ok(());
            }

            // 做市商败诉，扣分
            MakerCredits::<T>::mutate(maker_id, |record_opt| {
                if let Some(record) = record_opt {
                    let current_block = <frame_system::Pallet<T>>::block_number();

                    // 更新争议数据
                    record.dispute_loss_count = record.dispute_loss_count.saturating_add(1);
                    record.default_count = record.default_count.saturating_add(1);
                    record.last_default_block = Some(current_block);

                    // 严重惩罚
                    let penalty: u16 = T::MakerDisputeLossPenalty::get();
                    record.credit_score = record.credit_score.saturating_sub(penalty);

                    // 更新等级和状态
                    let old_level = record.level.clone();
                    let old_status = record.status.clone();
                    Self::update_maker_level_and_status(record);

                    let new_score = record.credit_score;

                    // 记录违约历史
                    let default_record = maker::DefaultRecord {
                        default_type: maker::DefaultType::DisputeLoss,
                        block: current_block,
                        penalty_score: penalty,
                        recovered: false,
                    };
                    MakerDefaultHistory::<T>::insert(maker_id, order_id, default_record);

                    // 触发等级变更事件
                    if old_level != record.level {
                        Self::deposit_event(Event::MakerLevelChanged {
                            maker_id,
                            old_level_code: Self::maker_level_to_code(&old_level),
                            new_level_code: Self::maker_level_to_code(&record.level),
                            credit_score: new_score,
                        });
                    }

                    // 触发状态变更事件
                    if old_status != record.status {
                        Self::deposit_event(Event::MakerStatusChanged {
                            maker_id,
                            old_status_code: Self::maker_status_to_code(&old_status),
                            new_status_code: Self::maker_status_to_code(&record.status),
                            credit_score: new_score,
                        });
                    }

                    // 触发争议解决事件
                    Self::deposit_event(Event::MakerDisputeResolved {
                        maker_id,
                        order_id,
                        maker_win: false,
                        new_score,
                    });

                    Ok(())
                } else {
                    Err(Error::<T>::CreditRecordNotFound.into())
                }
            })
        }

        /// 函数级详细中文注释：查询做市商信用分
        pub fn query_maker_credit_score(maker_id: u64) -> Option<u16> {
            MakerCredits::<T>::get(maker_id).map(|record| record.credit_score)
        }

        /// 函数级详细中文注释：检查服务状态
        pub fn check_maker_service_status(maker_id: u64) -> Result<maker::ServiceStatus, DispatchError> {
            MakerCredits::<T>::get(maker_id)
                .map(|record| record.status)
                .ok_or_else(|| Error::<T>::CreditRecordNotFound.into())
        }

        /// 函数级详细中文注释：计算动态保证金要求
        pub fn calculate_required_deposit(maker_id: u64) -> BalanceOf<T> {
            // 基础保证金：1,000,000 DUST
            let base_deposit = 1_000_000u128
                .checked_mul(1_000_000_000_000_000_000u128)
                .unwrap_or(1_000_000_000_000_000_000_000_000u128);

            let credit_score = Self::query_maker_credit_score(maker_id).unwrap_or(820);

            let multiplier_percent = match credit_score {
                950..=1000 => 50,  // Diamond: 0.5x
                900..=949 => 70,   // Platinum: 0.7x
                850..=899 => 80,   // Gold: 0.8x
                820..=849 => 90,   // Silver: 0.9x
                800..=819 => 100,  // Bronze: 1.0x
                750..=799 => 120,  // Warning: 1.2x
                _ => 200,          // Suspended: 2.0x
            };

            let required = base_deposit
                .checked_mul(multiplier_percent as u128)
                .and_then(|v| v.checked_div(100))
                .unwrap_or(base_deposit);

            required.try_into().unwrap_or(Zero::zero())
        }

        // ===== 辅助函数 =====

        /// 函数级详细中文注释：更新做市商信用分
        fn update_maker_credit_score(maker_id: u64, change: i16) -> Result<u16, DispatchError> {
            MakerCredits::<T>::mutate(maker_id, |record_opt| {
                if let Some(record) = record_opt {
                    let current_score = record.credit_score as i32;
                    let new_score_i32 = current_score + change as i32;

                    let new_score = if new_score_i32 < 0 {
                        0u16
                    } else if new_score_i32 > 1000 {
                        1000u16
                    } else {
                        new_score_i32 as u16
                    };

                    record.credit_score = new_score;

                    // 更新等级和状态
                    let old_level = record.level.clone();
                    let old_status = record.status.clone();
                    Self::update_maker_level_and_status(record);

                    // 触发等级变更事件
                    if old_level != record.level {
                        Self::deposit_event(Event::MakerLevelChanged {
                            maker_id,
                            old_level_code: Self::maker_level_to_code(&old_level),
                            new_level_code: Self::maker_level_to_code(&record.level),
                            credit_score: new_score,
                        });
                    }

                    // 触发状态变更事件
                    if old_status != record.status {
                        Self::deposit_event(Event::MakerStatusChanged {
                            maker_id,
                            old_status_code: Self::maker_status_to_code(&old_status),
                            new_status_code: Self::maker_status_to_code(&record.status),
                            credit_score: new_score,
                        });
                    }

                    Ok(new_score)
                } else {
                    Err(Error::<T>::CreditRecordNotFound.into())
                }
            })
        }

        /// 函数级详细中文注释：更新做市商信用等级和服务状态
        fn update_maker_level_and_status(record: &mut maker::CreditRecord<BlockNumberFor<T>>) {
            // 更新信用等级
            record.level = match record.credit_score {
                950..=1000 => maker::CreditLevel::Diamond,
                900..=949 => maker::CreditLevel::Platinum,
                850..=899 => maker::CreditLevel::Gold,
                820..=849 => maker::CreditLevel::Silver,
                _ => maker::CreditLevel::Bronze,
            };

            // 更新服务状态
            record.status = match record.credit_score {
                0..=749 => maker::ServiceStatus::Suspended,
                750..=799 => maker::ServiceStatus::Warning,
                _ => maker::ServiceStatus::Active,
            };
        }

        /// 函数级详细中文注释：将做市商 CreditLevel 转换为 u8
        fn maker_level_to_code(level: &maker::CreditLevel) -> u8 {
            match level {
                maker::CreditLevel::Diamond => 0,
                maker::CreditLevel::Platinum => 1,
                maker::CreditLevel::Gold => 2,
                maker::CreditLevel::Silver => 3,
                maker::CreditLevel::Bronze => 4,
            }
        }

        /// 函数级详细中文注释：将做市商 ServiceStatus 转换为 u8
        fn maker_status_to_code(status: &maker::ServiceStatus) -> u8 {
            match status {
                maker::ServiceStatus::Active => 0,
                maker::ServiceStatus::Warning => 1,
                maker::ServiceStatus::Suspended => 2,
            }
        }

        /// 🆕 清理过期的订单历史和违约记录
        ///
        /// 清理策略：
        /// - BuyerOrderHistory：保留90天内的记录
        /// - DefaultHistory：保留365天内的记录
        ///
        /// # 参数
        /// - `now`: 当前区块号
        /// - `max_accounts`: 每次最多处理的账户数
        ///
        /// # 返回
        /// - 消耗的权重
        pub fn cleanup_expired_records(now: BlockNumberFor<T>, max_accounts: u32) -> Weight {
            let mut cleaned_count = 0u32;
            let mut processed_count = 0u32;
            
            // 90天的区块数（假设6秒/块）
            let threshold_block_90: u32 = now.saturated_into::<u32>().saturating_sub(90 * 14400);
            
            // 遍历 BuyerOrderHistory，清理超过90天的记录
            for (account, mut history) in BuyerOrderHistory::<T>::iter().take(max_accounts as usize) {
                processed_count = processed_count.saturating_add(1);
                
                let original_len = history.len();
                
                // 只保留90天内的记录（使用 created_at_block 字段）
                history.retain(|record| record.created_at_block > threshold_block_90);
                
                // 如果记录被清理了，更新存储
                if history.len() < original_len {
                    if history.is_empty() {
                        BuyerOrderHistory::<T>::remove(&account);
                    } else {
                        BuyerOrderHistory::<T>::insert(&account, history);
                    }
                    cleaned_count = cleaned_count.saturating_add(1);
                }
            }
            
            // 遍历 DefaultHistory，清理超过365天的记录
            let threshold_block_365: BlockNumberFor<T> = now.saturating_sub((365u32 * 14400u32).into());
            
            for (account, mut history) in DefaultHistory::<T>::iter().take(max_accounts as usize) {
                let original_len = history.len();
                
                // 只保留365天内的记录（DefaultHistory 存储的是区块号）
                history.retain(|&block| block > threshold_block_365);
                
                if history.len() < original_len {
                    if history.is_empty() {
                        DefaultHistory::<T>::remove(&account);
                    } else {
                        DefaultHistory::<T>::insert(&account, history);
                    }
                    cleaned_count = cleaned_count.saturating_add(1);
                }
            }
            
            // 如果有清理发生，发出事件
            if cleaned_count > 0 {
                Self::deposit_event(Event::CreditRecordsCleanedUp {
                    processed_accounts: processed_count,
                    cleaned_records: cleaned_count,
                });
            }
            
            // 返回消耗的权重
            Weight::from_parts(
                (processed_count as u64) * 50_000 + (cleaned_count as u64) * 30_000 + 10_000,
                0
            )
        }
    }
}

// ===== Buyer Credit Interface =====

/// 函数级详细中文注释：买家信用接口
pub trait BuyerCreditInterface<AccountId> {
    fn get_buyer_credit_score(buyer: &AccountId) -> Result<u16, sp_runtime::DispatchError>;
    fn check_buyer_daily_limit(buyer: &AccountId, amount_usd_cents: u64) -> Result<(), sp_runtime::DispatchError>;
    fn check_buyer_single_limit(buyer: &AccountId, amount_usd_cents: u64) -> Result<(), sp_runtime::DispatchError>;
}

// ===== Maker Credit Interface (Legacy - 旧版接口，保留兼容性) =====

/// 函数级详细中文注释：做市商信用接口（旧版，基于 maker_id）
/// 
/// ⚠️ 此接口已被新版 MakerCreditInterface<AccountId> 替代
/// 保留此接口仅为兼容性，未来版本将移除
pub trait MakerCreditInterfaceLegacy {
    fn initialize_credit(maker_id: u64) -> sp_runtime::DispatchResult;
    fn check_service_status(maker_id: u64) -> Result<maker::ServiceStatus, sp_runtime::DispatchError>;
    fn record_order_completed(maker_id: u64, order_id: u64, response_time_seconds: u32) -> sp_runtime::DispatchResult;
    fn record_default_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult;
    fn record_default_dispute(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult;
}

/// 函数级详细中文注释：实现 MakerCreditInterfaceLegacy 用于其他 pallet 调用
impl<T: pallet::Config> MakerCreditInterfaceLegacy for pallet::Pallet<T> {
    fn initialize_credit(maker_id: u64) -> sp_runtime::DispatchResult {
        Self::initialize_maker_credit(maker_id)
    }
    
    fn check_service_status(maker_id: u64) -> Result<maker::ServiceStatus, sp_runtime::DispatchError> {
        Self::check_maker_service_status(maker_id)
    }
    
    fn record_order_completed(
        maker_id: u64,
        order_id: u64,
        response_time_seconds: u32,
    ) -> sp_runtime::DispatchResult {
        Self::record_maker_order_completed(maker_id, order_id, response_time_seconds)
    }
    
    fn record_default_timeout(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
        Self::record_maker_order_timeout(maker_id, order_id)
    }
    
    fn record_default_dispute(maker_id: u64, order_id: u64) -> sp_runtime::DispatchResult {
        Self::record_maker_dispute_result(maker_id, order_id, false)
    }
}

// ===== 🆕 2026-01-18: 统一 MakerCreditInterface 实现 =====

/// 函数级详细中文注释：为 Trading 模块实现统一的 MakerCreditInterface
/// 
/// 此实现提供了 OTC 和 Bridge 模块所需的做市商信用管理功能。
/// 使用 maker_id 直接标识做市商，无需 AccountId 映射。
impl<T: pallet::Config> pallet_trading_common::MakerCreditInterface for pallet::Pallet<T> {
    fn record_maker_order_completed(
        maker_id: u64,
        order_id: u64,
        response_time_seconds: u32,
    ) -> sp_runtime::DispatchResult {
        // 调用已有的做市商信用更新逻辑
        Self::record_maker_order_completed(maker_id, order_id, response_time_seconds)
    }
    
    fn record_maker_order_timeout(
        maker_id: u64,
        order_id: u64,
    ) -> sp_runtime::DispatchResult {
        // 调用已有的做市商信用更新逻辑
        Self::record_maker_order_timeout(maker_id, order_id)
    }
    
    fn record_maker_dispute_result(
        maker_id: u64,
        order_id: u64,
        maker_win: bool,
    ) -> sp_runtime::DispatchResult {
        // 调用已有的做市商信用更新逻辑
        Self::record_maker_dispute_result(maker_id, order_id, maker_win)
    }
}

// ===== 🆕 方案C+：买家额度管理接口实现 =====

/// 函数级详细中文注释：为OTC订单实现BuyerQuotaInterface
///
/// 这个实现提供了OTC订单所需的买家额度管理功能，
/// 包括额度占用、释放、违约记录等核心功能。
impl<T: pallet::Config> crate::quota::BuyerQuotaInterface<T::AccountId> for pallet::Pallet<T> {
    /// 函数级详细中文注释：获取可用额度
    fn get_available_quota(buyer: &T::AccountId) -> Result<u64, sp_runtime::DispatchError> {
        use frame_support::ensure;
        use pallet::{BuyerQuotas, Error};

        let profile = BuyerQuotas::<T>::get(buyer);

        // 检查是否被暂停或拉黑
        ensure!(!profile.is_suspended, Error::<T>::BuyerSuspended);
        ensure!(!profile.is_blacklisted, Error::<T>::BuyerBlacklisted);

        Ok(profile.available_quota)
    }

    /// 函数级详细中文注释：占用额度（创建订单时）
    fn occupy_quota(buyer: &T::AccountId, amount_usd: u64) -> sp_runtime::DispatchResult {
        use pallet::{BuyerQuotas, Error, Event};
        use frame_support::traits::Get;

        BuyerQuotas::<T>::try_mutate(buyer, |profile| -> sp_runtime::DispatchResult {
            use frame_support::ensure;
            // 检查是否被暂停或拉黑
            ensure!(!profile.is_suspended, Error::<T>::BuyerSuspended);
            ensure!(!profile.is_blacklisted, Error::<T>::BuyerBlacklisted);

            // 如果是新用户，初始化额度
            if profile.total_orders == 0 && profile.max_quota == 0 {
                profile.credit_score = T::InitialBuyerCreditScore::get();
                profile.max_quota = crate::quota::calculate_max_quota(
                    profile.credit_score,
                    profile.total_orders,
                );
                profile.available_quota = profile.max_quota;
                profile.max_concurrent_orders = crate::quota::calculate_max_concurrent(
                    profile.total_orders,
                );

                Self::deposit_event(Event::BuyerQuotaInitialized {
                    account: buyer.clone(),
                    initial_quota_usd: profile.max_quota,
                    credit_score: profile.credit_score,
                });
            }

            // 检查可用额度是否充足
            ensure!(
                profile.available_quota >= amount_usd,
                Error::<T>::InsufficientQuota
            );

            // 检查并发订单数限制
            ensure!(
                profile.active_orders < profile.max_concurrent_orders,
                Error::<T>::ExceedConcurrentLimit
            );

            // 占用额度
            profile.available_quota = profile.available_quota
                .checked_sub(amount_usd)
                .ok_or(Error::<T>::InsufficientQuota)?;
            profile.occupied_quota = profile.occupied_quota
                .checked_add(amount_usd)
                .ok_or(Error::<T>::ScoreOverflow)?;
            profile.active_orders += 1;

            Ok(())
        })?;

        Ok(())
    }

    /// 函数级详细中文注释：释放额度（订单完成/取消时）
    fn release_quota(buyer: &T::AccountId, amount_usd: u64) -> sp_runtime::DispatchResult {
        use pallet::{BuyerQuotas, Error};

        BuyerQuotas::<T>::try_mutate(buyer, |profile| -> sp_runtime::DispatchResult {
            // 释放已占用额度
            profile.occupied_quota = profile.occupied_quota
                .checked_sub(amount_usd)
                .unwrap_or(0); // 防御性编程：即使为0也不报错

            profile.available_quota = profile.available_quota
                .checked_add(amount_usd)
                .ok_or(Error::<T>::ScoreOverflow)?;

            profile.active_orders = profile.active_orders.saturating_sub(1);

            Ok(())
        })?;

        Ok(())
    }

    /// 函数级详细中文注释：检查并发订单数是否超限
    fn check_concurrent_limit(buyer: &T::AccountId) -> Result<bool, sp_runtime::DispatchError> {
        let profile = BuyerQuotas::<T>::get(buyer);
        Ok(profile.active_orders < profile.max_concurrent_orders)
    }

    /// 函数级详细中文注释：记录订单完成（提升信用）
    fn record_order_completed(
        buyer: &T::AccountId,
        _order_id: u64,
    ) -> sp_runtime::DispatchResult {
        use pallet::{BuyerQuotas, Event};

        BuyerQuotas::<T>::try_mutate(buyer, |profile| -> sp_runtime::DispatchResult {
            // 增加完成订单数
            profile.total_orders += 1;
            profile.consecutive_good_orders += 1;

            // 提升信用分（每笔+2分，上限1000）
            profile.credit_score = profile.credit_score.saturating_add(2).min(1000);

            // 🆕 检查是否达到连续10单奖励条件
            if profile.consecutive_good_orders >= 10 {
                let bonus = 5u16;
                profile.credit_score = profile.credit_score.saturating_add(bonus).min(1000);

                // 重置计数器
                profile.consecutive_good_orders = 0;

                // 发出信用恢复事件
                Self::deposit_event(Event::CreditRecovered {
                    account: buyer.clone(),
                    recovery_points: bonus,
                    new_credit_score: profile.credit_score,
                    recovery_reason: 1, // 10单奖励
                });
            }

            // 重新计算最大额度
            let old_max_quota = profile.max_quota;
            profile.max_quota = crate::quota::calculate_max_quota(
                profile.credit_score,
                profile.total_orders,
            );

            // 重新计算并发订单数限制
            profile.max_concurrent_orders = crate::quota::calculate_max_concurrent(
                profile.total_orders,
            );

            // 如果额度提升，发出事件
            if profile.max_quota > old_max_quota {
                Self::deposit_event(Event::QuotaIncreased {
                    account: buyer.clone(),
                    old_max_quota,
                    new_max_quota: profile.max_quota,
                    reason: b"Order completed".to_vec().try_into().unwrap_or_default(),
                });
            }

            Ok(())
        })?;

        Ok(())
    }

    /// 函数级详细中文注释：记录订单取消（轻度降低信用）
    fn record_order_cancelled(
        buyer: &T::AccountId,
        _order_id: u64,
    ) -> sp_runtime::DispatchResult {
        use pallet::BuyerQuotas;

        BuyerQuotas::<T>::try_mutate(buyer, |profile| -> sp_runtime::DispatchResult {
            // 轻度惩罚：信用分-5
            profile.credit_score = profile.credit_score.saturating_sub(5);

            // 重置连续良好订单计数
            profile.consecutive_good_orders = 0;

            Ok(())
        })?;

        Ok(())
    }

    /// 函数级详细中文注释：记录违约行为（降低信用+减少额度）
    fn record_violation(
        buyer: &T::AccountId,
        violation_type: crate::quota::ViolationType,
    ) -> sp_runtime::DispatchResult {
        use sp_runtime::traits::{CheckedAdd, CheckedMul};
        use frame_support::traits::Get;
        use pallet::{BuyerQuotas, Error, Event};

        BuyerQuotas::<T>::try_mutate(buyer, |profile| -> sp_runtime::DispatchResult {
            // 计算惩罚参数
            let (score_penalty, quota_reduction_bps, penalty_duration_days, should_suspend) =
                crate::quota::calculate_violation_penalty(&violation_type, profile.total_violations);

            // 扣除信用分
            profile.credit_score = profile.credit_score.saturating_sub(score_penalty);

            // 减少额度（按比例）
            let quota_reduction = (profile.max_quota as u128)
                .saturating_mul(quota_reduction_bps as u128)
                .saturating_div(10000);
            profile.max_quota = profile.max_quota.saturating_sub(quota_reduction as u64);
            profile.available_quota = profile.available_quota.min(profile.max_quota);

            // 增加违约次数
            profile.total_violations += 1;
            profile.warnings += 1;

            // 重置连续良好订单计数
            profile.consecutive_good_orders = 0;

            // 记录违约时间
            profile.last_violation_at = <frame_system::Pallet<T>>::block_number();

            // 是否暂停服务
            if should_suspend {
                profile.is_suspended = true;

                // 计算暂停解除时间（如果不是永久拉黑）
                if penalty_duration_days < u32::MAX {
                    let blocks_per_day = T::BlocksPerDay::get();
                    let suspension_blocks = blocks_per_day
                        .checked_mul(&penalty_duration_days.into())
                        .ok_or(Error::<T>::ScoreOverflow)?;
                    let suspension_until = profile.last_violation_at
                        .checked_add(&suspension_blocks)
                        .ok_or(Error::<T>::ScoreOverflow)?;
                    profile.suspension_until = Some(suspension_until);

                    Self::deposit_event(Event::BuyerSuspended {
                        account: buyer.clone(),
                        reason: b"Violation penalty".to_vec().try_into().unwrap_or_default(),
                        suspension_until,
                    });
                } else {
                    // 永久拉黑
                    profile.is_blacklisted = true;
                    profile.suspension_until = None;

                    Self::deposit_event(Event::BuyerBlacklisted {
                        account: buyer.clone(),
                        reason: b"Malicious behavior".to_vec().try_into().unwrap_or_default(),
                        total_violations: profile.total_violations,
                    });
                }
            }

            // 保存违约记录
            BuyerViolations::<T>::try_mutate(buyer, |violations| {
                let violation_record = crate::quota::ViolationRecord {
                    violation_type: violation_type.clone(),
                    occurred_at: profile.last_violation_at,
                    score_penalty,
                    quota_reduction_bps,
                    penalty_duration_days,
                    caused_suspension: should_suspend,
                };

                violations.try_push(violation_record)
                    .map_err(|_| Error::<T>::TooManyViolationRecords)
            })?;

            // 发出事件
            let violation_type_code = match violation_type {
                crate::quota::ViolationType::OrderTimeout { .. } => 0u8,
                crate::quota::ViolationType::DisputeLoss { .. } => 1u8,
                crate::quota::ViolationType::MaliciousBehavior { .. } => 2u8,
            };

            Self::deposit_event(Event::BuyerViolationRecorded {
                account: buyer.clone(),
                violation_type: violation_type_code,
                score_penalty,
                new_credit_score: profile.credit_score,
            });

            Ok(())
        })?;

        Ok(())
    }

    /// 函数级详细中文注释：检查是否被暂停服务
    fn is_suspended(buyer: &T::AccountId) -> Result<bool, sp_runtime::DispatchError> {
        use frame_support::traits::Get;
        use pallet::{BuyerQuotas, Event};

        let profile = BuyerQuotas::<T>::get(buyer);

        // 🆕 方案C+：检查30天无违约恢复条件
        let current_block = <frame_system::Pallet<T>>::block_number();
        let (can_recover, recovery_points) = crate::quota::can_recover_credit(
            &profile,
            current_block,
            T::BlocksPerDay::get()
        );

        if can_recover && recovery_points > 0 {
            BuyerQuotas::<T>::mutate(buyer, |p| {
                p.credit_score = p.credit_score.saturating_add(recovery_points).min(1000);

                // 重新计算最大额度
                let old_max_quota = p.max_quota;
                p.max_quota = crate::quota::calculate_max_quota(
                    p.credit_score,
                    p.total_orders,
                );

                // 如果额度提升，更新可用额度
                if p.max_quota > old_max_quota {
                    let quota_increase = p.max_quota - old_max_quota;
                    p.available_quota = p.available_quota.saturating_add(quota_increase);
                }
            });

            let updated_profile = BuyerQuotas::<T>::get(buyer);

            Self::deposit_event(Event::CreditRecovered {
                account: buyer.clone(),
                recovery_points,
                new_credit_score: updated_profile.credit_score,
                recovery_reason: 0, // 30天无违约恢复
            });
        }

        // 如果被暂停且有解除时间，检查是否已过期
        if profile.is_suspended {
            if let Some(suspension_until) = profile.suspension_until {
                if current_block >= suspension_until {
                    // 自动解除暂停
                    BuyerQuotas::<T>::mutate(buyer, |p| {
                        p.is_suspended = false;
                        p.suspension_until = None;
                    });

                    let reinstated_profile = BuyerQuotas::<T>::get(buyer);

                    Self::deposit_event(Event::BuyerReinstated {
                        account: buyer.clone(),
                        new_credit_score: reinstated_profile.credit_score,
                        new_max_quota: reinstated_profile.max_quota,
                    });

                    return Ok(false);
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// 函数级详细中文注释：检查是否被拉黑
    fn is_blacklisted(buyer: &T::AccountId) -> Result<bool, sp_runtime::DispatchError> {
        let profile = BuyerQuotas::<T>::get(buyer);
        Ok(profile.is_blacklisted)
    }
}

// ===== BuyerCreditInterface 实现 =====

/// 函数级详细中文注释：为OTC订单实现BuyerCreditInterface
impl<T: pallet::Config> BuyerCreditInterface<T::AccountId> for pallet::Pallet<T> {
    fn get_buyer_credit_score(buyer: &T::AccountId) -> Result<u16, sp_runtime::DispatchError> {
        let profile = BuyerQuotas::<T>::get(buyer);
        Ok(profile.credit_score)
    }

    fn check_buyer_daily_limit(buyer: &T::AccountId, _amount_usd_cents: u64) -> Result<(), sp_runtime::DispatchError> {
        use frame_support::ensure;
        let profile = BuyerQuotas::<T>::get(buyer);
        ensure!(!profile.is_suspended, pallet::Error::<T>::BuyerSuspended);
        ensure!(!profile.is_blacklisted, pallet::Error::<T>::BuyerBlacklisted);
        Ok(())
    }

    fn check_buyer_single_limit(buyer: &T::AccountId, _amount_usd_cents: u64) -> Result<(), sp_runtime::DispatchError> {
        use frame_support::ensure;
        let profile = BuyerQuotas::<T>::get(buyer);
        ensure!(!profile.is_suspended, pallet::Error::<T>::BuyerSuspended);
        ensure!(!profile.is_blacklisted, pallet::Error::<T>::BuyerBlacklisted);
        Ok(())
    }
}
