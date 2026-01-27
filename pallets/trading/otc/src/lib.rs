//! # OTC Order Pallet (场外交易订单模块 - 集成KYC认证)
//!
//! ## 概述
//!
//! 本模块负责 OTC（场外交易）订单的完整生命周期管理，包括：
//! - 订单创建与管理
//! - 首购订单特殊逻辑（固定USD价值，动态DUST数量）
//! - 订单状态流转（创建→付款→释放→完成）
//! - 订单争议与仲裁
//! - 自动清理过期订单
//! - **🆕 KYC身份认证要求（基于pallet-identity）**
//!
//! ## KYC认证功能
//!
//! - 委员会可以启用/禁用KYC要求
//! - 支持不同的认证等级要求（Reasonable/KnownGood等）
//! - 紧急豁免账户机制
//! - 只有通过KYC认证的用户才能创建OTC订单
//!
//! ## 版本历史
//!
//! - v0.1.0 (2025-11-03): 从 pallet-trading 拆分而来
//! - v0.2.0 (2025-11-13): 集成KYC认证功能
//! - v0.3.0 (2025-11-28): 集成聊天权限系统

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

mod types;
mod kyc;

// 选择性导出 types 中的类型（避免 KycConfig 冲突）
pub use types::{KycVerificationResult, KycFailureReason};

// TODO: 测试文件待创建
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        traits::{Currency, Get, UnixTime},
        BoundedVec,
        sp_runtime::SaturatedConversion,
    };
    use sp_core::H256;
    use pallet_escrow::Escrow as EscrowTrait;
    use pallet_chat_permission::SceneAuthorizationManager;
    use pallet_trading_credit::quota::BuyerQuotaInterface;
    use pallet_storage_service::CidLockManager;
    use sp_runtime::traits::Hash;
    
    // 🆕 v0.4.0: 从 pallet-trading-common 导入公共类型和 Trait
    use pallet_trading_common::{
        TronAddress,
        MomentOf,
        PricingProvider,
        MakerInterface,
        MakerCreditInterface,
        MakerValidationError,
    };
    
    // MakerApplicationInfo 通过 MakerInterface::get_maker_application 返回

    /// 函数级详细中文注释：Balance 类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
    // ===== 数据结构 =====
    
    /// 函数级详细中文注释：订单状态枚举
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum OrderState {
        /// 已创建，等待买家付款
        Created,
        /// 买家已标记付款或做市商已确认
        PaidOrCommitted,
        /// DUST已释放
        Released,
        /// 已退款
        Refunded,
        /// 已取消
        Canceled,
        /// 争议中
        Disputed,
        /// 已关闭
        Closed,
        /// 已过期（1小时未支付，自动取消）
        Expired,
    }
    
    // ===== 🆕 2026-01-18: 买家押金机制 =====
    
    /// 函数级详细中文注释：押金状态枚举
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub enum DepositStatus {
        /// 无押金（首购/信用免押）
        #[default]
        None,
        /// 押金已锁定
        Locked,
        /// 押金已释放（订单完成）
        Released,
        /// 押金已没收（超时/取消/争议败诉）
        Forfeited,
        /// 押金部分没收（买家主动取消）
        PartiallyForfeited,
    }
    
    /// 函数级详细中文注释：争议状态枚举
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DisputeStatus {
        /// 等待做市商响应
        WaitingMakerResponse,
        /// 等待仲裁
        WaitingArbitration,
        /// 买家胜诉
        BuyerWon,
        /// 做市商胜诉
        MakerWon,
        /// 已取消
        Cancelled,
    }
    
    /// 函数级详细中文注释：争议记录结构
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Dispute<T: Config> {
        /// 订单ID
        pub order_id: u64,
        /// 发起方（买家）
        pub initiator: T::AccountId,
        /// 被告方（做市商）
        pub respondent: T::AccountId,
        /// 发起时间（Unix秒）
        pub created_at: MomentOf,
        /// 做市商响应截止时间
        pub response_deadline: MomentOf,
        /// 仲裁截止时间
        pub arbitration_deadline: MomentOf,
        /// 争议状态
        pub status: DisputeStatus,
        /// 买家证据 CID
        pub buyer_evidence: Option<pallet_trading_common::Cid>,
        /// 做市商证据 CID
        pub maker_evidence: Option<pallet_trading_common::Cid>,
    }
    
    /// 🆕 2026-01-18: 订单时间信息结构（供 RPC 查询使用）
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    #[scale_info(skip_type_params(T))]
    pub struct OrderTimeInfo<T: Config> {
        /// 订单ID
        pub order_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 买家账户
        pub buyer: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// USDT 金额
        pub usdt_amount: BalanceOf<T>,
        /// 创建时间（Unix秒）
        pub created_at: u64,
        /// 过期时间（Unix秒）
        pub expire_at: u64,
        /// 剩余秒数（0表示已过期）
        pub remaining_seconds: u64,
        /// 可读剩余时间（如 "45m", "1h 30m"）
        pub remaining_readable: sp_std::vec::Vec<u8>,
        /// 订单状态（0-7）
        pub state: u8,
        /// 是否已过期（仅 Created 状态）
        pub is_expired: bool,
    }
    
    /// 🆕 存储膨胀防护：归档订单结构 L1（精简版，~48字节）
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct ArchivedOrder<T: Config> {
        /// 做市商ID
        pub maker_id: u64,
        /// 买家账户
        pub taker: T::AccountId,
        /// 数量（DUST数量，压缩为u64）
        pub qty: u64,
        /// 总金额（USDT金额，压缩为u64）
        pub amount: u64,
        /// 订单状态
        pub state: OrderState,
        /// 完成时间（Unix秒）
        pub completed_at: u64,
    }

    /// 🆕 存储膨胀防护：归档订单结构 L2（最小版，~16字节）
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct ArchivedOrderL2 {
        /// 订单ID
        pub id: u64,
        /// 订单状态 (0-7)
        pub status: u8,
        /// 年月 (YYMM格式，如2601表示2026年1月)
        pub year_month: u16,
        /// 金额档位 (0-5)
        pub amount_tier: u8,
        /// 保留标志位
        pub flags: u32,
    }

    /// 🆕 存储膨胀防护：OTC永久统计
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen, Default)]
    pub struct OtcPermanentStats {
        /// 总订单数
        pub total_orders: u64,
        /// 已完成订单数
        pub completed_orders: u64,
        /// 已取消订单数
        pub cancelled_orders: u64,
        /// 总交易额（压缩）
        pub total_volume: u64,
    }

    /// 函数级详细中文注释：OTC订单结构
    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Order<T: Config> {
        /// 做市商ID
        pub maker_id: u64,
        /// 做市商账户
        pub maker: T::AccountId,
        /// 买家账户
        pub taker: T::AccountId,
        /// 单价（USDT/DUST，精度10^6）
        pub price: BalanceOf<T>,
        /// 数量（DUST数量）
        pub qty: BalanceOf<T>,
        /// 总金额（USDT金额）
        pub amount: BalanceOf<T>,
        /// 创建时间
        pub created_at: MomentOf,
        /// 超时时间
        pub expire_at: MomentOf,
        /// 证据窗口截止时间
        pub evidence_until: MomentOf,
        /// 做市商TRON收款地址
        pub maker_tron_address: TronAddress,
        /// 支付承诺哈希（买家提供）
        pub payment_commit: H256,
        /// 联系方式承诺哈希（买家提供）
        pub contact_commit: H256,
        /// 订单状态
        pub state: OrderState,
        /// 订单完成时间
        pub completed_at: Option<MomentOf>,
        /// 是否为首购订单
        pub is_first_purchase: bool,
        // ===== 🆕 2026-01-18: 买家押金字段 =====
        /// 买家押金金额（0 表示免押金）
        pub buyer_deposit: BalanceOf<T>,
        /// 押金状态
        pub deposit_status: DepositStatus,
    }
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    // ===== 🆕 2026-01-18: 自动过期处理 Hooks =====
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// 函数级详细中文注释：区块初始化时检查过期订单
        /// 
        /// ## 功能说明
        /// - 每 100 个区块检查一次（约 10 分钟）
        /// - 仅处理 Created 状态的订单
        /// - 每次最多处理 10 个过期订单，避免区块过重
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            // 每100个区块检查一次
            let check_interval: u32 = 100;
            let now_u32: u32 = now.saturated_into();
            if now_u32 % check_interval != 0 {
                return Weight::zero();
            }
            
            Self::process_expired_orders()
        }

        /// 🆕 存储膨胀防护：空闲时归档已完成订单
        fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = Weight::from_parts(20_000, 0);
            if remaining_weight.ref_time() < base_weight.ref_time() * 10 {
                return Weight::zero();
            }

            // 阶段1: 活跃订单 → L1 归档
            let w1 = Self::archive_completed_orders(5);
            
            // 阶段2: L1 归档 → L2 归档
            let w2 = Self::archive_l1_to_l2(5);
            
            w1.saturating_add(w2)
        }
    }
    
    /// 函数级详细中文注释：OTC订单模块配置 trait
    #[pallet::config]
    /// 函数级中文注释：OtcOrder Pallet 配置 trait
    /// - 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    /// - 🆕 集成KYC认证配置（不再继承 pallet_identity::Config，使用数值表示等级）
    /// - 🆕 2025-11-28: 集成聊天权限系统
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {

        /// 货币类型
        type Currency: Currency<Self::AccountId>;

        /// Timestamp（用于获取当前时间）
        type Timestamp: UnixTime;

        /// 托管服务接口（注意：Escrow 使用 order_id 作为托管 ID）
        type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;

        /// 买家信用记录接口（同时支持额度管理）
        type Credit: pallet_trading_credit::BuyerCreditInterface<Self::AccountId>
            + pallet_trading_credit::quota::BuyerQuotaInterface<Self::AccountId>;

        /// 做市商信用记录接口
        /// 🆕 2026-01-18: 统一使用 pallet_trading_common::MakerCreditInterface
        type MakerCredit: pallet_trading_common::MakerCreditInterface;

        /// 定价服务接口
        type Pricing: PricingProvider<BalanceOf<Self>>;

        /// Maker Pallet 类型（用于跨 pallet 调用）
        type MakerPallet: MakerInterface<Self::AccountId, BalanceOf<Self>>;

        /// 🆕 委员会起源（用于KYC配置管理）
        type CommitteeOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 🆕 Identity Provider（用于KYC验证）
        type IdentityProvider: IdentityVerificationProvider<Self::AccountId>;

        /// 🆕 2025-11-28: 聊天权限管理器
        /// 用于在订单创建时自动授予买卖双方聊天权限
        type ChatPermission: pallet_chat_permission::SceneAuthorizationManager<
            Self::AccountId,
            BlockNumberFor<Self>,
        >;

        /// 订单超时时间（默认 1 小时，毫秒）
        #[pallet::constant]
        type OrderTimeout: Get<u64>;

        /// 证据窗口时间（默认 24 小时，毫秒）
        #[pallet::constant]
        type EvidenceWindow: Get<u64>;

        /// 首购订单USD固定价值（精度 10^6，10_000_000 = 10 USD）
        #[pallet::constant]
        type FirstPurchaseUsdValue: Get<u128>;

        /// 首购订单最小DUST数量（防止汇率异常）
        #[pallet::constant]
        type MinFirstPurchaseDustAmount: Get<BalanceOf<Self>>;

        /// 首购订单最大DUST数量（防止汇率异常）
        #[pallet::constant]
        type MaxFirstPurchaseDustAmount: Get<BalanceOf<Self>>;

        /// OTC订单最大USD金额（200 USD，精度10^6）
        #[pallet::constant]
        type MaxOrderUsdAmount: Get<u64>;

        /// OTC订单最小USD金额（20 USD，精度10^6，首购除外）
        #[pallet::constant]
        type MinOrderUsdAmount: Get<u64>;

        /// 首购订单固定USD金额（10 USD，精度10^6）
        #[pallet::constant]
        type FirstPurchaseUsdAmount: Get<u64>;

        /// 金额验证容差（1%，用于处理价格微小波动）
        #[pallet::constant]
        type AmountValidationTolerance: Get<u16>;

        /// 每个做市商最多同时接收的首购订单数量（默认 5）
        #[pallet::constant]
        type MaxFirstPurchaseOrdersPerMaker: Get<u32>;

        // ===== 🆕 2026-01-18: 买家押金配置 =====
        
        /// 最小押金金额
        #[pallet::constant]
        type MinDeposit: Get<BalanceOf<Self>>;
        
        /// 低风险押金比例（bps，300 = 3%，信用分 50-69）
        #[pallet::constant]
        type DepositRateLow: Get<u16>;
        
        /// 中风险押金比例（bps，500 = 5%，信用分 30-49）
        #[pallet::constant]
        type DepositRateMedium: Get<u16>;
        
        /// 高风险押金比例（bps，1000 = 10%，信用分 < 30）
        #[pallet::constant]
        type DepositRateHigh: Get<u16>;
        
        /// 免押金信用分阈值（默认 70）
        #[pallet::constant]
        type CreditScoreExempt: Get<u16>;
        
        /// 免押金最少完成订单数（默认 5）
        #[pallet::constant]
        type MinOrdersForExempt: Get<u32>;
        
        /// 取消订单押金扣除比例（bps，3000 = 30%）
        #[pallet::constant]
        type CancelPenaltyRate: Get<u16>;
        
        /// 做市商最低押金USD价值（精度10^6，800_000_000 = 800 USD）
        #[pallet::constant]
        type MinMakerDepositUsd: Get<u64>;
        
        /// 争议响应超时时间（秒，默认 24 小时 = 86400）
        #[pallet::constant]
        type DisputeResponseTimeout: Get<u64>;
        
        /// 争议仲裁超时时间（秒，默认 48 小时 = 172800）
        #[pallet::constant]
        type DisputeArbitrationTimeout: Get<u64>;
        
        /// 仲裁员起源（用于争议判定）
        type ArbitratorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 权重信息
        type WeightInfo: WeightInfo;

        /// 🆕 P3: CID 锁定管理器（争议期间锁定证据 CID）
        /// 
        /// 功能：
        /// - 发起争议时自动 PIN 并锁定证据 CID
        /// - 仲裁完成后自动解锁并 Unpin
        /// - 防止争议期间证据被删除
        type CidLockManager: pallet_storage_service::CidLockManager<Self::Hash, BlockNumberFor<Self>>;
    }
    
    // 🆕 v0.4.0: PricingProvider, MakerInterface, MakerApplicationInfo 已移至 common 模块

    /// 函数级详细中文注释：Identity 验证 Provider trait
    /// 用于查询账户的身份认证状态，避免直接依赖 pallet_identity::Config
    pub trait IdentityVerificationProvider<AccountId> {
        /// 获取账户的最高身份认证等级（数值）
        /// 返回 None 表示未设置身份信息
        /// 返回值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        fn get_highest_judgement_priority(who: &AccountId) -> Option<u8>;

        /// 检查账户的身份认证是否有问题
        fn has_problematic_judgement(who: &AccountId) -> bool;
    }

    /// 临时实现（用于编译通过）
    impl<AccountId> IdentityVerificationProvider<AccountId> for () {
        fn get_highest_judgement_priority(_who: &AccountId) -> Option<u8> {
            None
        }

        fn has_problematic_judgement(_who: &AccountId) -> bool {
            false
        }
    }
    
    // 🆕 v0.4.0: PricingProvider 空实现已移至 common 模块
    
    // ===== 存储 =====
    
    /// 函数级详细中文注释：下一个订单 ID
    #[pallet::storage]
    #[pallet::getter(fn next_order_id)]
    pub type NextOrderId<T> = StorageValue<_, u64, ValueQuery>;
    
    /// 函数级详细中文注释：订单记录
    #[pallet::storage]
    #[pallet::getter(fn orders)]
    pub type Orders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // order_id
        Order<T>,
    >;
    
    /// 函数级详细中文注释：买家订单列表
    #[pallet::storage]
    #[pallet::getter(fn buyer_orders)]
    pub type BuyerOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,  // 每个买家最多100个订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商订单列表
    #[pallet::storage]
    #[pallet::getter(fn maker_orders)]
    pub type MakerOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<200>>,  // 每个做市商最多200个活跃订单（已完成订单应归档）
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：买家是否已首购
    #[pallet::storage]
    #[pallet::getter(fn has_first_purchased)]
    pub type HasFirstPurchased<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商首购订单计数
    #[pallet::storage]
    #[pallet::getter(fn maker_first_purchase_count)]
    pub type MakerFirstPurchaseCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        u32,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商首购订单列表
    #[pallet::storage]
    #[pallet::getter(fn maker_first_purchase_orders)]
    pub type MakerFirstPurchaseOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<10>>,  // 最多10个首购订单
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：TRON 交易哈希使用记录（防重放）
    #[pallet::storage]
    #[pallet::getter(fn tron_tx_used)]
    pub type TronTxUsed<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,  // tx_hash
        BlockNumberFor<T>,  // recorded_at
    >;
    
    /// 函数级详细中文注释：TRON 交易哈希队列（用于清理）
    #[pallet::storage]
    #[pallet::getter(fn tron_tx_queue)]
    pub type TronTxQueue<T: Config> = StorageValue<
        _,
        BoundedVec<(H256, BlockNumberFor<T>), ConstU32<2000>>,
        ValueQuery,
    >;

    // ===== 🆕 2026-01-18: 买家押金存储 =====
    
    /// 函数级详细中文注释：争议记录
    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // order_id
        Dispute<T>,
        OptionQuery,
    >;
    
    /// 函数级详细中文注释：买家已完成订单计数（用于判断信用免押）
    #[pallet::storage]
    #[pallet::getter(fn buyer_completed_order_count)]
    pub type BuyerCompletedOrderCount<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        u32,
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：押金池总余额（用于审计）
    #[pallet::storage]
    #[pallet::getter(fn total_deposit_pool_balance)]
    pub type TotalDepositPoolBalance<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    // ========================================
    // 🆕 存储膨胀防护 - 订单归档存储
    // ========================================

    /// 归档订单（精简格式）
    #[pallet::storage]
    #[pallet::getter(fn archived_orders)]
    pub type ArchivedOrders<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // order_id
        ArchivedOrder<T>,
        OptionQuery,
    >;

    /// 归档游标（记录处理进度）
    #[pallet::storage]
    pub type ArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 🆕 L2归档订单（最小格式）
    #[pallet::storage]
    #[pallet::getter(fn archived_orders_l2)]
    pub type ArchivedOrdersL2<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // order_id
        ArchivedOrderL2,
        OptionQuery,
    >;

    /// 🆕 L1归档游标
    #[pallet::storage]
    pub type L1ArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 🆕 OTC永久统计
    #[pallet::storage]
    #[pallet::getter(fn otc_stats)]
    pub type OtcStats<T: Config> = StorageValue<_, OtcPermanentStats, ValueQuery>;

    // ===== KYC存储 =====

    /// 函数级详细中文注释：KYC配置存储
    #[pallet::storage]
    #[pallet::getter(fn kyc_config)]
    pub type KycConfig<T: Config> = StorageValue<
        _,
        crate::types::KycConfig<BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// 函数级详细中文注释：KYC豁免账户列表
    #[pallet::storage]
    #[pallet::getter(fn kyc_exempt_accounts)]
    pub type KycExemptAccounts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        (),
        OptionQuery,
    >;

    // ===== Genesis配置 =====

    /// 函数级详细中文注释：Genesis配置结构
    ///
    /// 🔮 延迟实现：需要解决以下问题
    /// 1. Storage type `KycConfig` 与 GenesisConfig 字段同名冲突
    /// 2. BlockNumberFor<T> 需要额外的 serde bounds
    /// 3. T::AccountId 需要 serde 支持
    /// 
    /// 建议方案：
    /// - 在 runtime genesis_config_presets.rs 中手动初始化 KYC 配置
    /// - 或使用 pallet::genesis_config 的简化版本（仅 exempt_accounts）

    // ===== 事件 =====
    
    /// 函数级详细中文注释：OTC订单模块事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 订单已创建
        OrderCreated {
            order_id: u64,
            maker_id: u64,
            buyer: T::AccountId,
            dust_amount: BalanceOf<T>,
            is_first_purchase: bool,
        },
        /// 订单状态已变更
        OrderStateChanged {
            order_id: u64,
            old_state: u8,
            new_state: u8,
            actor: Option<T::AccountId>,
        },
        /// 首购订单已创建
        FirstPurchaseOrderCreated {
            order_id: u64,
            buyer: T::AccountId,
            maker_id: u64,
            usd_value: u128,
            dust_amount: BalanceOf<T>,
        },
        /// TRON 交易哈希已记录
        TronTxHashRecorded {
            tx_hash: H256,
        },
        /// TRON 交易哈希已清理
        TronTxHashCleaned {
            count: u32,
        },

        // ===== KYC相关事件 =====

        /// KYC要求已启用
        /// 等级优先级：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        KycEnabled {
            min_judgment_priority: u8,
        },
        /// KYC要求已禁用
        KycDisabled,
        /// KYC最低等级已更新
        /// 等级优先级：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood
        KycLevelUpdated {
            new_priority: u8,
        },
        /// 账户被添加到KYC豁免列表
        AccountExemptedFromKyc {
            account: T::AccountId,
        },
        /// 账户从KYC豁免列表中移除
        AccountRemovedFromKycExemption {
            account: T::AccountId,
        },
        /// KYC验证失败
        /// 原因代码：0=IdentityNotSet, 1=NoValidJudgement, 2=InsufficientLevel, 3=QualityIssue
        KycVerificationFailed {
            account: T::AccountId,
            reason_code: u8,
        },
        
        // ===== 🆕 2026-01-18: 自动过期事件 =====
        
        /// 订单已自动过期
        OrderAutoExpired {
            order_id: u64,
            buyer: T::AccountId,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
        },
        /// 过期订单批量处理完成
        ExpiredOrdersProcessed {
            count: u32,
            block_number: BlockNumberFor<T>,
        },
        
        // ===== 🆕 2026-01-18: 买家押金事件 =====
        
        /// 买家押金已锁定
        BuyerDepositLocked {
            order_id: u64,
            buyer: T::AccountId,
            deposit_amount: BalanceOf<T>,
        },
        /// 买家押金已释放（订单完成）
        BuyerDepositReleased {
            order_id: u64,
            buyer: T::AccountId,
            refund_amount: BalanceOf<T>,
        },
        /// 买家押金已没收（超时）
        BuyerDepositForfeited {
            order_id: u64,
            buyer: T::AccountId,
            maker_id: u64,
            forfeited_amount: BalanceOf<T>,
        },
        /// 买家押金部分没收（主动取消）
        BuyerDepositPartiallyForfeited {
            order_id: u64,
            buyer: T::AccountId,
            maker_id: u64,
            forfeited_amount: BalanceOf<T>,
            refund_amount: BalanceOf<T>,
        },
        /// 争议已发起
        DisputeInitiated {
            order_id: u64,
            buyer: T::AccountId,
        },
        /// 做市商已响应争议
        DisputeResponded {
            order_id: u64,
            maker: T::AccountId,
        },
        /// 争议已判定
        DisputeResolved {
            order_id: u64,
            buyer_wins: bool,
        },
    }
    
    // ===== 错误 =====
    
    /// 函数级详细中文注释：OTC订单模块错误
    #[pallet::error]
    pub enum Error<T> {
        /// 订单不存在
        OrderNotFound,
        /// 做市商不存在
        MakerNotFound,
        /// 做市商未激活
        MakerNotActive,
        /// 订单状态不正确
        InvalidOrderStatus,
        /// 未授权
        NotAuthorized,
        /// 编码错误
        EncodingError,
        /// 存储限制已达到
        StorageLimitReached,
        /// 订单太多
        TooManyOrders,
        /// 已经首购过
        AlreadyFirstPurchased,
        /// 首购配额已用完
        FirstPurchaseQuotaExhausted,
        /// 做市商余额不足
        MakerInsufficientBalance,
        /// 做市商押金不足（USD价值低于阈值）
        MakerDepositInsufficient,
        /// 定价不可用
        PricingUnavailable,
        /// 价格无效
        InvalidPrice,
        /// 计算溢出
        CalculationOverflow,
        /// TRON交易哈希已使用
        TronTxHashAlreadyUsed,

        /// 订单金额超过限制
        OrderAmountExceedsLimit,

        /// 订单金额太小
        OrderAmountTooSmall,

        /// 金额计算溢出
        AmountCalculationOverflow,

        /// 定价服务不可用
        PricingServiceUnavailable,

        // ===== KYC相关错误 =====

        /// 未设置身份信息
        IdentityNotSet,
        /// 没有有效的身份判断
        NoValidJudgement,
        /// KYC认证等级不足
        InsufficientKycLevel,
        /// 身份认证质量问题
        IdentityQualityIssue,
        /// 账户已在豁免列表中
        AccountAlreadyExempted,
        /// 账户不在豁免列表中
        AccountNotExempted,
        
        // ===== 🆕 2026-01-18: 买家押金相关错误 =====
        
        /// 买家押金余额不足
        InsufficientDepositBalance,
        /// 争议不存在
        DisputeNotFound,
        /// 争议状态不正确
        InvalidDisputeStatus,
        /// 非争议发起方
        NotDisputeInitiator,
        /// 非争议响应方
        NotDisputeRespondent,
        /// 争议响应已超时
        DisputeResponseTimeout,
        /// 不是订单买家
        NotOrderBuyer,
    }
    
    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建OTC订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST数量
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn create_order(
            origin: OriginFor<T>,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            payment_commit: H256,
            contact_commit: H256,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let _order_id = Self::do_create_order(
                &buyer,
                maker_id,
                dust_amount,
                payment_commit,
                contact_commit,
            )?;
            Ok(())
        }
        
        /// 函数级详细中文注释：创建首购订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn create_first_purchase(
            origin: OriginFor<T>,
            maker_id: u64,
            payment_commit: H256,
            contact_commit: H256,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            let _order_id = Self::do_create_first_purchase(
                &buyer,
                maker_id,
                payment_commit,
                contact_commit,
            )?;
            Ok(())
        }
        
        /// 函数级详细中文注释：买家标记已付款
        ///
        /// # 参数
        /// - `origin`: 调用者（买家，必须是签名账户）
        /// - `order_id`: 订单ID
        /// - `tron_tx_hash`: TRON交易哈希（可选）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn mark_paid(
            origin: OriginFor<T>,
            order_id: u64,
            tron_tx_hash: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            Self::do_mark_paid(&buyer, order_id, tron_tx_hash)
        }
        
        /// 函数级详细中文注释：做市商释放DUST
        ///
        /// # 参数
        /// - `origin`: 调用者（做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn release_dust(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            Self::do_release_dust(&maker, order_id)
        }
        
        /// 函数级详细中文注释：取消订单
        ///
        /// # 参数
        /// - `origin`: 调用者（买家或做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_cancel_order(&who, order_id)
        }
        
        /// 函数级详细中文注释：发起订单争议
        ///
        /// # 参数
        /// - `origin`: 调用者（买家或做市商，必须是签名账户）
        /// - `order_id`: 订单ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn dispute_order(
            origin: OriginFor<T>,
            order_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_dispute_order(&who, order_id)
        }

        // ===== KYC管理函数 =====

        /// 函数级详细中文注释：启用KYC要求
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `min_judgment_priority`: 最低认证等级（数值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(6)]
        #[pallet::weight(<T as Config>::WeightInfo::enable_kyc_requirement())]
        pub fn enable_kyc_requirement(
            origin: OriginFor<T>,
            min_judgment_priority: u8,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            let config = crate::types::KycConfig {
                enabled: true,
                min_judgment_priority,
                effective_block: current_block,
                updated_at: current_block,
            };

            KycConfig::<T>::put(config);

            Self::deposit_event(Event::KycEnabled { min_judgment_priority });
            Ok(())
        }

        /// 函数级详细中文注释：禁用KYC要求
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(7)]
        #[pallet::weight(<T as Config>::WeightInfo::disable_kyc_requirement())]
        pub fn disable_kyc_requirement(origin: OriginFor<T>) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            KycConfig::<T>::mutate(|config| {
                config.enabled = false;
                config.effective_block = current_block;
                config.updated_at = current_block;
            });

            Self::deposit_event(Event::KycDisabled);
            Ok(())
        }

        /// 函数级详细中文注释：更新最低认证等级
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `new_priority`: 新的最低认证等级（数值：0=Unknown, 1=FeePaid, 2=Reasonable, 3=KnownGood）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(8)]
        #[pallet::weight(<T as Config>::WeightInfo::update_min_judgment_level())]
        pub fn update_min_judgment_level(
            origin: OriginFor<T>,
            new_priority: u8,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            let current_block = frame_system::Pallet::<T>::block_number();
            KycConfig::<T>::mutate(|config| {
                config.min_judgment_priority = new_priority;
                config.effective_block = current_block;
                config.updated_at = current_block;
            });

            Self::deposit_event(Event::KycLevelUpdated { new_priority });
            Ok(())
        }

        /// 函数级详细中文注释：将账户添加到KYC豁免列表
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `account`: 要豁免的账户
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(9)]
        #[pallet::weight(<T as Config>::WeightInfo::exempt_account_from_kyc())]
        pub fn exempt_account_from_kyc(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            ensure!(
                !KycExemptAccounts::<T>::contains_key(&account),
                Error::<T>::AccountAlreadyExempted
            );

            KycExemptAccounts::<T>::insert(&account, ());

            Self::deposit_event(Event::AccountExemptedFromKyc { account });
            Ok(())
        }

        /// 函数级详细中文注释：从KYC豁免列表移除账户
        ///
        /// # 参数
        /// - `origin`: 调用者（委员会起源）
        /// - `account`: 要移除豁免的账户
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_kyc_exemption())]
        pub fn remove_kyc_exemption(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            T::CommitteeOrigin::ensure_origin(origin)?;

            ensure!(
                KycExemptAccounts::<T>::contains_key(&account),
                Error::<T>::AccountNotExempted
            );

            KycExemptAccounts::<T>::remove(&account);

            Self::deposit_event(Event::AccountRemovedFromKycExemption { account });
            Ok(())
        }
        
        // ===== 🆕 2026-01-18: 争议相关 Extrinsics =====
        
        // ============================================================================
        // 🆕 争议功能已迁移到统一仲裁模块 (pallet-arbitration)
        // 
        // 迁移说明：
        // - 使用 arbitration.file_complaint 替代 initiate_dispute
        // - 使用 arbitration.respond_to_complaint 替代 respond_dispute  
        // - 使用 arbitration.resolve_complaint 替代 resolve_dispute
        //
        // OTC 域常量: b"otc_ord_"
        // 投诉类型: OtcSellerNotDeliver, OtcBuyerFalseClaim, OtcTradeFraud, OtcPriceDispute
        //
        // ArbitrationRouter.apply_decision 会调用 do_resolve_dispute 执行裁决
        // ============================================================================
        
        /// [已废弃] 买家发起争议 - 请使用 arbitration.file_complaint
        #[deprecated(note = "Use arbitration.file_complaint with domain=b\"otc_ord_\" instead")]
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn initiate_dispute(
            origin: OriginFor<T>,
            order_id: u64,
            evidence_cid: pallet_trading_common::Cid,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;
            Self::do_initiate_dispute(&buyer, order_id, evidence_cid)
        }
        
        /// [已废弃] 做市商响应争议 - 请使用 arbitration.respond_to_complaint
        #[deprecated(note = "Use arbitration.respond_to_complaint instead")]
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn respond_dispute(
            origin: OriginFor<T>,
            order_id: u64,
            evidence_cid: pallet_trading_common::Cid,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            Self::do_respond_dispute(&maker, order_id, evidence_cid)
        }
        
        /// [已废弃] 仲裁判定争议 - 请使用 arbitration.resolve_complaint
        #[deprecated(note = "Use arbitration.resolve_complaint instead")]
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::create_order())]
        pub fn resolve_dispute(
            origin: OriginFor<T>,
            order_id: u64,
            buyer_wins: bool,
        ) -> DispatchResult {
            T::ArbitratorOrigin::ensure_origin(origin)?;
            Self::do_resolve_dispute(order_id, buyer_wins)
        }
    }
    
    // ===== 内部实现 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建OTC订单
        /// 
        /// ## 功能说明
        /// 1. 验证做市商存在且激活
        /// 2. 获取当前DUST/USD价格
        /// 3. 计算订单总金额
        /// 4. 将做市商的DUST锁定到托管
        /// 5. 创建订单记录
        /// 6. 更新买家和做市商的订单列表
        /// 7. 发出订单创建事件
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: 购买的DUST数量
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        /// 
        /// ## 返回
        /// - `Ok(order_id)`: 订单ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_create_order(
            buyer: &T::AccountId,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            payment_commit: H256,
            contact_commit: H256,
        ) -> Result<u64, DispatchError> {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 🆕 Step 0: KYC验证检查
            Self::enforce_kyc_requirement(buyer)?;

            // 1. 验证订单金额（新增）
            let _usd_amount = Self::validate_order_amount(dust_amount, false)?;

            // 2. 🆕 使用统一的做市商验证逻辑
            let maker_app = T::MakerPallet::validate_maker(maker_id)
                .map_err(|e| match e {
                    MakerValidationError::NotFound => Error::<T>::MakerNotFound,
                    MakerValidationError::NotActive => Error::<T>::MakerNotActive,
                })?;
            
            // 2.5 验证做市商押金USD价值（使用pricing模块换算）
            // MakerPallet::get_deposit_usd_value 内部使用 Pricing::get_dust_to_usd_rate 换算
            let min_deposit_usd = T::MinMakerDepositUsd::get(); // 500_000_000 (500 USDT, 精度10^6)
            let maker_deposit_usd = T::MakerPallet::get_deposit_usd_value(maker_id)
                .unwrap_or(0);
            ensure!(
                maker_deposit_usd >= min_deposit_usd,
                Error::<T>::MakerDepositInsufficient
            );
            
            // 3. 获取当前DUST/USD价格
            let price = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingUnavailable)?;
            
            // 4. 计算总金额（USDT）= dust_amount * price
            let amount = dust_amount
                .checked_mul(&price)
                .ok_or(Error::<T>::CalculationOverflow)?;

            // 🆕 方案C+：买家额度检查和占用
            // 5. 计算订单USD金额（精度10^6）
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(dust_amount, price)?;

            // 6. 检查并占用买家额度
            T::Credit::occupy_quota(buyer, amount_usd)?;

            // 7. 获取做市商的TRON收款地址
            let maker_tron_address = maker_app.tron_address
                .try_into()
                .map_err(|_| Error::<T>::EncodingError)?;

            // 8. 获取订单ID（提前）
            let order_id = NextOrderId::<T>::get();

            // 9. 将做市商的DUST锁定到托管（使用 order_id 作为托管 ID）
            T::Escrow::lock_from(
                &maker_app.account,
                order_id,
                dust_amount,
            )?;
            
            // 🆕 2026-01-18: 计算并锁定买家押金
            let buyer_deposit = Self::calculate_buyer_deposit(buyer, dust_amount);
            let deposit_status = if buyer_deposit.is_zero() {
                DepositStatus::None
            } else {
                Self::lock_buyer_deposit(buyer, buyer_deposit)?;
                DepositStatus::Locked
            };

            // 10. 获取当前时间并计算超时时间
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let expire_at = now
                .checked_add(T::OrderTimeout::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            let evidence_until = now
                .checked_add(T::EvidenceWindow::get())
                .ok_or(Error::<T>::CalculationOverflow)?;

            // 11. 创建订单记录
            let order = Order {
                maker_id,
                maker: maker_app.account.clone(),
                taker: buyer.clone(),
                price,
                qty: dust_amount,
                amount,
                created_at: now,
                expire_at,
                evidence_until,
                maker_tron_address,
                payment_commit,
                contact_commit,
                state: OrderState::Created,
                completed_at: None,
                is_first_purchase: false,
                buyer_deposit,
                deposit_status,
            };

            // 12. 保存订单
            Orders::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id + 1);

            // 13. 更新买家订单列表
            BuyerOrders::<T>::try_mutate(buyer, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;

            // 14. 更新做市商订单列表
            MakerOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;

            // 15. 发出事件
            Self::deposit_event(Event::OrderCreated {
                order_id,
                maker_id,
                buyer: buyer.clone(),
                dust_amount,
                is_first_purchase: false,
            });
            
            // 🆕 2026-01-18: 发出押金锁定事件
            if !buyer_deposit.is_zero() {
                Self::deposit_event(Event::BuyerDepositLocked {
                    order_id,
                    buyer: buyer.clone(),
                    deposit_amount: buyer_deposit,
                });
            }

            // 16. 🆕 2025-11-28: 授予买卖双方聊天权限
            // 订单创建后，买家和做市商之间自动获得基于订单场景的聊天权限
            // 有效期：30天（30 * 24 * 60 * 10 个区块，假设 6 秒/区块）
            let chat_duration = 30u32 * 24 * 60 * 10; // 30天
            let order_metadata = sp_std::vec::Vec::from(
                alloc::format!("OTC订单#{}", order_id).as_bytes()
            );
            let _ = T::ChatPermission::grant_bidirectional_scene_authorization(
                *b"otc_ordr",
                buyer,
                &maker_app.account,
                pallet_chat_permission::SceneType::Order,
                pallet_chat_permission::SceneId::Numeric(order_id),
                Some(chat_duration.into()),
                order_metadata,
            );

            Ok(order_id)
        }
        
        /// 函数级详细中文注释：创建首购订单
        /// 
        /// ## 功能说明
        /// 1. 验证买家未进行过首购
        /// 2. 验证做市商首购配额未用完
        /// 3. 获取当前DUST/USD价格
        /// 4. 根据固定USD价值计算DUST数量
        /// 5. 验证DUST数量在合理范围内
        /// 6. 创建首购订单
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `maker_id`: 做市商ID
        /// - `payment_commit`: 支付承诺哈希
        /// - `contact_commit`: 联系方式承诺哈希
        /// 
        /// ## 返回
        /// - `Ok(order_id)`: 订单ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_create_first_purchase(
            buyer: &T::AccountId,
            maker_id: u64,
            payment_commit: H256,
            contact_commit: H256,
        ) -> Result<u64, DispatchError> {
            // 🆕 Step 0: KYC验证检查
            Self::enforce_kyc_requirement(buyer)?;

            // 1. 检查买家是否已首购
            ensure!(
                !HasFirstPurchased::<T>::get(buyer),
                Error::<T>::AlreadyFirstPurchased
            );
            
            // 2. 🆕 使用统一的做市商验证逻辑
            let maker_app = T::MakerPallet::validate_maker(maker_id)
                .map_err(|e| match e {
                    MakerValidationError::NotFound => Error::<T>::MakerNotFound,
                    MakerValidationError::NotActive => Error::<T>::MakerNotActive,
                })?;
            
            // 3. 检查做市商首购配额
            let current_count = MakerFirstPurchaseCount::<T>::get(maker_id);
            ensure!(
                current_count < T::MaxFirstPurchaseOrdersPerMaker::get(),
                Error::<T>::FirstPurchaseQuotaExhausted
            );
            
            // 5. 获取当前DUST/USD价格
            let price = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingUnavailable)?;
            
            // 6. 计算DUST数量
            // USD价值 / 价格 = DUST数量
            // 注意：price 是 USDT/DUST，所以需要除法
            let usd_value = T::FirstPurchaseUsdValue::get();
            let price_u128 = TryInto::<u128>::try_into(price)
                .map_err(|_| Error::<T>::CalculationOverflow)?;
            
            ensure!(price_u128 > 0, Error::<T>::InvalidPrice);
            
            // dust_amount = usd_value * 10^12 / price (考虑精度)
            let dust_amount_u128 = usd_value
                .checked_mul(1_000_000_000_000) // 10^12 (DUST精度)
                .and_then(|v| v.checked_div(price_u128))
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            let dust_amount: BalanceOf<T> = TryInto::<u128>::try_into(dust_amount_u128)
                .ok()
                .and_then(|v| TryInto::<BalanceOf<T>>::try_into(v).ok())
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            // 7. 验证DUST数量在合理范围内
            ensure!(
                dust_amount >= T::MinFirstPurchaseDustAmount::get(),
                Error::<T>::InvalidPrice
            );
            ensure!(
                dust_amount <= T::MaxFirstPurchaseDustAmount::get(),
                Error::<T>::InvalidPrice
            );
            
            // 8. 验证做市商余额
            let maker_balance = <T as Config>::Currency::free_balance(&maker_app.account);
            ensure!(
                maker_balance >= dust_amount,
                Error::<T>::MakerInsufficientBalance
            );
            
            // 9. 获取做市商的TRON收款地址
            let maker_tron_address = maker_app.tron_address
                .try_into()
                .map_err(|_| Error::<T>::EncodingError)?;
            
            // 10. 获取订单ID（提前）
            let order_id = NextOrderId::<T>::get();
            
            // 11. 将做市商的DUST锁定到托管（使用 order_id 作为托管 ID）
            T::Escrow::lock_from(
                &maker_app.account,
                order_id,
                dust_amount,
            )?;
            
            // 12. 获取当前时间并计算超时时间
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let expire_at = now
                .checked_add(T::OrderTimeout::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            let evidence_until = now
                .checked_add(T::EvidenceWindow::get())
                .ok_or(Error::<T>::CalculationOverflow)?;
            
            // 13. 创建订单记录
            let amount = usd_value
                .try_into()
                .map_err(|_| Error::<T>::CalculationOverflow)?;
            
            // 🆕 2026-01-18: 首购用户免押金
            use sp_runtime::traits::Zero;
            let buyer_deposit: BalanceOf<T> = Zero::zero();
            let deposit_status = DepositStatus::None;
            
            let order = Order {
                maker_id,
                maker: maker_app.account.clone(),
                taker: buyer.clone(),
                price,
                qty: dust_amount,
                amount,
                created_at: now,
                expire_at,
                evidence_until,
                maker_tron_address,
                payment_commit,
                contact_commit,
                state: OrderState::Created,
                completed_at: None,
                is_first_purchase: true,
                buyer_deposit,
                deposit_status,
            };
            
            // 14. 保存订单
            Orders::<T>::insert(order_id, order);
            NextOrderId::<T>::put(order_id + 1);
            
            // 15. 更新买家订单列表
            BuyerOrders::<T>::try_mutate(buyer, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;
            
            // 16. 更新做市商订单列表
            MakerOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::TooManyOrders)
            })?;
            
            // 17. 更新做市商首购计数和列表
            MakerFirstPurchaseCount::<T>::mutate(maker_id, |count| {
                *count = count.saturating_add(1);
            });
            
            MakerFirstPurchaseOrders::<T>::try_mutate(maker_id, |orders| {
                orders.try_push(order_id)
                    .map_err(|_| Error::<T>::StorageLimitReached)
            })?;
            
            // 18. 发出事件
            Self::deposit_event(Event::FirstPurchaseOrderCreated {
                order_id,
                buyer: buyer.clone(),
                maker_id,
                usd_value,
                dust_amount,
            });

            // 19. 🆕 2025-11-28: 授予买卖双方聊天权限
            // 首购订单创建后，买家和做市商之间自动获得基于订单场景的聊天权限
            // 有效期：30天（30 * 24 * 60 * 10 个区块，假设 6 秒/区块）
            let chat_duration = 30u32 * 24 * 60 * 10; // 30天
            let order_metadata = sp_std::vec::Vec::from(
                alloc::format!("首购订单#{}", order_id).as_bytes()
            );
            let _ = T::ChatPermission::grant_bidirectional_scene_authorization(
                *b"otc_ordr",
                buyer,
                &maker_app.account,
                pallet_chat_permission::SceneType::Order,
                pallet_chat_permission::SceneId::Numeric(order_id),
                Some(chat_duration.into()),
                order_metadata,
            );

            Ok(order_id)
        }
        
        /// 函数级详细中文注释：买家标记已付款
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在且状态为 Created
        /// 2. 验证调用者是订单买家
        /// 3. 如提供TRON交易哈希，验证未被使用
        /// 4. 更新订单状态为 PaidOrCommitted
        /// 5. 记录TRON交易哈希（如有）
        /// 6. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `order_id`: 订单ID
        /// - `tron_tx_hash`: TRON交易哈希（可选）
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_mark_paid(
            buyer: &T::AccountId,
            order_id: u64,
            tron_tx_hash: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证订单状态
            ensure!(
                matches!(order.state, OrderState::Created),
                Error::<T>::InvalidOrderStatus
            );
            
            // 3. 验证调用者是买家
            ensure!(order.taker == *buyer, Error::<T>::NotAuthorized);
            
            // 4. 如提供TRON交易哈希，验证并记录
            if let Some(tx_hash_vec) = tron_tx_hash {
                // 将 Vec<u8> 转换为 H256
                ensure!(tx_hash_vec.len() == 32, Error::<T>::EncodingError);
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&tx_hash_vec);
                let tx_hash = H256::from(hash_bytes);
                
                // 检查是否已使用
                ensure!(
                    !TronTxUsed::<T>::contains_key(tx_hash),
                    Error::<T>::TronTxHashAlreadyUsed
                );
                
                // 记录使用
                let current_block = frame_system::Pallet::<T>::block_number();
                TronTxUsed::<T>::insert(tx_hash, current_block);
                
                // 添加到清理队列
                TronTxQueue::<T>::try_mutate(|queue| {
                    queue.try_push((tx_hash, current_block))
                        .map_err(|_| Error::<T>::StorageLimitReached)
                })?;
                
                Self::deposit_event(Event::TronTxHashRecorded { tx_hash });
            }
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::PaidOrCommitted;
            Orders::<T>::insert(order_id, order);
            
            // 6. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::PaidOrCommitted),
                actor: Some(buyer.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：做市商释放DUST
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在且状态为 PaidOrCommitted
        /// 2. 验证调用者是订单做市商
        /// 3. 从托管释放DUST到买家
        /// 4. 更新订单状态为 Released
        /// 5. 更新信用记录
        /// 6. 更新首购状态（如是首购订单）
        /// 7. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `maker`: 做市商账户
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_release_dust(
            maker: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证订单状态
            ensure!(
                matches!(order.state, OrderState::PaidOrCommitted),
                Error::<T>::InvalidOrderStatus
            );
            
            // 3. 验证调用者是做市商
            ensure!(order.maker == *maker, Error::<T>::NotAuthorized);
            
            // 4. 从托管释放DUST到买家（使用 order_id 作为托管 ID）
            T::Escrow::release_all(order_id, &order.taker)?;
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Released;
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            order.completed_at = Some(now);
            Orders::<T>::insert(order_id, order.clone());
            
            // 6. 记录做市商订单完成到信用分 ✅
            let response_time_seconds = now.saturating_sub(order.created_at) as u32;
            let _ = T::MakerCredit::record_maker_order_completed(
                order.maker_id,
                order_id,
                response_time_seconds,
            );

            // 🆕 方案C+：买家额度管理
            // 7. 释放买家占用的额度
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(order.qty, order.price)?;
            let _ = T::Credit::release_quota(&order.taker, amount_usd);

            // 8. 记录订单完成，提升买家信用分
            let _ = T::Credit::record_order_completed(&order.taker, order_id);

            // 9. 如是首购订单，更新首购状态
            if order.is_first_purchase {
                HasFirstPurchased::<T>::insert(&order.taker, true);

                // 减少做市商首购订单计数
                MakerFirstPurchaseCount::<T>::mutate(order.maker_id, |count| {
                    *count = count.saturating_sub(1);
                });
            }
            
            // 🆕 2026-01-18: 退还买家押金
            if !order.buyer_deposit.is_zero() {
                let _ = Self::release_buyer_deposit(&order.taker, order.buyer_deposit);
                
                // 更新押金状态
                Orders::<T>::mutate(order_id, |o| {
                    if let Some(ord) = o {
                        ord.deposit_status = DepositStatus::Released;
                    }
                });
                
                Self::deposit_event(Event::BuyerDepositReleased {
                    order_id,
                    buyer: order.taker.clone(),
                    refund_amount: order.buyer_deposit,
                });
            }
            
            // 🆕 2026-01-18: 更新买家完成订单计数
            BuyerCompletedOrderCount::<T>::mutate(&order.taker, |count| {
                *count = count.saturating_add(1);
            });

            // 10. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Released),
                actor: Some(maker.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：取消订单
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在
        /// 2. 验证调用者权限（买家或做市商）
        /// 3. 验证订单状态可以取消
        /// 4. 从托管退还DUST给做市商
        /// 5. 更新订单状态为 Canceled
        /// 6. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `who`: 调用者账户（买家或做市商）
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_cancel_order(
            who: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            use pallet_trading_credit::quota::BuyerQuotaInterface;

            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证调用者是买家或做市商
            ensure!(
                order.taker == *who || order.maker == *who,
                Error::<T>::NotAuthorized
            );
            
            // 3. 验证订单状态（只有 Created 和 Expired 状态可以取消）
            ensure!(
                matches!(order.state, OrderState::Created | OrderState::Expired),
                Error::<T>::InvalidOrderStatus
            );
            
            // 4. 从托管退还DUST给做市商（使用 order_id 作为托管 ID）
            T::Escrow::refund_all(order_id, &order.maker)?;
            
            // 5. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Canceled;
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            order.completed_at = Some(now);
            Orders::<T>::insert(order_id, order.clone());

            // 🆕 方案C+：买家额度管理
            // 6. 释放买家占用的额度
            let amount_usd: u64 = Self::calculate_usd_amount_from_dust(order.qty, order.price)?;
            let _ = T::Credit::release_quota(&order.taker, amount_usd);

            // 7. 记录订单取消（轻度降低信用）
            let _ = T::Credit::record_order_cancelled(&order.taker, order_id);

            // 8. 如是首购订单，减少做市商首购计数
            if order.is_first_purchase {
                MakerFirstPurchaseCount::<T>::mutate(order.maker_id, |count| {
                    *count = count.saturating_sub(1);
                });
            }
            
            // 🆕 2026-01-18: 处理买家押金
            if !order.buyer_deposit.is_zero() {
                let is_buyer_cancel = order.taker == *who;
                
                if is_buyer_cancel {
                    // 买家主动取消：30% 没收给做市商，70% 退还
                    let penalty_rate = T::CancelPenaltyRate::get(); // bps, 3000 = 30%
                    // penalty = deposit * rate / 10000
                    let penalty_rate_balance: BalanceOf<T> = penalty_rate.into();
                    let divisor: BalanceOf<T> = 10000u32.into();
                    let penalty = order.buyer_deposit * penalty_rate_balance / divisor;
                    let refund = if order.buyer_deposit > penalty {
                        order.buyer_deposit - penalty
                    } else {
                        Zero::zero()
                    };
                    
                    // 没收部分给做市商
                    if !penalty.is_zero() {
                        let _ = Self::forfeit_buyer_deposit(&order.maker, penalty);
                    }
                    
                    // 退还剩余给买家
                    if !refund.is_zero() {
                        let _ = Self::release_buyer_deposit(&order.taker, refund);
                    }
                    
                    // 更新押金状态
                    Orders::<T>::mutate(order_id, |o| {
                        if let Some(ord) = o {
                            ord.deposit_status = DepositStatus::PartiallyForfeited;
                        }
                    });
                    
                    Self::deposit_event(Event::BuyerDepositPartiallyForfeited {
                        order_id,
                        buyer: order.taker.clone(),
                        maker_id: order.maker_id,
                        forfeited_amount: penalty,
                        refund_amount: refund,
                    });
                } else {
                    // 做市商取消：100% 退还买家
                    let _ = Self::release_buyer_deposit(&order.taker, order.buyer_deposit);
                    
                    // 更新押金状态
                    Orders::<T>::mutate(order_id, |o| {
                        if let Some(ord) = o {
                            ord.deposit_status = DepositStatus::Released;
                        }
                    });
                    
                    Self::deposit_event(Event::BuyerDepositReleased {
                        order_id,
                        buyer: order.taker.clone(),
                        refund_amount: order.buyer_deposit,
                    });
                }
            }

            // 9. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Canceled),
                actor: Some(who.clone()),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：发起订单争议
        /// 
        /// ## 功能说明
        /// 1. 验证订单存在
        /// 2. 验证调用者权限（买家或做市商）
        /// 3. 验证订单状态可以争议
        /// 4. 更新订单状态为 Disputed
        /// 5. 发出状态变更事件
        /// 
        /// ## 参数
        /// - `who`: 调用者账户（买家或做市商）
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_dispute_order(
            who: &T::AccountId,
            order_id: u64,
        ) -> DispatchResult {
            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证调用者是买家或做市商
            ensure!(
                order.taker == *who || order.maker == *who,
                Error::<T>::NotAuthorized
            );
            
            // 3. 验证订单状态（只有 PaidOrCommitted 状态可以发起争议）
            ensure!(
                matches!(order.state, OrderState::PaidOrCommitted),
                Error::<T>::InvalidOrderStatus
            );
            
            // 4. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Disputed;
            Orders::<T>::insert(order_id, order);
            
            // 5. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Disputed),
                actor: Some(who.clone()),
            });
            
            Ok(())
        }
        
        // ===== 🆕 2026-01-18: 争议处理内部函数 =====
        
        /// 函数级详细中文注释：买家发起争议（内部实现）
        /// 
        /// ## 处理步骤
        /// 1. 验证订单状态为 PaidOrCommitted
        /// 2. 验证调用者是订单买家
        /// 3. 验证订单尚未存在争议
        /// 4. 创建争议记录
        /// 5. 更新订单状态为 Disputed
        /// 
        /// 注：争议押金已移除，改用统一仲裁模块的投诉押金机制
        pub fn do_initiate_dispute(
            buyer: &T::AccountId,
            order_id: u64,
            evidence_cid: pallet_trading_common::Cid,
        ) -> DispatchResult {
            // 1. 获取订单
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证订单状态（只有 PaidOrCommitted 状态可以发起争议）
            ensure!(
                matches!(order.state, OrderState::PaidOrCommitted),
                Error::<T>::InvalidOrderStatus
            );
            
            // 3. 验证调用者是买家
            ensure!(order.taker == *buyer, Error::<T>::NotOrderBuyer);
            
            // 4. 验证订单尚未存在争议
            ensure!(
                !Disputes::<T>::contains_key(order_id),
                Error::<T>::InvalidDisputeStatus
            );
            
            // 5. 计算截止时间
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let response_deadline = now + T::DisputeResponseTimeout::get();
            let arbitration_deadline = now + T::DisputeArbitrationTimeout::get();
            
            // 6. 创建争议记录（无争议押金）
            let dispute = Dispute {
                order_id,
                initiator: buyer.clone(),
                respondent: order.maker.clone(),
                created_at: now,
                response_deadline,
                arbitration_deadline,
                status: DisputeStatus::WaitingMakerResponse,
                buyer_evidence: Some(evidence_cid),
                maker_evidence: None,
            };
            
            // 🆕 P3: 自动 PIN 并锁定买家证据 CID
            // 争议期间证据必须保持可访问，仲裁完成后自动解锁
            if let Some(ref cid) = dispute.buyer_evidence {
                let lock_reason = sp_std::vec::Vec::from(
                    alloc::format!("otc-dispute:{}", order_id).as_bytes()
                );
                let cid_hash = T::Hashing::hash(&cid[..]);
                let _ = T::CidLockManager::lock_cid(cid_hash, lock_reason, None);
            }
            
            Disputes::<T>::insert(order_id, dispute);
            
            // 8. 更新订单状态
            let old_state = order.state.clone();
            order.state = OrderState::Disputed;
            Orders::<T>::insert(order_id, order);
            
            // 9. 发出事件
            Self::deposit_event(Event::OrderStateChanged {
                order_id,
                old_state: Self::state_to_u8(&old_state),
                new_state: Self::state_to_u8(&OrderState::Disputed),
                actor: Some(buyer.clone()),
            });
            
            Self::deposit_event(Event::DisputeInitiated {
                order_id,
                buyer: buyer.clone(),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：做市商响应争议（内部实现）
        /// 
        /// ## 处理步骤
        /// 1. 验证争议存在且状态为 WaitingMakerResponse
        /// 2. 验证调用者是订单做市商
        /// 3. 验证响应未超时
        /// 4. 更新争议状态为 WaitingArbitration
        /// 
        /// 注：争议押金已移除，改用统一仲裁模块的投诉押金机制
        pub fn do_respond_dispute(
            maker: &T::AccountId,
            order_id: u64,
            evidence_cid: pallet_trading_common::Cid,
        ) -> DispatchResult {
            // 1. 获取争议记录
            let mut dispute = Disputes::<T>::get(order_id)
                .ok_or(Error::<T>::DisputeNotFound)?;
            
            // 2. 验证争议状态
            ensure!(
                dispute.status == DisputeStatus::WaitingMakerResponse,
                Error::<T>::InvalidDisputeStatus
            );
            
            // 3. 验证调用者是做市商
            ensure!(dispute.respondent == *maker, Error::<T>::NotDisputeRespondent);
            
            // 4. 验证响应未超时
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            ensure!(now <= dispute.response_deadline, Error::<T>::DisputeResponseTimeout);
            
            // 5. 更新争议记录（无争议押金）
            dispute.maker_evidence = Some(evidence_cid.clone());
            dispute.status = DisputeStatus::WaitingArbitration;
            Disputes::<T>::insert(order_id, dispute);
            
            // 🆕 P3: 自动 PIN 并锁定做市商证据 CID
            let lock_reason = sp_std::vec::Vec::from(
                alloc::format!("otc-dispute:{}", order_id).as_bytes()
            );
            let cid_hash = T::Hashing::hash(&evidence_cid[..]);
            let _ = T::CidLockManager::lock_cid(cid_hash, lock_reason, None);
            
            // 6. 发出事件
            Self::deposit_event(Event::DisputeResponded {
                order_id,
                maker: maker.clone(),
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：仲裁判定争议（内部实现）
        /// 
        /// ## 判定结果处理
        /// - 买家胜诉：退还买家订单押金，释放托管资金给买家
        /// - 做市商胜诉：没收买家订单押金给做市商，退还托管资金
        /// - 做市商未响应：自动判买家胜诉
        /// 
        /// 注：争议押金已移除，仅处理订单押金
        pub fn do_resolve_dispute(
            order_id: u64,
            buyer_wins: bool,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;
            
            // 1. 获取争议和订单记录
            let mut dispute = Disputes::<T>::get(order_id)
                .ok_or(Error::<T>::DisputeNotFound)?;
            let order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 2. 验证争议状态（WaitingArbitration 或 WaitingMakerResponse 超时）
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            let can_resolve = match dispute.status {
                DisputeStatus::WaitingArbitration => true,
                DisputeStatus::WaitingMakerResponse => now > dispute.response_deadline,
                _ => false,
            };
            ensure!(can_resolve, Error::<T>::InvalidDisputeStatus);
            
            if buyer_wins {
                // === 买家胜诉 ===
                
                // 1. 退还买家订单押金
                if !order.buyer_deposit.is_zero() {
                    let _ = Self::release_buyer_deposit(&order.taker, order.buyer_deposit);
                }
                
                // 2. 释放托管的 DUST 给买家（订单完成）
                let _ = T::Escrow::release_all(order_id, &order.taker);
                
                // 3. 更新订单状态
                Orders::<T>::mutate(order_id, |o| {
                    if let Some(ord) = o {
                        ord.state = OrderState::Released;
                        ord.deposit_status = DepositStatus::Released;
                        ord.completed_at = Some(now);
                    }
                });
                
                // 4. 更新争议状态
                dispute.status = DisputeStatus::BuyerWon;
                
            } else {
                // === 做市商胜诉 ===
                
                // 1. 没收买家订单押金给做市商
                if !order.buyer_deposit.is_zero() {
                    let _ = Self::forfeit_buyer_deposit(&order.maker, order.buyer_deposit);
                }
                
                // 2. 退还托管的 DUST 给做市商（订单取消）
                let _ = T::Escrow::refund_all(order_id, &order.maker);
                
                // 3. 更新订单状态
                Orders::<T>::mutate(order_id, |o| {
                    if let Some(ord) = o {
                        ord.state = OrderState::Canceled;
                        ord.deposit_status = DepositStatus::Forfeited;
                        ord.completed_at = Some(now);
                    }
                });
                
                // 4. 更新争议状态
                dispute.status = DisputeStatus::MakerWon;
            }
            
            // 7. 保存争议记录（在解锁前克隆需要的数据）
            let buyer_evidence = dispute.buyer_evidence.clone();
            let maker_evidence = dispute.maker_evidence.clone();
            Disputes::<T>::insert(order_id, dispute);
            
            // 🆕 P3: 仲裁完成后解锁所有证据 CID
            // 解锁原因与锁定时相同
            let lock_reason = sp_std::vec::Vec::from(
                alloc::format!("otc-dispute:{}", order_id).as_bytes()
            );
            
            // 解锁买家证据
            if let Some(ref cid) = buyer_evidence {
                let cid_hash = T::Hashing::hash(&cid[..]);
                let _ = T::CidLockManager::unlock_cid(cid_hash, lock_reason.clone());
            }
            
            // 解锁做市商证据
            if let Some(ref cid) = maker_evidence {
                let cid_hash = T::Hashing::hash(&cid[..]);
                let _ = T::CidLockManager::unlock_cid(cid_hash, lock_reason);
            }
            
            // 8. 发出事件
            Self::deposit_event(Event::DisputeResolved {
                order_id,
                buyer_wins,
            });
            
            Ok(())
        }
    }
    
    // ===== 公共查询接口 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：检查买家是否已首购
        pub fn has_user_first_purchased(who: &T::AccountId) -> bool {
            HasFirstPurchased::<T>::get(who)
        }
        
        /// 函数级详细中文注释：获取做市商首购订单数量
        pub fn get_maker_first_purchase_count(maker_id: u64) -> u32 {
            MakerFirstPurchaseCount::<T>::get(maker_id)
        }
        
        // ===== 🆕 2026-01-18: 可读时间查询接口 =====
        
        /// 函数级详细中文注释：获取订单详情（含可读时间）
        /// 
        /// ## 功能说明
        /// 为前端提供人可读的时间信息，无需前端自行计算
        /// 
        /// ## 返回字段
        /// - `order_id`: 订单ID
        /// - `created_at`: 创建时间（Unix秒）
        /// - `expire_at`: 过期时间（Unix秒）
        /// - `remaining_seconds`: 剩余秒数（0表示已过期）
        /// - `remaining_readable`: 可读剩余时间（如 "45m", "1h 30m"）
        /// - `state`: 订单状态
        pub fn get_order_with_time(order_id: u64) -> Option<OrderTimeInfo<T>> {
            let order = Orders::<T>::get(order_id)?;
            let now = T::Timestamp::now().as_secs().saturated_into::<u64>();
            
            let remaining_seconds = if order.expire_at > now {
                order.expire_at.saturating_sub(now)
            } else {
                0
            };
            
            Some(OrderTimeInfo {
                order_id,
                maker_id: order.maker_id,
                buyer: order.taker.clone(),
                dust_amount: order.qty,
                usdt_amount: order.amount,
                created_at: order.created_at,
                expire_at: order.expire_at,
                remaining_seconds,
                remaining_readable: pallet_trading_common::format_duration(remaining_seconds),
                state: Self::state_to_u8(&order.state),
                is_expired: remaining_seconds == 0 && order.state == OrderState::Created,
            })
        }
        
        /// 函数级详细中文注释：批量获取用户订单（含可读时间）
        pub fn get_buyer_orders_with_time(who: &T::AccountId) -> sp_std::vec::Vec<OrderTimeInfo<T>> {
            BuyerOrders::<T>::get(who)
                .iter()
                .filter_map(|&order_id| Self::get_order_with_time(order_id))
                .collect()
        }
        
        /// 函数级详细中文注释：将订单状态转换为 u8（用于事件）
        fn state_to_u8(state: &OrderState) -> u8 {
            match state {
                OrderState::Created => 0,
                OrderState::PaidOrCommitted => 1,
                OrderState::Released => 2,
                OrderState::Refunded => 3,
                OrderState::Canceled => 4,
                OrderState::Disputed => 5,
                OrderState::Closed => 6,
                OrderState::Expired => 7,
            }
        }
        
        // ===== 🆕 2026-01-18: 买家押金计算 =====
        
        /// 函数级详细中文注释：计算买家应缴押金
        /// 
        /// ## 押金规则
        /// - 首购用户：免押金
        /// - 信用用户（≥70分，≥5单）：免押金
        /// - 普通用户（50-69分）：3%
        /// - 低信用用户（30-49分）：5%
        /// - 高风险用户（<30分）：10%
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `order_amount`: 订单 DUST 金额
        /// 
        /// ## 返回
        /// - 应缴押金金额（0 表示免押金）
        pub fn calculate_buyer_deposit(
            buyer: &T::AccountId,
            order_amount: BalanceOf<T>,
        ) -> BalanceOf<T> {
            use sp_runtime::traits::Zero;
            
            // 1. 首购用户免押金
            if !HasFirstPurchased::<T>::get(buyer) {
                return Zero::zero();
            }
            
            // 2. 获取买家完成订单数（作为信用评估依据）
            let completed_orders = BuyerCompletedOrderCount::<T>::get(buyer);
            
            // 简化信用分计算：基于完成订单数
            // 0单 = 30分, 1-2单 = 40分, 3-4单 = 50分, 5-9单 = 60分, 10+单 = 80分
            let credit_score: u16 = if completed_orders >= 10 {
                80
            } else if completed_orders >= 5 {
                60
            } else if completed_orders >= 3 {
                50
            } else if completed_orders >= 1 {
                40
            } else {
                30
            };
            
            // 3. 信用用户免押金（≥70分 且 ≥5单）
            if credit_score >= T::CreditScoreExempt::get() 
                && completed_orders >= T::MinOrdersForExempt::get() 
            {
                return Zero::zero();
            }
            
            // 4. 根据信用分计算押金比例（bps）
            let deposit_rate_bps: u16 = if credit_score >= 50 {
                T::DepositRateLow::get()      // 3% = 300 bps
            } else if credit_score >= 30 {
                T::DepositRateMedium::get()   // 5% = 500 bps
            } else {
                T::DepositRateHigh::get()     // 10% = 1000 bps
            };
            
            // 5. 计算押金金额 = order_amount * rate / 10000
            let deposit_rate_balance: BalanceOf<T> = deposit_rate_bps.into();
            let divisor: BalanceOf<T> = 10000u32.into();
            let deposit = order_amount * deposit_rate_balance / divisor;
            
            // 6. 确保不低于最小押金
            let min_deposit = T::MinDeposit::get();
            if deposit < min_deposit {
                min_deposit
            } else {
                deposit
            }
        }
        
        /// 函数级详细中文注释：锁定买家押金到押金池
        /// 
        /// ## 功能说明
        /// 从买家账户扣除押金，转入押金池账户
        /// 
        /// ## 参数
        /// - `buyer`: 买家账户
        /// - `amount`: 押金金额
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(InsufficientDepositBalance)`: 余额不足
        fn lock_buyer_deposit(
            buyer: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;
            use frame_support::traits::ExistenceRequirement;
            
            if amount.is_zero() {
                return Ok(());
            }
            
            // 从买家账户转账到押金池
            T::Currency::transfer(
                buyer,
                &Self::deposit_pool_account(),
                amount,
                ExistenceRequirement::KeepAlive,
            ).map_err(|_| Error::<T>::InsufficientDepositBalance)?;
            
            // 更新押金池总余额
            TotalDepositPoolBalance::<T>::mutate(|balance| {
                *balance = *balance + amount;
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：释放买家押金（退还给买家）
        fn release_buyer_deposit(
            buyer: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;
            use frame_support::traits::ExistenceRequirement;
            
            if amount.is_zero() {
                return Ok(());
            }
            
            // 从押金池转账到买家
            T::Currency::transfer(
                &Self::deposit_pool_account(),
                buyer,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;
            
            // 更新押金池总余额
            TotalDepositPoolBalance::<T>::mutate(|balance| {
                if *balance >= amount {
                    *balance = *balance - amount;
                } else {
                    *balance = Zero::zero();
                }
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：没收买家押金（转给做市商）
        fn forfeit_buyer_deposit(
            maker: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            use sp_runtime::traits::Zero;
            use frame_support::traits::ExistenceRequirement;
            
            if amount.is_zero() {
                return Ok(());
            }
            
            // 从押金池转账到做市商
            T::Currency::transfer(
                &Self::deposit_pool_account(),
                maker,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;
            
            // 更新押金池总余额
            TotalDepositPoolBalance::<T>::mutate(|balance| {
                if *balance >= amount {
                    *balance = *balance - amount;
                } else {
                    *balance = Zero::zero();
                }
            });
            
            Ok(())
        }
        
        /// 函数级详细中文注释：获取押金池账户（PDA，无私钥）
        fn deposit_pool_account() -> T::AccountId {
            // 使用 pallet 模块名作为种子生成 PDA
            let entropy = (b"otc/deposit", ).using_encoded(sp_core::hashing::blake2_256);
            T::AccountId::decode(&mut &entropy[..]).expect("valid account id")
        }
        
        // ===== 🆕 2026-01-18: 自动过期处理 =====
        
        /// 函数级详细中文注释：处理过期订单
        /// 
        /// ## 功能说明
        /// 1. 遍历最近的订单（最多检查100个）
        /// 2. 找出 Created 状态且已超时的订单
        /// 3. 执行过期处理（退款、释放额度）
        /// 4. 每次最多处理10个订单，避免区块过重
        /// 
        /// ## 返回
        /// - `Weight`: 消耗的权重
        pub fn process_expired_orders() -> Weight {
            let mut processed = 0u32;
            let max_per_block = 10u32; // 每次最多处理10个
            let max_check = 100u64;    // 每次最多检查100个订单
            
            let next_id = NextOrderId::<T>::get();
            let start_id = next_id.saturating_sub(max_check);
            let now_secs = T::Timestamp::now().as_secs().saturated_into::<u64>();
            
            for order_id in start_id..next_id {
                if processed >= max_per_block {
                    break;
                }
                
                if let Some(order) = Orders::<T>::get(order_id) {
                    // 仅处理 Created 状态的订单
                    if order.state != OrderState::Created {
                        continue;
                    }
                    
                    // 检查是否已过期
                    if now_secs > order.expire_at {
                        // 执行过期处理
                        if Self::do_expire_order(order_id, &order).is_ok() {
                            processed += 1;
                        }
                    }
                }
            }
            
            // 发出批量处理事件
            if processed > 0 {
                Self::deposit_event(Event::ExpiredOrdersProcessed {
                    count: processed,
                    block_number: <frame_system::Pallet<T>>::block_number(),
                });
            }
            
            // 返回消耗的权重
            Weight::from_parts((processed as u64) * 100_000 + 10_000, 0)
        }
        
        /// 函数级详细中文注释：执行单个订单的过期处理
        /// 
        /// ## 处理步骤
        /// 1. 更新订单状态为 Expired
        /// 2. 退还托管资金给买家
        /// 3. 释放买家占用的额度
        /// 4. 如是首购订单，减少做市商首购计数
        fn do_expire_order(order_id: u64, order: &Order<T>) -> DispatchResult {
            // 1. 更新订单状态
            Orders::<T>::mutate(order_id, |maybe_order| {
                if let Some(o) = maybe_order {
                    o.state = OrderState::Expired;
                }
            });
            
            // 2. 退还托管资金给买家
            let _ = T::Escrow::refund_all(order_id, &order.taker);
            
            // 3. 释放买家占用的额度（amount 是 USDT 金额）
            let usd_amount: u64 = order.amount.saturated_into();
            let _ = T::Credit::release_quota(&order.taker, usd_amount);
            
            // 4. 如是首购订单，减少做市商首购计数
            if order.is_first_purchase {
                MakerFirstPurchaseCount::<T>::mutate(order.maker_id, |count| {
                    *count = count.saturating_sub(1);
                });
            }
            
            // 🆕 2026-01-18: 超时没收买家押金给做市商（100%）
            if !order.buyer_deposit.is_zero() {
                let _ = Self::forfeit_buyer_deposit(&order.maker, order.buyer_deposit);
                
                // 更新押金状态
                Orders::<T>::mutate(order_id, |o| {
                    if let Some(ord) = o {
                        ord.deposit_status = DepositStatus::Forfeited;
                    }
                });
                
                Self::deposit_event(Event::BuyerDepositForfeited {
                    order_id,
                    buyer: order.taker.clone(),
                    maker_id: order.maker_id,
                    forfeited_amount: order.buyer_deposit,
                });
            }
            
            // 5. 发出事件
            Self::deposit_event(Event::OrderAutoExpired {
                order_id,
                buyer: order.taker.clone(),
                maker_id: order.maker_id,
                dust_amount: order.qty,  // qty 是 DUST 数量
            });
            
            Ok(())
        }
        
        // ===== 仲裁支持接口 =====
        
        /// 函数级详细中文注释：检查用户是否有权对订单发起争议
        /// 
        /// ## 权限规则
        /// - 买家（taker）：可以对自己的订单发起争议
        /// - 做市商（maker）：可以对自己参与的订单发起争议
        /// 
        /// ## 参数
        /// - `who`: 发起争议的用户
        /// - `order_id`: 订单ID
        /// 
        /// ## 返回
        /// - `true`: 有权发起争议
        /// - `false`: 无权发起争议
        pub fn can_dispute_order(who: &T::AccountId, order_id: u64) -> bool {
            if let Some(order) = Orders::<T>::get(order_id) {
                // 买家或做市商都可以发起争议
                &order.taker == who || &order.maker == who
            } else {
                false
            }
        }
        
        /// 函数级详细中文注释：应用仲裁裁决到订单
        /// 
        /// ## 裁决类型
        /// - Release: 全额放款给做市商（买家败诉）
        /// - Refund: 全额退款给买家（做市商败诉）
        /// - Partial(bps): 按比例分账（双方都有责任）
        /// 
        /// ## 参数
        /// - `order_id`: 订单ID
        /// - `decision`: 仲裁裁决
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 失败
        pub fn apply_arbitration_decision(
            order_id: u64,
            decision: pallet_arbitration::pallet::Decision,
        ) -> DispatchResult {
            // 获取订单记录
            let mut order = Orders::<T>::get(order_id)
                .ok_or(Error::<T>::OrderNotFound)?;
            
            // 确保状态是 Disputed（争议中）
            ensure!(
                order.state == OrderState::Disputed,
                Error::<T>::InvalidOrderStatus
            );
            
            // 根据裁决类型执行相应操作
            use pallet_arbitration::pallet::Decision;
            let maker_win = match decision {
                Decision::Release => {
                    // 放款给做市商（买家败诉）
                    T::Escrow::release_all(order_id, &order.maker)?;
                    order.state = OrderState::Released;
                    true  // 做市商胜诉
                },
                Decision::Refund => {
                    // 退款给买家（做市商败诉）
                    T::Escrow::refund_all(order_id, &order.taker)?;
                    order.state = OrderState::Refunded;
                    false  // 做市商败诉
                },
                Decision::Partial(bps) => {
                    // 按比例分账：bps/10000 给做市商，剩余给买家
                    T::Escrow::split_partial(order_id, &order.maker, &order.taker, bps)?;
                    order.state = OrderState::Released;  // 部分分账视为完成
                    bps >= 5000  // 做市商获得 >= 50% 视为胜诉
                },
            };
            
            // 记录争议结果到信用分 ✅
            let _ = T::MakerCredit::record_maker_dispute_result(
                order.maker_id,
                order_id,
                maker_win,
            );
            
            // 更新订单
            order.completed_at = Some(T::Timestamp::now().as_secs());
            Orders::<T>::insert(order_id, order);
            
            Ok(())
        }

        // ===== 新增：订单金额验证逻辑 =====

        /// 函数级详细中文注释：验证订单金额是否符合限制
        ///
        /// # 参数
        /// - dust_amount: 购买的DUST数量
        /// - is_first_purchase: 是否为首购订单
        ///
        /// # 返回
        /// - Ok(usd_amount): 验证通过，返回对应的USD金额
        /// - Err(DispatchError): 验证失败
        pub fn validate_order_amount(
            dust_amount: BalanceOf<T>,
            is_first_purchase: bool,
        ) -> Result<u64, DispatchError> {
            // 首购订单使用固定价格，无需验证限额
            if is_first_purchase {
                return Ok(T::FirstPurchaseUsdAmount::get());
            }

            // 获取当前DUST/USD价格
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            // 计算订单的USD金额
            let usd_amount = Self::calculate_usd_amount_from_dust(
                dust_amount,
                dust_to_usd_rate,
            )?;

            // 验证最小金额（至少20 USD，首购除外）
            ensure!(
                usd_amount >= T::MinOrderUsdAmount::get(),
                Error::<T>::OrderAmountTooSmall
            );

            // 验证是否超过最大限制
            let max_amount = T::MaxOrderUsdAmount::get();
            ensure!(
                usd_amount <= max_amount,
                Error::<T>::OrderAmountExceedsLimit
            );

            Ok(usd_amount)
        }

        /// 函数级详细中文注释：计算DUST对应的USD金额
        ///
        /// # 参数
        /// - dust_amount: DUST数量
        /// - dust_to_usd_rate: DUST/USD汇率
        ///
        /// # 返回
        /// - Ok(u64): USD金额（精度10^6）
        /// - Err(DispatchError): 计算错误
        fn calculate_usd_amount_from_dust(
            dust_amount: BalanceOf<T>,
            dust_to_usd_rate: BalanceOf<T>,
        ) -> Result<u64, DispatchError> {
            // 转换为u128进行高精度计算
            let dust_u128: u128 = dust_amount.saturated_into();
            let rate_u128: u128 = dust_to_usd_rate.saturated_into();

            // 计算USD金额 = DUST数量 × DUST/USD汇率 ÷ DUST精度
            // DUST精度为10^12，USD精度为10^6
            let usd_u128 = dust_u128
                .checked_mul(rate_u128)
                .ok_or(Error::<T>::AmountCalculationOverflow)?
                .checked_div(1_000_000_000_000u128) // 除以DUST精度10^12
                .ok_or(Error::<T>::AmountCalculationOverflow)?;

            // 验证结果是否在u64范围内
            let usd_amount: u64 = usd_u128
                .try_into()
                .map_err(|_| Error::<T>::AmountCalculationOverflow)?;

            Ok(usd_amount)
        }

        /// 函数级详细中文注释：计算指定USD金额对应的最大DUST数量
        ///
        /// # 参数
        /// - usd_amount: USD金额（精度10^6）
        ///
        /// # 返回
        /// - Ok(BalanceOf<T>): 对应的DUST数量
        /// - Err(DispatchError): 计算错误
        pub fn calculate_max_dust_for_usd_amount(
            usd_amount: u64,
        ) -> Result<BalanceOf<T>, DispatchError> {
            // 获取当前DUST/USD价格
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            // 计算DUST数量 = USD金额 × DUST精度 ÷ DUST/USD汇率
            let usd_u128 = usd_amount as u128;
            let rate_u128: u128 = dust_to_usd_rate.saturated_into();

            let dust_u128 = usd_u128
                .checked_mul(1_000_000_000_000u128) // 乘以DUST精度10^12
                .ok_or(Error::<T>::AmountCalculationOverflow)?
                .checked_div(rate_u128)
                .ok_or(Error::<T>::AmountCalculationOverflow)?;

            // 转换为BalanceOf<T>
            let dust_amount: BalanceOf<T> = dust_u128
                .try_into()
                .map_err(|_| Error::<T>::AmountCalculationOverflow)?;

            Ok(dust_amount)
        }

        /// 函数级详细中文注释：查询当前最大可购买DUST数量
        ///
        /// # 返回
        /// - Ok(BalanceOf<T>): 当前价格下最大可购买的DUST数量
        /// - Err(DispatchError): 查询失败
        pub fn get_max_purchasable_dust() -> Result<BalanceOf<T>, DispatchError> {
            Self::calculate_max_dust_for_usd_amount(T::MaxOrderUsdAmount::get())
        }

        /// 函数级详细中文注释：查询指定DUST数量对应的USD金额
        ///
        /// # 参数
        /// - dust_amount: DUST数量
        ///
        /// # 返回
        /// - Ok(u64): 对应的USD金额
        /// - Err(DispatchError): 查询失败
        pub fn get_usd_amount_for_dust(
            dust_amount: BalanceOf<T>
        ) -> Result<u64, DispatchError> {
            let dust_to_usd_rate = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PricingServiceUnavailable)?;

            Self::calculate_usd_amount_from_dust(dust_amount, dust_to_usd_rate)
        }

        /// 函数级详细中文注释：检查指定DUST数量是否符合订单限制
        ///
        /// # 参数
        /// - dust_amount: 要检查的DUST数量
        ///
        /// # 返回
        /// - true: 符合限制
        /// - false: 超过限制
        pub fn is_dust_amount_valid(dust_amount: BalanceOf<T>) -> bool {
            Self::validate_order_amount(dust_amount, false).is_ok()
        }

        // ========================================
        // 🆕 存储膨胀防护 - 订单归档函数
        // ========================================

        /// 归档已完成订单（每次最多处理 max_count 个）
        ///
        /// 归档条件：
        /// - 订单状态为 Closed, Released, Refunded, Canceled, Expired
        /// - 订单完成时间超过 30 天
        fn archive_completed_orders(max_count: u32) -> Weight {
            let mut cursor = ArchiveCursor::<T>::get();
            let next_id = NextOrderId::<T>::get();
            let mut processed = 0u32;

            // 30天 = 2592000秒
            const ARCHIVE_DELAY_SECS: u64 = 30 * 24 * 60 * 60;
            let now_secs = T::Timestamp::now().as_secs();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(order) = Orders::<T>::get(cursor) {
                    // 检查是否为可归档状态
                    let is_final_state = matches!(
                        order.state,
                        OrderState::Closed | OrderState::Released |
                        OrderState::Refunded | OrderState::Canceled | OrderState::Expired
                    );

                    if !is_final_state {
                        continue;
                    }

                    // 检查完成时间是否超过归档延迟
                    let completed_at = order.completed_at.unwrap_or(order.expire_at);
                    if now_secs.saturating_sub(completed_at) < ARCHIVE_DELAY_SECS {
                        continue;
                    }

                    // 创建归档记录
                    let archived = ArchivedOrder {
                        maker_id: order.maker_id,
                        taker: order.taker.clone(),
                        qty: order.qty.saturated_into(),
                        amount: order.amount.saturated_into(),
                        state: order.state.clone(),
                        completed_at,
                    };

                    // 保存归档并删除原订单
                    ArchivedOrders::<T>::insert(cursor, archived);
                    Orders::<T>::remove(cursor);

                    // 从做市商订单列表中移除
                    MakerOrders::<T>::mutate(order.maker_id, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    // 从买家订单列表中移除
                    BuyerOrders::<T>::mutate(&order.taker, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    processed = processed.saturating_add(1);
                }
            }

            ArchiveCursor::<T>::put(cursor);
            Weight::from_parts(25_000 * processed as u64, 0)
        }

        /// 🆕 L1 归档转 L2（每次最多处理 max_count 个）
        ///
        /// 归档条件：
        /// - L1归档时间超过 90 天
        fn archive_l1_to_l2(max_count: u32) -> Weight {
            let mut cursor = L1ArchiveCursor::<T>::get();
            let next_id = NextOrderId::<T>::get();
            let mut processed = 0u32;

            // 90天 = 7776000秒
            const L2_ARCHIVE_DELAY_SECS: u64 = 90 * 24 * 60 * 60;
            let now_secs = T::Timestamp::now().as_secs();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(archived_l1) = ArchivedOrders::<T>::get(cursor) {
                    // 检查 L1 归档时间是否超过延迟
                    if now_secs.saturating_sub(archived_l1.completed_at) < L2_ARCHIVE_DELAY_SECS {
                        continue;
                    }

                    // 创建 L2 归档记录
                    let archived_l2 = ArchivedOrderL2 {
                        id: cursor,
                        status: Self::order_state_to_u8(&archived_l1.state),
                        year_month: Self::timestamp_to_year_month(archived_l1.completed_at),
                        amount_tier: pallet_storage_lifecycle::amount_to_tier(archived_l1.amount),
                        flags: 0,
                    };

                    // 更新永久统计
                    OtcStats::<T>::mutate(|stats| {
                        stats.total_orders = stats.total_orders.saturating_add(1);
                        if matches!(archived_l1.state, OrderState::Released | OrderState::Closed) {
                            stats.completed_orders = stats.completed_orders.saturating_add(1);
                            stats.total_volume = stats.total_volume.saturating_add(archived_l1.amount);
                        } else {
                            stats.cancelled_orders = stats.cancelled_orders.saturating_add(1);
                        }
                    });

                    // 保存 L2 归档并删除 L1 归档
                    ArchivedOrdersL2::<T>::insert(cursor, archived_l2);
                    ArchivedOrders::<T>::remove(cursor);

                    processed = processed.saturating_add(1);
                }
            }

            L1ArchiveCursor::<T>::put(cursor);
            Weight::from_parts(20_000 * processed as u64, 0)
        }

        /// 辅助函数：OrderState 转 u8
        fn order_state_to_u8(state: &OrderState) -> u8 {
            match state {
                OrderState::Created => 0,
                OrderState::PaidOrCommitted => 1,
                OrderState::Released => 2,
                OrderState::Refunded => 3,
                OrderState::Canceled => 4,
                OrderState::Disputed => 5,
                OrderState::Closed => 6,
                OrderState::Expired => 7,
            }
        }

        /// 辅助函数：时间戳转年月 (YYMM格式)
        fn timestamp_to_year_month(timestamp: u64) -> u16 {
            // 简化计算：假设2024年1月1日为起点
            const BASE_TIMESTAMP: u64 = 1704067200; // 2024-01-01 00:00:00 UTC
            const SECONDS_PER_MONTH: u64 = 30 * 24 * 60 * 60;
            
            let months_since_base = timestamp.saturating_sub(BASE_TIMESTAMP) / SECONDS_PER_MONTH;
            let year = 24 + (months_since_base / 12) as u16;
            let month = (months_since_base % 12 + 1) as u16;
            year * 100 + month
        }
    }
}
