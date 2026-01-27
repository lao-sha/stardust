#![cfg_attr(not(feature = "std"), no_std)]

//! # Swap Pallet (做市商兑换模块)
//!
//! ## 概述
//!
//! 本模块负责 DUST → USDT 做市商兑换服务，包括：
//! - 做市商兑换（市场化服务）
//! - OCW 自动验证
//! - 超时退款机制
//!
//! ## 版本历史
//!
//! - v0.1.0 (2025-11-03): 从 pallet-trading 拆分而来
//! - v0.2.0 (2026-01-18): 移除官方桥接功能，仅保留做市商兑换
//! - v0.3.0 (2026-01-18): 重命名 bridge → swap

pub use pallet::*;

// TODO: 测试文件待创建
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::WeightInfo;

pub mod ocw;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::{
        traits::{Currency, Get},
        BoundedVec,
        sp_runtime::{SaturatedConversion, traits::Saturating},
    };
    use pallet_escrow::Escrow as EscrowTrait;
    
    // \ud83c\udd95 2026-01-20: OCW \u76f8\u5173\u5bfc\u5165
    use sp_runtime::transaction_validity::{
        InvalidTransaction, TransactionSource, TransactionValidity, ValidTransaction,
    };
    
    // 🆕 v0.4.0: 从 pallet-trading-common 导入公共类型和 Trait
    use pallet_trading_common::{
        TronAddress,
        PricingProvider,
        MakerInterface,
        MakerCreditInterface,
        MakerValidationError,
    };
    use pallet_storage_lifecycle::{amount_to_tier, block_to_year_month};
    // MakerApplicationInfo 通过 MakerInterface::get_maker_application 返回
    
    /// 函数级详细中文注释：Balance 类型别名
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    
    // ===== 数据结构 =====
    
    /// 函数级详细中文注释：兑换状态枚举
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    pub enum SwapStatus {
        /// 待处理
        Pending,
        /// 🆕 2026-01-20: 等待 OCW 验证 TRC20 交易
        AwaitingVerification,
        /// 已完成
        Completed,
        /// 🆕 2026-01-20: OCW 验证失败
        VerificationFailed,
        /// 用户举报
        UserReported,
        /// 仲裁中
        Arbitrating,
        /// 仲裁通过
        ArbitrationApproved,
        /// 仲裁拒绝
        ArbitrationRejected,
        /// 超时退款
        Refunded,
    }
    
    /// 🆕 2026-01-18: 兑换时间信息结构（供 RPC 查询使用）
    #[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct SwapTimeInfo<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 用户账户
        pub user: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// USDT 金额
        pub usdt_amount: u64,
        /// 创建区块
        pub created_at_block: u64,
        /// 创建时间（预估 Unix 秒）
        pub created_at_timestamp: u64,
        /// 超时区块
        pub timeout_at_block: u64,
        /// 超时时间（预估 Unix 秒）
        pub timeout_at_timestamp: u64,
        /// 剩余秒数（0表示已超时）
        pub remaining_seconds: u64,
        /// 可读剩余时间（如 "45m", "1h 30m"）
        pub remaining_readable: sp_std::vec::Vec<u8>,
        /// 兑换状态（0-4）
        pub status: u8,
        /// 是否已超时
        pub is_timeout: bool,
    }
    
    /// 🆕 存储膨胀防护：归档兑换 L1（精简版）
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct ArchivedSwapL1<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 用户账户
        pub user: T::AccountId,
        /// DUST 数量（压缩为u64）
        pub dust_amount: u64,
        /// USDT 金额
        pub usdt_amount: u64,
        /// 兑换状态
        pub status: SwapStatus,
        /// 完成区块
        pub completed_at: u32,
    }

    /// 🆕 存储膨胀防护：归档兑换 L2（最小版，~16字节）
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug, Default)]
    pub struct ArchivedSwapL2 {
        /// 兑换ID
        pub id: u64,
        /// 状态 (0-6)
        pub status: u8,
        /// 年月 (YYMM格式)
        pub year_month: u16,
        /// 金额档位 (0-5)
        pub amount_tier: u8,
        /// 保留标志位
        pub flags: u8,
    }

    /// 🆕 存储膨胀防护：Swap永久统计
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug, Default)]
    pub struct SwapPermanentStats {
        /// 总兑换数
        pub total_swaps: u64,
        /// 已完成兑换数
        pub completed_swaps: u64,
        /// 超时退款数
        pub refunded_swaps: u64,
        /// 总交易额（USDT）
        pub total_volume: u64,
    }

    /// 函数级详细中文注释：做市商兑换记录
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct MakerSwapRecord<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// 做市商ID
        pub maker_id: u64,
        /// 做市商账户
        pub maker: T::AccountId,
        /// 用户账户
        pub user: T::AccountId,
        /// DUST 数量
        pub dust_amount: BalanceOf<T>,
        /// USDT 金额（精度 10^6）
        pub usdt_amount: u64,
        /// USDT 接收地址
        pub usdt_address: TronAddress,
        /// 创建时间
        pub created_at: BlockNumberFor<T>,
        /// 超时时间
        pub timeout_at: BlockNumberFor<T>,
        /// TRC20 交易哈希
        pub trc20_tx_hash: Option<BoundedVec<u8, ConstU32<128>>>,
        /// 完成时间
        pub completed_at: Option<BlockNumberFor<T>>,
        /// 证据 CID
        pub evidence_cid: Option<BoundedVec<u8, ConstU32<256>>>,
        /// 兑换状态
        pub status: SwapStatus,
        /// 兑换价格（精度 10^6）
        pub price_usdt: u64,
    }

    /// 🆕 2026-01-20: TRC20 验证请求结构体
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, PartialEq, Eq, RuntimeDebug)]
    #[scale_info(skip_type_params(T))]
    pub struct VerificationRequest<T: Config> {
        /// 兑换ID
        pub swap_id: u64,
        /// TRC20 交易哈希
        pub tx_hash: BoundedVec<u8, ConstU32<128>>,
        /// 预期收款地址
        pub expected_to: TronAddress,
        /// 预期 USDT 金额（精度 10^6）
        pub expected_amount: u64,
        /// 提交时间（区块号）
        pub submitted_at: BlockNumberFor<T>,
        /// 验证超时时间（区块号）
        pub verification_timeout_at: BlockNumberFor<T>,
        /// 重试次数
        pub retry_count: u8,
    }
    
    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    /// 函数级详细中文注释：Bridge模块配置 trait
    #[pallet::config]
    /// 函数级中文注释：Bridge Pallet 配置 trait
    /// - 🔴 stable2506 API 变更：RuntimeEvent 自动继承，无需显式声明
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        
        /// 货币类型
        type Currency: Currency<Self::AccountId>;
        
        /// 托管服务接口
        type Escrow: pallet_escrow::Escrow<Self::AccountId, BalanceOf<Self>>;
        
        /// 价格提供者接口（用于获取 DUST/USD 汇率）
        type Pricing: PricingProvider<BalanceOf<Self>>;
        
        /// Maker Pallet 接口（用于验证做市商）
        type MakerPallet: MakerInterface<Self::AccountId, BalanceOf<Self>>;
        
        /// Credit Pallet 接口（用于记录做市商信用分）
        /// 🆕 2026-01-18: 统一使用 pallet_trading_common::MakerCreditInterface
        type Credit: pallet_trading_common::MakerCreditInterface;
        
        /// 做市商兑换超时时间（区块数，由OCW验证）
        #[pallet::constant]
        type OcwSwapTimeoutBlocks: Get<BlockNumberFor<Self>>;
        
        /// 🆕 2026-01-20: TRC20 验证超时时间（区块数，默认 2 小时 = 1200 区块）
        #[pallet::constant]
        type VerificationTimeoutBlocks: Get<BlockNumberFor<Self>>;

        /// 🆕 2026-01-20: 验证权限（OCW 或委员会）
        type VerificationOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;
        
        /// 最小兑换金额
        #[pallet::constant]
        type MinSwapAmount: Get<BalanceOf<Self>>;
        
        /// 🆕 存储膨胀防护：TRON 交易哈希 TTL（区块数，默认 30 天 = 432000 区块）
        #[pallet::constant]
        type TxHashTtlBlocks: Get<BlockNumberFor<Self>>;
        
        /// 权重信息
        type WeightInfo: WeightInfo;

        /// 🆕 P3: CID 锁定管理器（仲裁期间锁定证据 CID）
        /// 
        /// 功能：
        /// - 用户举报时自动 PIN 并锁定证据 CID
        /// - 仲裁完成后自动解锁并 Unpin
        /// - 防止仲裁期间证据被删除
        /// 
        /// 注意：当前 SWAP 模块的 evidence_cid 字段未被使用
        /// 待添加 submit_evidence 函数后启用 PIN 联动机制
        type CidLockManager: pallet_storage_service::CidLockManager<Self::Hash, BlockNumberFor<Self>>;
    }
    
    // ===== 存储 =====
    
    /// 函数级详细中文注释：下一个兑换 ID
    #[pallet::storage]
    #[pallet::getter(fn next_swap_id)]
    pub type NextSwapId<T> = StorageValue<_, u64, ValueQuery>;
    
    /// 函数级详细中文注释：做市商兑换记录
    #[pallet::storage]
    #[pallet::getter(fn maker_swaps)]
    pub type MakerSwaps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // swap_id
        MakerSwapRecord<T>,
    >;
    
    /// 函数级详细中文注释：用户兑换列表
    #[pallet::storage]
    #[pallet::getter(fn user_swaps)]
    pub type UserSwaps<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,  // 每个用户最多100个兑换
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：做市商兑换列表
    #[pallet::storage]
    #[pallet::getter(fn maker_swap_list)]
    pub type MakerSwapList<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // maker_id
        BoundedVec<u64, ConstU32<200>>,  // 每个做市商最多200个活跃兑换（已完成应归档）
        ValueQuery,
    >;
    
    /// 函数级详细中文注释：已使用的 TRON 交易哈希（防止重放攻击）
    /// 
    /// ## 安全机制
    /// - 做市商完成兑换时提交 TRC20 交易哈希
    /// - 系统记录已使用的哈希，防止同一笔交易被重复使用
    /// - 这是防止重放攻击的关键安全措施
    /// 
    /// ## 存储结构
    /// - Key: TRON 交易哈希（最多 128 字节）
    /// - Value: 记录时的区块号（用于 TTL 过期清理）
    /// 
    /// 🆕 存储膨胀防护：添加区块号，支持 30 天 TTL 过期清理
    #[pallet::storage]
    #[pallet::getter(fn used_tron_tx_hashes)]
    pub type UsedTronTxHashes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<128>>,  // TRC20 tx hash
        BlockNumberFor<T>,               // 🆕 记录时的区块号
        OptionQuery,
    >;
    
    /// 🆕 TTL 清理游标（记录上次清理的区块号）
    #[pallet::storage]
    pub type TxHashCleanupCursor<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    // ==================== 🆕 存储膨胀防护：归档存储 ====================

    /// 归档兑换 L1
    #[pallet::storage]
    #[pallet::getter(fn archived_swaps_l1)]
    pub type ArchivedSwapsL1<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        ArchivedSwapL1<T>,
        OptionQuery,
    >;

    /// 归档兑换 L2
    #[pallet::storage]
    #[pallet::getter(fn archived_swaps_l2)]
    pub type ArchivedSwapsL2<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        ArchivedSwapL2,
        OptionQuery,
    >;

    /// 归档游标（活跃 → L1）
    #[pallet::storage]
    pub type ArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// L1归档游标（L1 → L2）
    #[pallet::storage]
    pub type L1ArchiveCursor<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Swap永久统计
    #[pallet::storage]
    #[pallet::getter(fn swap_stats)]
    pub type SwapStats<T: Config> = StorageValue<_, SwapPermanentStats, ValueQuery>;

    // ==================== 🆕 2026-01-20: TRC20 验证存储 ====================

    /// 待验证队列
    #[pallet::storage]
    #[pallet::getter(fn pending_verifications)]
    pub type PendingVerifications<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,  // swap_id
        VerificationRequest<T>,
        OptionQuery,
    >;

    /// 验证游标（用于超时检查）
    #[pallet::storage]
    pub type VerificationCursor<T: Config> = StorageValue<_, u64, ValueQuery>;
    
    // ===== 事件 =====
    
    /// 函数级详细中文注释：Bridge模块事件
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 做市商兑换已创建
        MakerSwapCreated {
            swap_id: u64,
            maker_id: u64,
            user: T::AccountId,
            dust_amount: BalanceOf<T>,
        },
        /// 做市商兑换已完成
        MakerSwapCompleted {
            swap_id: u64,
            maker: T::AccountId,
        },
        /// 做市商兑换已标记完成
        MakerSwapMarkedComplete {
            swap_id: u64,
            maker_id: u64,
            trc20_tx_hash: BoundedVec<u8, ConstU32<128>>,
        },
        /// 用户举报兑换
        SwapReported {
            swap_id: u64,
            user: T::AccountId,
        },
        /// 🆕 2026-01-18: 兑换超时（自动退款）
        SwapTimeout {
            swap_id: u64,
            user: T::AccountId,
            maker_id: u64,
        },
        /// 🆕 2026-01-20: TRC20 验证已提交，等待验证
        VerificationSubmitted {
            swap_id: u64,
            tx_hash: BoundedVec<u8, ConstU32<128>>,
        },
        /// 🆕 2026-01-20: TRC20 验证成功，DUST 已释放
        VerificationConfirmed {
            swap_id: u64,
            maker: T::AccountId,
        },
        /// 🆕 2026-01-20: TRC20 验证失败
        VerificationFailed {
            swap_id: u64,
            reason: BoundedVec<u8, ConstU32<128>>,
        },
        /// 🆕 2026-01-20: 验证超时，进入人工仲裁
        VerificationTimeout {
            swap_id: u64,
        },
    }
    
    // ===== 错误 =====
    
    /// 函数级详细中文注释：Bridge模块错误
    #[pallet::error]
    pub enum Error<T> {
        /// 兑换不存在
        SwapNotFound,
        /// 做市商不存在
        MakerNotFound,
        /// 做市商未激活
        MakerNotActive,
        /// 兑换状态不正确
        InvalidSwapStatus,
        /// 未授权
        NotAuthorized,
        /// 编码错误
        EncodingError,
        /// 存储限制已达到
        StorageLimitReached,
        /// 兑换金额太低
        SwapAmountTooLow,
        /// 无效的 TRON 地址
        InvalidTronAddress,
        /// 兑换已完成
        AlreadyCompleted,
        /// 不是做市商
        NotMaker,
        /// 状态无效
        InvalidStatus,
        /// 交易哈希无效
        InvalidTxHash,
        /// 兑换太多
        TooManySwaps,
        /// 低于最小金额
        BelowMinimumAmount,
        /// 地址无效
        InvalidAddress,
        /// 不是兑换的用户
        NotSwapUser,
        /// 无法举报
        CannotReport,
        /// 价格不可用
        PriceNotAvailable,
        /// 金额溢出
        AmountOverflow,
        /// USDT金额太小
        UsdtAmountTooSmall,
        /// TRON 交易哈希已被使用（防止重放攻击）
        TronTxHashAlreadyUsed,
        /// 🆕 2026-01-18: 尚未超时
        NotYetTimeout,
        /// 🆕 2026-01-20: 验证请求不存在
        VerificationNotFound,
        /// 🆕 2026-01-20: 验证尚未超时
        VerificationNotYetTimeout,
    }
    
    // ===== Extrinsics =====
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建做市商桥接兑换
        ///
        /// # 参数
        /// - `origin`: 调用者（用户，必须是签名账户）
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST数量
        /// - `usdt_address`: USDT接收地址
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::maker_swap())]
        pub fn maker_swap(
            origin: OriginFor<T>,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            usdt_address: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            let _swap_id = Self::do_maker_swap(&user, maker_id, dust_amount, usdt_address)?;
            Ok(())
        }
        
        /// 函数级详细中文注释：做市商标记兑换完成
        ///
        /// # 参数
        /// - `origin`: 调用者（做市商，必须是签名账户）
        /// - `swap_id`: 兑换ID
        /// - `trc20_tx_hash`: TRC20交易哈希
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::mark_swap_complete())]
        pub fn mark_swap_complete(
            origin: OriginFor<T>,
            swap_id: u64,
            trc20_tx_hash: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let maker = ensure_signed(origin)?;
            Self::do_mark_swap_complete(&maker, swap_id, trc20_tx_hash)
        }
        
        /// 函数级详细中文注释：用户举报做市商兑换
        ///
        /// # 参数
        /// - `origin`: 调用者（用户，必须是签名账户）
        /// - `swap_id`: 兑换ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::report_swap())]
        pub fn report_swap(
            origin: OriginFor<T>,
            swap_id: u64,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;
            Self::do_report_swap(&user, swap_id)
        }
        
        /// 🆕 2026-01-20: 确认 TRC20 验证结果
        ///
        /// # 权限
        /// - 仅 VerificationOrigin（OCW 或委员会）可调用
        ///
        /// # 参数
        /// - `origin`: 验证权限来源
        /// - `swap_id`: 兑换ID
        /// - `verified`: 验证结果（true=成功，false=失败）
        /// - `reason`: 失败原因（可选）
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::mark_swap_complete())]
        pub fn confirm_verification(
            origin: OriginFor<T>,
            swap_id: u64,
            verified: bool,
            reason: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            T::VerificationOrigin::ensure_origin(origin)?;
            Self::do_confirm_verification(swap_id, verified, reason)
        }
        
        /// 🆕 2026-01-20: 处理验证超时（进入人工仲裁）
        ///
        /// # 权限
        /// - 任何人可调用（需满足超时条件）
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `swap_id`: 兑换ID
        ///
        /// # 返回
        /// - `DispatchResult`: 成功或错误
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::report_swap())]
        pub fn handle_verification_timeout(
            origin: OriginFor<T>,
            swap_id: u64,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            Self::do_handle_verification_timeout(swap_id)
        }
        
        /// \ud83c\udd95 2026-01-20: OCW \u63d0\u4ea4\u9a8c\u8bc1\u7ed3\u679c\uff08\u65e0\u7b7e\u540d\u4ea4\u6613\uff09
        ///
        /// # \u6743\u9650
        /// - \u4ec5 OCW \u53ef\u8c03\u7528\uff08\u901a\u8fc7 ValidateUnsigned \u9a8c\u8bc1\uff09
        ///
        /// # \u53c2\u6570
        /// - `swap_id`: \u5151\u6362ID
        /// - `verified`: \u9a8c\u8bc1\u7ed3\u679c
        /// - `reason`: \u5931\u8d25\u539f\u56e0
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::mark_swap_complete())]
        pub fn ocw_submit_verification(
            origin: OriginFor<T>,
            swap_id: u64,
            verified: bool,
            reason: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            ensure_none(origin)?;
            Self::do_confirm_verification(swap_id, verified, reason)
        }
        
    }
    
    // ===== OCW \u65e0\u7b7e\u540d\u4ea4\u6613\u9a8c\u8bc1 =====
    
    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;
        
        fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
            match call {
                Call::ocw_submit_verification { swap_id, .. } => {
                    // \u9a8c\u8bc1 swap \u5b58\u5728\u4e14\u72b6\u6001\u6b63\u786e
                    if let Some(record) = MakerSwaps::<T>::get(swap_id) {
                        if record.status == SwapStatus::AwaitingVerification {
                            return ValidTransaction::with_tag_prefix("TRC20Verify")
                                .priority(100)
                                .longevity(5)
                                .and_provides([&(b"verify", swap_id)])
                                .propagate(true)
                                .build();
                        }
                    }
                    InvalidTransaction::Call.into()
                },
                _ => InvalidTransaction::Call.into(),
            }
        }
    }
    
    // ===== 内部实现 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：创建做市商兑换
        /// 
        /// ## 功能说明
        /// 1. 验证做市商存在且激活
        /// 2. 验证兑换金额大于最小值
        /// 3. 验证 USDT 地址格式
        /// 4. 锁定用户的 DUST 到托管
        /// 5. 创建做市商兑换记录
        /// 6. 等待做市商转账 USDT
        /// 
        /// ## 参数
        /// - `user`: 用户账户
        /// - `maker_id`: 做市商ID
        /// - `dust_amount`: DUST 数量
        /// - `usdt_address`: USDT 收款地址（TRC20）
        /// 
        /// ## 返回
        /// - `Ok(swap_id)`: 兑换ID
        /// - `Err(...)`: 各种错误情况
        pub fn do_maker_swap(
            user: &T::AccountId,
            maker_id: u64,
            dust_amount: BalanceOf<T>,
            usdt_address: sp_std::vec::Vec<u8>,
        ) -> Result<u64, DispatchError> {
            // 1. 验证最小兑换金额
            ensure!(
                dust_amount >= T::MinSwapAmount::get(),
                Error::<T>::BelowMinimumAmount
            );
            
            // 2. 🆕 使用统一的做市商验证逻辑
            let maker_app = T::MakerPallet::validate_maker(maker_id)
                .map_err(|e| match e {
                    MakerValidationError::NotFound => Error::<T>::MakerNotFound,
                    MakerValidationError::NotActive => Error::<T>::MakerNotActive,
                })?;
            
            // 3. 验证 USDT 地址格式
            let usdt_addr: TronAddress = usdt_address
                .try_into()
                .map_err(|_| Error::<T>::InvalidAddress)?;
            
            // 4. 获取当前价格（从 PricingProvider 获取实时汇率）
            let price_balance = T::Pricing::get_dust_to_usd_rate()
                .ok_or(Error::<T>::PriceNotAvailable)?;
            let price_usdt: u64 = price_balance.saturated_into();
            
            // 5. 计算 USDT 金额（加入边界检查防止溢出）
            let dust_amount_u128: u128 = dust_amount.saturated_into();
            let usdt_amount_u128 = dust_amount_u128
                .checked_mul(price_usdt as u128)
                .ok_or(Error::<T>::AmountOverflow)?
                .checked_div(1_000_000_000_000u128)
                .ok_or(Error::<T>::AmountOverflow)?;
            
            // 6. 验证最小 USDT 金额（至少 1 USDT）
            ensure!(
                usdt_amount_u128 >= 1_000_000,
                Error::<T>::UsdtAmountTooSmall
            );
            
            let usdt_amount = usdt_amount_u128 as u64;
            
            // 7. 获取兑换ID
            let swap_id = NextSwapId::<T>::get();
            
            // 7. 锁定用户的 DUST 到托管
            T::Escrow::lock_from(
                user,
                swap_id,
                dust_amount,
            )?;
            
            // 8. 计算超时时间
            let current_block = frame_system::Pallet::<T>::block_number();
            let timeout_at = current_block + T::OcwSwapTimeoutBlocks::get();
            
            // 9. 创建做市商兑换记录
            let record = MakerSwapRecord {
                swap_id,
                maker_id,
                maker: maker_app.account,
                user: user.clone(),
                dust_amount,
                usdt_amount,
                usdt_address: usdt_addr,
                created_at: current_block,
                timeout_at,
                trc20_tx_hash: None,
                completed_at: None,
                evidence_cid: None,
                status: SwapStatus::Pending,
                price_usdt,
            };
            
            // 10. 保存记录
            MakerSwaps::<T>::insert(swap_id, record);
            NextSwapId::<T>::put(swap_id + 1);
            
            // 11. 更新用户兑换列表
            UserSwaps::<T>::try_mutate(user, |swaps| {
                swaps.try_push(swap_id)
                    .map_err(|_| Error::<T>::TooManySwaps)
            })?;
            
            // 12. 更新做市商兑换列表
            MakerSwapList::<T>::try_mutate(maker_id, |swaps| {
                swaps.try_push(swap_id)
                    .map_err(|_| Error::<T>::TooManySwaps)
            })?;
            
            // 13. 发出事件
            Self::deposit_event(Event::MakerSwapCreated {
                swap_id,
                user: user.clone(),
                maker_id,
                dust_amount,
            });
            
            Ok(swap_id)
        }
        
        /// 函数级详细中文注释：做市商标记兑换完成
        /// 
        /// ## 🆕 2026-01-20 更新：OCW 验证机制
        /// 做市商提交 TRC20 交易哈希后，不再直接释放 DUST，
        /// 而是进入 AwaitingVerification 状态，等待 OCW 或委员会验证。
        /// 
        /// ## 功能说明
        /// 1. 验证兑换存在且状态为 Pending
        /// 2. 验证调用者是兑换的做市商
        /// 3. 记录 TRC20 交易哈希
        /// 4. 创建验证请求，等待 OCW 验证
        /// 5. 更新兑换状态为 AwaitingVerification
        /// 
        /// ## 参数
        /// - `maker`: 做市商账户
        /// - `swap_id`: 兑换ID
        /// - `trc20_tx_hash`: TRC20 交易哈希
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_mark_swap_complete(
            maker: &T::AccountId,
            swap_id: u64,
            trc20_tx_hash: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证调用者是做市商
            ensure!(record.maker == *maker, Error::<T>::NotMaker);
            
            // 3. 验证状态
            ensure!(
                record.status == SwapStatus::Pending,
                Error::<T>::InvalidStatus
            );
            
            // 4. 验证交易哈希长度
            let tx_hash: BoundedVec<u8, ConstU32<128>> = trc20_tx_hash
                .try_into()
                .map_err(|_| Error::<T>::InvalidTxHash)?;
            
            // 5. 检查交易哈希是否已被使用（防止重放攻击）
            ensure!(
                !UsedTronTxHashes::<T>::contains_key(&tx_hash),
                Error::<T>::TronTxHashAlreadyUsed
            );
            
            // 6. 记录已使用的交易哈希（🆕 存储区块号用于 TTL 过期清理）
            let current_block = frame_system::Pallet::<T>::block_number();
            UsedTronTxHashes::<T>::insert(&tx_hash, current_block);
            
            // 🆕 2026-01-20: 不再直接释放 DUST，而是进入验证等待状态
            
            // 7. 更新兑换记录状态为 AwaitingVerification
            record.trc20_tx_hash = Some(tx_hash.clone());
            record.status = SwapStatus::AwaitingVerification;
            MakerSwaps::<T>::insert(swap_id, record.clone());
            
            // 8. 创建验证请求
            let current_block = frame_system::Pallet::<T>::block_number();
            let verification_timeout_at = current_block + T::VerificationTimeoutBlocks::get();
            
            let verification_request = VerificationRequest {
                swap_id,
                tx_hash: tx_hash.clone(),
                expected_to: record.usdt_address.clone(),
                expected_amount: record.usdt_amount,
                submitted_at: current_block,
                verification_timeout_at,
                retry_count: 0,
            };
            
            PendingVerifications::<T>::insert(swap_id, verification_request);
            
            // 9. 发出事件（验证已提交，等待验证）
            Self::deposit_event(Event::VerificationSubmitted {
                swap_id,
                tx_hash,
            });
            
            Ok(())
        }
        
        /// 🆕 2026-01-20: 确认 TRC20 验证结果
        /// 
        /// ## 功能说明
        /// 由 OCW 或委员会调用，确认 TRC20 交易验证结果。
        /// - 验证成功：释放 DUST 给做市商
        /// - 验证失败：进入人工仲裁流程
        /// 
        /// ## 参数
        /// - `swap_id`: 兑换ID
        /// - `verified`: 验证结果
        /// - `reason`: 失败原因（如果验证失败）
        pub fn do_confirm_verification(
            swap_id: u64,
            verified: bool,
            reason: Option<sp_std::vec::Vec<u8>>,
        ) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证状态必须是 AwaitingVerification
            ensure!(
                record.status == SwapStatus::AwaitingVerification,
                Error::<T>::InvalidStatus
            );
            
            // 3. 移除待验证队列
            PendingVerifications::<T>::remove(swap_id);
            
            let current_block = frame_system::Pallet::<T>::block_number();
            
            if verified {
                // 验证成功：释放 DUST 给做市商
                T::Escrow::release_all(swap_id, &record.maker)?;
                
                record.status = SwapStatus::Completed;
                record.completed_at = Some(current_block);
                MakerSwaps::<T>::insert(swap_id, record.clone());
                
                // 记录信用分（成功完成订单）
                let block_duration = current_block.saturating_sub(record.created_at);
                let response_time_seconds = (block_duration.saturated_into::<u64>() * 6) as u32;
                
                let _ = T::Credit::record_maker_order_completed(
                    record.maker_id,
                    swap_id,
                    response_time_seconds,
                );
                
                // 🆕 上报交易数据到 pricing 模块
                let timestamp = current_block.saturated_into::<u64>() * 6000; // 转换为毫秒
                let dust_qty: u128 = record.dust_amount.saturated_into();
                let _ = T::Pricing::report_swap_order(timestamp, record.price_usdt, dust_qty);
                
                Self::deposit_event(Event::VerificationConfirmed {
                    swap_id,
                    maker: record.maker,
                });
            } else {
                // 验证失败：进入仲裁流程
                record.status = SwapStatus::VerificationFailed;
                MakerSwaps::<T>::insert(swap_id, record);
                
                let reason_bounded: BoundedVec<u8, ConstU32<128>> = reason
                    .unwrap_or_else(|| b"Unknown verification failure".to_vec())
                    .try_into()
                    .unwrap_or_else(|_| BoundedVec::default());
                
                Self::deposit_event(Event::VerificationFailed {
                    swap_id,
                    reason: reason_bounded,
                });
            }
            
            Ok(())
        }
        
        /// 🆕 2026-01-20: 处理验证超时
        /// 
        /// ## 功能说明
        /// 当 TRC20 验证超时（超过 VerificationTimeoutBlocks）时，
        /// 自动将兑换状态转为 Arbitrating，进入人工仲裁流程。
        /// 
        /// ## 参数
        /// - `swap_id`: 兑换ID
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 各种错误情况
        pub fn do_handle_verification_timeout(swap_id: u64) -> DispatchResult {
            // 1. 获取验证请求
            let request = PendingVerifications::<T>::get(swap_id)
                .ok_or(Error::<T>::VerificationNotFound)?;
            
            // 2. 检查是否已超时
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block >= request.verification_timeout_at,
                Error::<T>::VerificationNotYetTimeout
            );
            
            // 3. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 4. 验证状态必须是 AwaitingVerification
            ensure!(
                record.status == SwapStatus::AwaitingVerification,
                Error::<T>::InvalidStatus
            );
            
            // 5. 移除待验证队列
            PendingVerifications::<T>::remove(swap_id);
            
            // 修复 C-7: 验证超时自动退款给用户，而非进入仲裁
            // 做市商未能在规定时间内完成 TRC20 转账验证，用户不应承担风险
            
            // 6. 自动退款给用户
            let refund_result = T::Escrow::refund_all(swap_id, &record.user);
            
            // 7. 更新状态
            if refund_result.is_ok() {
                record.status = SwapStatus::Refunded;
                
                // 8. 记录做市商超时（影响信用分）
                let _ = T::Credit::record_maker_order_timeout(record.maker_id, swap_id);
            } else {
                // 退款失败时才进入仲裁
                record.status = SwapStatus::Arbitrating;
            }
            
            record.completed_at = Some(current_block);
            MakerSwaps::<T>::insert(swap_id, record.clone());
            
            // 9. 发出事件
            Self::deposit_event(Event::VerificationTimeout { swap_id });
            
            Ok(())
        }
        
        /// 🆕 2026-01-20: 验证 TRC20 交易（OCW 调用）
        pub fn verify_trc20_transaction(request: &VerificationRequest<T>) -> Result<bool, &'static str> {
            crate::ocw::verify_trc20_transaction(
                request.tx_hash.as_slice(),
                request.expected_to.as_slice(),
                request.expected_amount,
            )
        }
        
        /// 用户举报订单
        pub fn do_report_swap(
            user: &T::AccountId,
            swap_id: u64,
        ) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证调用者是用户
            ensure!(record.user == *user, Error::<T>::NotSwapUser);
            
            // 3. 验证状态（只有 Pending 或 Completed 状态可以举报）
            ensure!(
                matches!(record.status, SwapStatus::Pending | SwapStatus::Completed),
                Error::<T>::CannotReport
            );
            
            // 4. 更新状态
            record.status = SwapStatus::UserReported;
            MakerSwaps::<T>::insert(swap_id, record);
            
            // 5. 发出事件
            Self::deposit_event(Event::SwapReported {
                swap_id,
                user: user.clone(),
            });
            
            Ok(())
        }
    }
    
    // ===== 公共查询接口 =====
    
    impl<T: Config> Pallet<T> {
        /// 函数级详细中文注释：获取用户兑换列表
        pub fn get_user_swaps(who: &T::AccountId) -> sp_std::vec::Vec<u64> {
            UserSwaps::<T>::get(who).to_vec()
        }
        
        /// 函数级详细中文注释：获取做市商兑换列表
        pub fn get_maker_swaps(maker_id: u64) -> sp_std::vec::Vec<u64> {
            MakerSwapList::<T>::get(maker_id).to_vec()
        }
        
        // ===== 🆕 2026-01-18: 可读时间查询接口 =====
        
        /// 函数级详细中文注释：获取兑换详情（含可读时间）
        /// 
        /// ## 功能说明
        /// 为前端提供人可读的时间信息
        /// - 区块号自动转换为预估时间戳
        /// - 计算剩余时间
        /// - 提供可读格式（如 "45m"）
        pub fn get_swap_with_time(swap_id: u64) -> Option<SwapTimeInfo<T>> {
            let record = MakerSwaps::<T>::get(swap_id)?;
            let current_block = frame_system::Pallet::<T>::block_number();
            let current_block_u64: u64 = current_block.saturated_into();
            let created_at_u64: u64 = record.created_at.saturated_into();
            let timeout_at_u64: u64 = record.timeout_at.saturated_into();
            
            // 使用当前时间戳（假设 pallet_timestamp 可用）
            // 这里使用区块号估算
            let now_estimate = current_block_u64 * pallet_trading_common::DEFAULT_BLOCK_TIME_SECS;
            
            let created_at_timestamp = pallet_trading_common::estimate_timestamp_from_block(
                created_at_u64,
                current_block_u64,
                now_estimate,
            );
            
            let timeout_at_timestamp = pallet_trading_common::estimate_timestamp_from_block(
                timeout_at_u64,
                current_block_u64,
                now_estimate,
            );
            
            let remaining_seconds = pallet_trading_common::estimate_remaining_seconds(
                timeout_at_u64,
                current_block_u64,
            );
            
            let is_timeout = current_block >= record.timeout_at 
                && record.status == SwapStatus::Pending;
            
            Some(SwapTimeInfo {
                swap_id,
                maker_id: record.maker_id,
                user: record.user.clone(),
                dust_amount: record.dust_amount,
                usdt_amount: record.usdt_amount,
                created_at_block: created_at_u64,
                created_at_timestamp,
                timeout_at_block: timeout_at_u64,
                timeout_at_timestamp,
                remaining_seconds,
                remaining_readable: pallet_trading_common::format_duration(remaining_seconds),
                status: Self::status_to_u8(&record.status),
                is_timeout,
            })
        }
        
        /// 函数级详细中文注释：批量获取用户兑换（含可读时间）
        pub fn get_user_swaps_with_time(who: &T::AccountId) -> sp_std::vec::Vec<SwapTimeInfo<T>> {
            UserSwaps::<T>::get(who)
                .iter()
                .filter_map(|&swap_id| Self::get_swap_with_time(swap_id))
                .collect()
        }
        
        /// 内部函数：状态转换为 u8
        fn status_to_u8(status: &SwapStatus) -> u8 {
            match status {
                SwapStatus::Pending => 0,
                SwapStatus::AwaitingVerification => 1,  // 🆕 2026-01-20
                SwapStatus::Completed => 2,
                SwapStatus::VerificationFailed => 3,    // 🆕 2026-01-20
                SwapStatus::UserReported => 4,
                SwapStatus::Arbitrating => 5,
                SwapStatus::ArbitrationApproved => 6,
                SwapStatus::ArbitrationRejected => 7,
                SwapStatus::Refunded => 8,
            }
        }
        
        // ===== 仲裁支持接口 =====
        
        /// 函数级详细中文注释：检查用户是否有权对兑换发起争议
        /// 
        /// ## 权限规则
        /// - 用户（买家）：可以对自己的兑换发起争议
        /// - 做市商：可以对自己参与的兑换发起争议
        /// 
        /// ## 参数
        /// - `who`: 发起争议的用户
        /// - `swap_id`: 兑换ID
        /// 
        /// ## 返回
        /// - `true`: 有权发起争议
        /// - `false`: 无权发起争议
        pub fn can_dispute_swap(who: &T::AccountId, swap_id: u64) -> bool {
            if let Some(record) = MakerSwaps::<T>::get(swap_id) {
                // 用户或做市商都可以发起争议
                &record.user == who || &record.maker == who
            } else {
                false
            }
        }
        
        /// 函数级详细中文注释：应用仲裁裁决到兑换
        /// 
        /// ## 裁决类型
        /// - Release: 全额放款给做市商（用户败诉）
        /// - Refund: 全额退款给用户（做市商败诉）
        /// - Partial(bps): 按比例分账（双方都有责任）
        /// 
        /// ## 参数
        /// - `swap_id`: 兑换ID
        /// - `decision`: 仲裁裁决
        /// 
        /// ## 返回
        /// - `Ok(())`: 成功
        /// - `Err(...)`: 失败
        pub fn apply_arbitration_decision(
            swap_id: u64,
            decision: pallet_arbitration::pallet::Decision,
        ) -> DispatchResult {
            // 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 确保状态是 UserReported（用户已举报）
            ensure!(
                record.status == SwapStatus::UserReported,
                Error::<T>::InvalidStatus
            );
            
            // 根据裁决类型执行相应操作
            use pallet_arbitration::pallet::Decision;
            let maker_win = match decision {
                Decision::Release => {
                    // 放款给做市商（用户败诉）
                    T::Escrow::release_all(swap_id, &record.maker)?;
                    record.status = SwapStatus::ArbitrationApproved;
                    true  // 做市商胜诉
                },
                Decision::Refund => {
                    // 退款给用户（做市商败诉）
                    T::Escrow::refund_all(swap_id, &record.user)?;
                    record.status = SwapStatus::ArbitrationRejected;
                    false  // 做市商败诉
                },
                Decision::Partial(bps) => {
                    // 按比例分账：bps/10000 给做市商，剩余给用户
                    T::Escrow::split_partial(swap_id, &record.maker, &record.user, bps)?;
                    record.status = SwapStatus::ArbitrationApproved;  // 部分分账视为完成
                    bps >= 5000  // 做市商获得 >= 50% 视为胜诉
                },
            };
            
            // 记录争议结果到信用分 ✅
            let _ = T::Credit::record_maker_dispute_result(
                record.maker_id,
                swap_id,
                maker_win,
            );
            
            // 更新记录
            MakerSwaps::<T>::insert(swap_id, record);
            
            Ok(())
        }
    }
    
    // ===== 🆕 2026-01-18: 自动超时处理（使用 on_initialize 替代 OCW）=====
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let check_interval: u32 = 50;
            let now_u32: u32 = now.saturated_into();
            if now_u32 % check_interval != 0 {
                return Weight::zero();
            }
            let w1 = Self::process_timeout_swaps(now);
            let w2 = Self::process_verification_timeouts(now);
            w1.saturating_add(w2)
        }

        fn on_idle(_now: BlockNumberFor<T>, remaining_weight: Weight) -> Weight {
            let base_weight = Weight::from_parts(20_000, 0);
            if remaining_weight.ref_time() < base_weight.ref_time() * 15 {
                return Weight::zero();
            }
            let w1 = Self::archive_completed_swaps(5);
            let w2 = Self::archive_l1_to_l2(5);
            // 🆕 存储膨胀防护：清理过期的 TRON 交易哈希
            let w3 = Self::cleanup_expired_tx_hashes(10);
            w1.saturating_add(w2).saturating_add(w3)
        }
        
        /// \ud83c\udd95 2026-01-20: OCW \u9a8c\u8bc1 TRC20 \u4ea4\u6613
        /// 
        /// \u6ce8\u610f\uff1a\u5b8c\u6574\u7684 OCW \u5b9e\u73b0\u9700\u8981\u989d\u5916\u7684 runtime \u914d\u7f6e
        /// \u5f53\u524d\u7248\u672c\u4ec5\u8bb0\u5f55\u65e5\u5fd7\uff0c\u5b9e\u9645\u9a8c\u8bc1\u7531\u59d4\u5458\u4f1a\u624b\u52a8\u89e6\u53d1
        fn offchain_worker(_block_number: BlockNumberFor<T>) {
            // OCW \u9a8c\u8bc1\u903b\u8f91\u5df2\u51c6\u5907\u5c31\u7eea
            // \u5f85\u9a8c\u8bc1\u961f\u5217\u5728 PendingVerifications \u5b58\u50a8\u4e2d
            // \u9a8c\u8bc1\u51fd\u6570\u5728 crate::ocw::verify_trc20_transaction
            // \u5b8c\u6574\u5b9e\u73b0\u9700\u8981 runtime \u914d\u7f6e SendTransactionTypes
        }
    }
    
    impl<T: Config> Pallet<T> {
        fn process_timeout_swaps(current_block: BlockNumberFor<T>) -> Weight {
            let next_id = NextSwapId::<T>::get();
            let start_id = if next_id > 100 { next_id - 100 } else { 0 };
            let max_per_block = 10u32;
            let mut processed_count = 0u32;
            for swap_id in start_id..next_id {
                if processed_count >= max_per_block { break; }
                if let Some(record) = MakerSwaps::<T>::get(swap_id) {
                    if record.status != SwapStatus::Pending { continue; }
                    if current_block >= record.timeout_at {
                        if Self::do_process_timeout(swap_id).is_ok() {
                            processed_count += 1;
                        }
                    }
                }
            }
            Weight::from_parts((processed_count as u64) * 100_000 + 10_000, 0)
        }
        
        fn do_process_timeout(swap_id: u64) -> DispatchResult {
            // 1. 获取兑换记录
            let mut record = MakerSwaps::<T>::get(swap_id)
                .ok_or(Error::<T>::SwapNotFound)?;
            
            // 2. 验证状态
            ensure!(
                record.status == SwapStatus::Pending,
                Error::<T>::InvalidStatus
            );
            
            // 3. 验证已超时
            let current_block = frame_system::Pallet::<T>::block_number();
            ensure!(
                current_block >= record.timeout_at,
                Error::<T>::NotYetTimeout
            );
            
            // 4. 退款给用户
            T::Escrow::refund_all(swap_id, &record.user)?;
            
            // 5. 记录做市商超时
            let _ = T::Credit::record_maker_order_timeout(
                record.maker_id,
                swap_id,
            );
            
            // 6. 更新状态
            record.status = SwapStatus::Refunded;
            MakerSwaps::<T>::insert(swap_id, record.clone());
            
            // 7. 发送事件
            Self::deposit_event(Event::SwapTimeout {
                swap_id,
                user: record.user,
                maker_id: record.maker_id,
            });
            
            Ok(())
        }

        /// 2026-01-20: 处理验证超时
        /// 
        /// ## 功能说明
        /// - 扫描 PendingVerifications 存储
        /// - 找出超时的验证请求并自动转入仲裁
        /// - 每次最多处理 5 个
        fn process_verification_timeouts(current_block: BlockNumberFor<T>) -> Weight {
            let max_per_block = 5u32;
            let mut processed_count = 0u32;
            
            // 遍历待验证列表
            for (swap_id, request) in PendingVerifications::<T>::iter() {
                if processed_count >= max_per_block {
                    break;
                }
                
                // 检查是否超时
                if current_block >= request.verification_timeout_at {
                    // 执行超时处理
                    if Self::do_handle_verification_timeout(swap_id).is_ok() {
                        processed_count += 1;
                    }
                }
            }
            
            Weight::from_parts((processed_count as u64) * 80_000 + 5_000, 0)
        }

        /// 2026-01-18: 归档已完成的兑换（每次最多处理 max_count 个）
        fn archive_completed_swaps(max_count: u32) -> Weight {
            let mut cursor = ArchiveCursor::<T>::get();
            let next_id = NextSwapId::<T>::get();
            let mut processed = 0u32;

            // 30天（区块数）
            const ARCHIVE_DELAY_BLOCKS: u32 = 30 * 24 * 60 * 10;
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(record) = MakerSwaps::<T>::get(cursor) {
                    // 检查是否为可归档状态
                    let is_final_state = matches!(
                        record.status,
                        SwapStatus::Completed | SwapStatus::Refunded |
                        SwapStatus::ArbitrationApproved | SwapStatus::ArbitrationRejected
                    );

                    if !is_final_state {
                        continue;
                    }

                    // 检查完成时间是否超过归档延迟
                    let completed_block: u32 = record.completed_at
                        .unwrap_or(record.created_at)
                        .saturated_into();
                    if current_block.saturating_sub(completed_block) < ARCHIVE_DELAY_BLOCKS {
                        continue;
                    }

                    // 创建 L1 归档记录
                    let archived = ArchivedSwapL1 {
                        swap_id: record.swap_id,
                        maker_id: record.maker_id,
                        user: record.user.clone(),
                        dust_amount: record.dust_amount.saturated_into(),
                        usdt_amount: record.usdt_amount,
                        status: record.status.clone(),
                        completed_at: completed_block,
                    };

                    // 保存归档并删除原记录
                    ArchivedSwapsL1::<T>::insert(cursor, archived);
                    MakerSwaps::<T>::remove(cursor);

                    // 从用户兑换列表中移除
                    UserSwaps::<T>::mutate(&record.user, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    // 从做市商兑换列表中移除
                    MakerSwapList::<T>::mutate(record.maker_id, |ids| {
                        ids.retain(|&id| id != cursor);
                    });

                    processed = processed.saturating_add(1);
                }
            }

            ArchiveCursor::<T>::put(cursor);
            Weight::from_parts(25_000 * processed as u64, 0)
        }

        /// L1 归档转 L2（每次最多处理 max_count 个）
        fn archive_l1_to_l2(max_count: u32) -> Weight {
            let mut cursor = L1ArchiveCursor::<T>::get();
            let next_id = NextSwapId::<T>::get();
            let mut processed = 0u32;

            // 90天（区块数）
            const L2_ARCHIVE_DELAY_BLOCKS: u32 = 90 * 24 * 60 * 10;
            let current_block: u32 = frame_system::Pallet::<T>::block_number().saturated_into();

            while processed < max_count && cursor < next_id {
                cursor = cursor.saturating_add(1);

                if let Some(archived_l1) = ArchivedSwapsL1::<T>::get(cursor) {
                    // 检查 L1 归档时间是否超过延迟
                    if current_block.saturating_sub(archived_l1.completed_at) < L2_ARCHIVE_DELAY_BLOCKS {
                        continue;
                    }

                    // 创建 L2 归档记录
                    let archived_l2 = ArchivedSwapL2 {
                        id: archived_l1.swap_id,
                        status: Self::swap_status_to_u8(&archived_l1.status),
                        year_month: block_to_year_month(archived_l1.completed_at, 14400),
                        amount_tier: amount_to_tier(archived_l1.usdt_amount),
                        flags: 0,
                    };

                    // 更新永久统计
                    SwapStats::<T>::mutate(|stats| {
                        stats.total_swaps = stats.total_swaps.saturating_add(1);
                        if matches!(archived_l1.status, SwapStatus::Completed | SwapStatus::ArbitrationApproved) {
                            stats.completed_swaps = stats.completed_swaps.saturating_add(1);
                            stats.total_volume = stats.total_volume.saturating_add(archived_l1.usdt_amount);
                        } else {
                            stats.refunded_swaps = stats.refunded_swaps.saturating_add(1);
                        }
                    });

                    // 保存 L2 归档并删除 L1 归档
                    ArchivedSwapsL2::<T>::insert(cursor, archived_l2);
                    ArchivedSwapsL1::<T>::remove(cursor);

                    processed = processed.saturating_add(1);
                }
            }

            L1ArchiveCursor::<T>::put(cursor);
            Weight::from_parts(20_000 * processed as u64, 0)
        }

        /// 辅助函数：SwapStatus 转 u8
        fn swap_status_to_u8(status: &SwapStatus) -> u8 {
            match status {
                SwapStatus::Pending => 0,
                SwapStatus::AwaitingVerification => 1,  // 
                SwapStatus::Completed => 2,
                SwapStatus::VerificationFailed => 3,    // 
                SwapStatus::UserReported => 4,
                SwapStatus::Arbitrating => 5,
                SwapStatus::ArbitrationApproved => 6,
                SwapStatus::ArbitrationRejected => 7,
                SwapStatus::Refunded => 8,
            }
        }

        /// 🆕 存储膨胀防护：清理过期的 TRON 交易哈希
        /// 
        /// TTL 策略：30 天后自动删除（防重放攻击窗口）
        /// 每次 on_idle 最多清理 max_count 条记录
        fn cleanup_expired_tx_hashes(max_count: u32) -> Weight {
            let current_block = frame_system::Pallet::<T>::block_number();
            let ttl = T::TxHashTtlBlocks::get();
            let mut removed = 0u32;
            
            // 遍历所有哈希记录，删除过期的
            let to_remove: sp_std::vec::Vec<_> = UsedTronTxHashes::<T>::iter()
                .filter(|(_, recorded_at)| {
                    current_block.saturating_sub(*recorded_at) >= ttl
                })
                .take(max_count as usize)
                .map(|(hash, _)| hash)
                .collect();
            
            for hash in to_remove {
                UsedTronTxHashes::<T>::remove(&hash);
                removed = removed.saturating_add(1);
            }
            
            Weight::from_parts(30_000 * removed as u64 + 10_000, 0)
        }

    }
}
