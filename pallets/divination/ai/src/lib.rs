//! # 通用玄学 AI 解读 Pallet
//!
//! 本模块实现了基于链下预言机的 AI 智能解读系统，支持多种玄学系统：
//! - 梅花易数卦象解读
//! - 八字命盘解读
//! - 六爻占卜解读（预留）
//! - 奇门遁甲解读（预留）
//!
//! ## 核心功能
//!
//! 1. **解读请求**: 用户为占卜结果请求 AI 解读
//! 2. **预言机管理**: 注册、质押、评分管理
//! 3. **结果处理**: 提交解读、评分、争议
//! 4. **费用分配**: 预言机、国库、燃烧分成
//!
//! ## 架构说明
//!
//! 本模块通过 `DivinationProvider` trait 与各玄学核心模块解耦：
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                   pallet-divination-ai                  │
//! │    (通用 AI 解读、预言机管理、争议处理)                   │
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
    use pallet_divination_common::{
        DivinationProvider, DivinationType, InterpretationStatus, InterpretationType,
    };
    use sp_runtime::traits::{SaturatedConversion, Saturating};
    use sp_std::prelude::*;

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type AiCurrency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 占卜结果查询接口
        type DivinationProvider: DivinationProvider<Self::AccountId>;

        /// IPFS 内容注册接口（用于自动 Pin AI 解读内容）
        type ContentRegistry: pallet_storage_service::ContentRegistry;

        /// 基础解读费用
        #[pallet::constant]
        type BaseInterpretationFee: Get<BalanceOf<Self>>;

        /// 预言机最低质押
        #[pallet::constant]
        type MinOracleStake: Get<BalanceOf<Self>>;

        /// 争议押金兜底值（DUST数量，pricing不可用时使用）
        #[pallet::constant]
        type DisputeDeposit: Get<BalanceOf<Self>>;

        /// 争议押金USD价值（精度10^6，1_000_000 = 1 USDT）
        #[pallet::constant]
        type DisputeDepositUsd: Get<u64>;

        /// 保证金计算器（统一的 USD 价值动态计算）
        type DepositCalculator: pallet_trading_common::DepositCalculator<BalanceOf<Self>>;

        /// 请求超时（区块数）
        #[pallet::constant]
        type RequestTimeout: Get<BlockNumberFor<Self>>;

        /// 处理超时（区块数）
        #[pallet::constant]
        type ProcessingTimeout: Get<BlockNumberFor<Self>>;

        /// 争议期限（区块数）
        #[pallet::constant]
        type DisputePeriod: Get<BlockNumberFor<Self>>;

        /// 最大 IPFS CID 长度
        #[pallet::constant]
        type MaxCidLength: Get<u32>;

        /// 最大预言机数量
        #[pallet::constant]
        type MaxOracles: Get<u32>;

        /// 国库账户
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// 仲裁员权限来源
        type ArbitratorOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 治理权限来源（用于参数调整）
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::AiCurrency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    /// 请求类型别名
    pub type InterpretationRequestOf<T> = InterpretationRequest<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 结果类型别名
    pub type InterpretationResultOf<T> = InterpretationResult<
        <T as frame_system::Config>::AccountId,
        BlockNumberFor<T>,
        <T as Config>::MaxCidLength,
    >;

    /// 预言机类型别名
    pub type OracleNodeOf<T> = OracleNode<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    /// 争议类型别名
    pub type DisputeOf<T> = InterpretationDispute<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        BlockNumberFor<T>,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个请求 ID
    #[pallet::storage]
    #[pallet::getter(fn next_request_id)]
    pub type NextRequestId<T> = StorageValue<_, u64, ValueQuery>;

    /// 下一个争议 ID
    #[pallet::storage]
    #[pallet::getter(fn next_dispute_id)]
    pub type NextDisputeId<T> = StorageValue<_, u64, ValueQuery>;

    /// 解读请求存储
    #[pallet::storage]
    #[pallet::getter(fn requests)]
    pub type Requests<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, InterpretationRequestOf<T>>;

    /// 解读结果存储
    #[pallet::storage]
    #[pallet::getter(fn results)]
    pub type Results<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, InterpretationResultOf<T>>;

    /// 预言机节点存储
    #[pallet::storage]
    #[pallet::getter(fn oracles)]
    pub type Oracles<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, OracleNodeOf<T>>;

    /// 活跃预言机列表
    #[pallet::storage]
    #[pallet::getter(fn active_oracles)]
    pub type ActiveOracles<T: Config> =
        StorageValue<_, BoundedVec<T::AccountId, T::MaxOracles>, ValueQuery>;

    /// 争议存储
    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> = StorageMap<_, Blake2_128Concat, u64, DisputeOf<T>>;

    /// 用户请求索引
    #[pallet::storage]
    #[pallet::getter(fn user_requests)]
    pub type UserRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<200>>,
        ValueQuery,
    >;

    /// 预言机处理队列
    #[pallet::storage]
    #[pallet::getter(fn oracle_queue)]
    pub type OracleQueue<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, ConstU32<100>>,
        ValueQuery,
    >;

    /// 费用分配配置
    #[pallet::storage]
    #[pallet::getter(fn fee_distribution)]
    pub type FeeDistributionConfig<T> = StorageValue<_, FeeDistribution, ValueQuery>;

    /// 全局统计信息
    #[pallet::storage]
    #[pallet::getter(fn stats)]
    pub type Stats<T> = StorageValue<_, InterpretationStats, ValueQuery>;

    /// 按占卜类型的统计
    #[pallet::storage]
    #[pallet::getter(fn type_stats)]
    pub type TypeStats<T: Config> =
        StorageMap<_, Blake2_128Concat, DivinationType, TypeInterpretationStats, ValueQuery>;

    /// 占卜类型的 AI 模型配置
    ///
    /// 存储每种占卜类型的模型要求和费用配置
    #[pallet::storage]
    #[pallet::getter(fn model_configs)]
    pub type ModelConfigs<T: Config> =
        StorageMap<_, Blake2_128Concat, DivinationType, ModelConfig, OptionQuery>;

    /// Oracle 节点的模型支持信息
    ///
    /// 存储每个 Oracle 节点声明支持的模型详情
    #[pallet::storage]
    #[pallet::getter(fn oracle_model_support)]
    pub type OracleModelSupports<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, OracleModelSupport, ValueQuery>;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 解读请求已创建
        InterpretationRequested {
            request_id: u64,
            divination_type: DivinationType,
            result_id: u64,
            requester: T::AccountId,
            interpretation_type: InterpretationType,
            fee: BalanceOf<T>,
        },

        /// 预言机已接收请求
        RequestAccepted {
            request_id: u64,
            oracle: T::AccountId,
        },

        /// 解读结果已提交
        ResultSubmitted {
            request_id: u64,
            oracle: T::AccountId,
            content_cid: BoundedVec<u8, T::MaxCidLength>,
        },

        /// 请求处理失败
        RequestFailed {
            request_id: u64,
            oracle: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        },

        /// 用户已评分
        ResultRated {
            request_id: u64,
            user: T::AccountId,
            rating: u8,
        },

        /// 预言机已注册
        OracleRegistered {
            oracle: T::AccountId,
            stake: BalanceOf<T>,
        },

        /// 预言机已注销
        OracleUnregistered { oracle: T::AccountId },

        /// 预言机已暂停
        OraclePaused { oracle: T::AccountId },

        /// 预言机已恢复
        OracleResumed { oracle: T::AccountId },

        /// 争议已创建
        DisputeCreated {
            dispute_id: u64,
            request_id: u64,
            disputer: T::AccountId,
        },

        /// 争议已解决
        DisputeResolved {
            dispute_id: u64,
            resolution: DisputeResolution,
        },

        /// 请求已超时
        RequestExpired { request_id: u64 },

        /// 请求已取消
        RequestCancelled {
            request_id: u64,
            requester: T::AccountId,
            refunded: BalanceOf<T>,
        },

        /// 费用已分配
        FeesDistributed {
            request_id: u64,
            oracle_amount: BalanceOf<T>,
            treasury_amount: BalanceOf<T>,
        },

        /// 模型配置已更新
        ModelConfigUpdated {
            divination_type: DivinationType,
            fee_multiplier: u32,
            enabled: bool,
        },

        /// Oracle 模型支持已更新
        OracleModelSupportUpdated {
            oracle: T::AccountId,
            divination_type: DivinationType,
            model_version: u32,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 占卜结果不存在
        DivinationResultNotFound,
        /// 请求不存在
        RequestNotFound,
        /// 结果不存在
        ResultNotFound,
        /// 预言机不存在
        OracleNotFound,
        /// 争议不存在
        DisputeNotFound,
        /// 非请求所有者
        NotRequestOwner,
        /// 非预言机所有者
        NotOracleOwner,
        /// 预言机已注册
        OracleAlreadyRegistered,
        /// 预言机未注册
        OracleNotRegistered,
        /// 质押不足
        InsufficientStake,
        /// 费用不足
        InsufficientFee,
        /// 请求状态无效
        InvalidRequestStatus,
        /// 评分无效（应为 1-5）
        InvalidRating,
        /// 请求已超时
        RequestExpired,
        /// 争议期已过
        DisputePeriodExpired,
        /// 争议已存在
        DisputeAlreadyExists,
        /// 预言机不活跃
        OracleNotActive,
        /// 预言机不支持该占卜类型
        OracleDivinationTypeNotSupported,
        /// 预言机不支持该解读类型
        OracleInterpretationTypeNotSupported,
        /// 无可用预言机
        NoAvailableOracle,
        /// CID 太长
        CidTooLong,
        /// 名称太长
        NameTooLong,
        /// 请求列表已满
        RequestListFull,
        /// 预言机列表已满
        OracleListFull,
        /// 已评分
        AlreadyRated,
        /// 结果已提交
        ResultAlreadySubmitted,
        /// 非仲裁员
        NotArbitrator,
        /// 争议状态无效
        InvalidDisputeStatus,
        /// 解读类型不适用于该占卜类型
        InterpretationTypeNotApplicable,
        /// 占卜类型 AI 解读未启用
        DivinationTypeNotEnabled,
        /// 模型版本不满足要求
        ModelVersionTooLow,
        /// Oracle 模型列表已满
        OracleModelListFull,
        /// 无效的模型配置
        InvalidModelConfig,
        /// 请求仍在处理中，无法取消
        RequestStillProcessing,
        /// 请求已完成，无法取消
        RequestAlreadyCompleted,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 请求 AI 解读
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `divination_type`: 占卜类型（梅花、八字等）
        /// - `result_id`: 占卜结果 ID（卦象 ID、命盘 ID 等）
        /// - `interpretation_type`: 解读类型
        /// - `context_hash`: 额外上下文哈希（可选）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn request_interpretation(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            interpretation_type: InterpretationType,
            context_hash: Option<[u8; 32]>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证占卜结果存在
            ensure!(
                T::DivinationProvider::result_exists(divination_type, result_id),
                Error::<T>::DivinationResultNotFound
            );

            // 验证解读类型是否适用于该占卜类型
            ensure!(
                interpretation_type.is_applicable_to(divination_type),
                Error::<T>::InterpretationTypeNotApplicable
            );

            // 获取模型配置（如果存在则使用，否则使用默认值）
            let model_config = ModelConfigs::<T>::get(divination_type)
                .unwrap_or_else(|| ModelConfig::new_default(divination_type));

            // 检查该占卜类型是否启用
            ensure!(model_config.enabled, Error::<T>::DivinationTypeNotEnabled);

            // 计算费用：基础费用 × 解读类型倍数 × 占卜类型倍数
            let base_fee = T::BaseInterpretationFee::get();
            let interpretation_multiplier = interpretation_type.fee_multiplier();
            let divination_multiplier = model_config.fee_multiplier;
            let fee = base_fee
                .saturating_mul(interpretation_multiplier.into())
                .saturating_mul(divination_multiplier.into())
                / 10000u32.into(); // 两个百分比相乘需要除以 10000

            // 扣除费用（暂存）
            T::AiCurrency::reserve(&who, fee)?;

            // 创建请求
            let request_id = NextRequestId::<T>::get();
            NextRequestId::<T>::put(request_id.saturating_add(1));

            let block_number = <frame_system::Pallet<T>>::block_number();

            let request = InterpretationRequest {
                id: request_id,
                divination_type,
                result_id,
                requester: who.clone(),
                interpretation_type,
                status: InterpretationStatus::Pending,
                fee_paid: fee,
                created_at: block_number,
                processing_started_at: None,
                completed_at: None,
                oracle_node: None,
                context_hash,
            };

            // 存储请求
            Requests::<T>::insert(request_id, request);

            // 更新用户请求索引
            UserRequests::<T>::try_mutate(&who, |list| {
                list.try_push(request_id)
                    .map_err(|_| Error::<T>::RequestListFull)
            })?;

            // 更新统计
            Stats::<T>::mutate(|s| s.total_requests += 1);
            TypeStats::<T>::mutate(divination_type, |s| s.request_count += 1);

            Self::deposit_event(Event::InterpretationRequested {
                request_id,
                divination_type,
                result_id,
                requester: who,
                interpretation_type,
                fee,
            });

            Ok(())
        }

        /// 预言机接收请求
        ///
        /// 在接收前会检查：
        /// 1. Oracle 是否活跃
        /// 2. Oracle 是否支持该占卜类型
        /// 3. Oracle 的模型版本是否满足要求
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn accept_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证预言机
            let oracle = Oracles::<T>::get(&who).ok_or(Error::<T>::OracleNotFound)?;
            ensure!(oracle.is_active, Error::<T>::OracleNotActive);

            // 验证请求
            Requests::<T>::try_mutate(request_id, |maybe_request| {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;
                ensure!(
                    request.status == InterpretationStatus::Pending,
                    Error::<T>::InvalidRequestStatus
                );

                // 检查超时
                let current_block = <frame_system::Pallet<T>>::block_number();
                let timeout = T::RequestTimeout::get();
                ensure!(
                    current_block <= request.created_at + timeout,
                    Error::<T>::RequestExpired
                );

                // 检查预言机是否支持该占卜类型（旧的位图检查）
                ensure!(
                    oracle.supports_divination_type(request.divination_type),
                    Error::<T>::OracleDivinationTypeNotSupported
                );

                // 检查预言机是否支持该解读类型
                ensure!(
                    oracle.supports_interpretation_type(request.interpretation_type),
                    Error::<T>::OracleInterpretationTypeNotSupported
                );

                // 新增：检查模型版本要求
                let model_config = ModelConfigs::<T>::get(request.divination_type);
                if let Some(config) = model_config {
                    // 如果配置了最低版本要求，检查 Oracle 的模型版本
                    if config.min_model_version > 1 {
                        let oracle_models = OracleModelSupports::<T>::get(&who);
                        ensure!(
                            oracle_models.meets_version_requirement(
                                request.divination_type,
                                config.min_model_version
                            ),
                            Error::<T>::ModelVersionTooLow
                        );
                    }

                    // 检查 Oracle 评分要求
                    if config.min_oracle_rating > 0 {
                        ensure!(
                            oracle.average_rating >= config.min_oracle_rating,
                            Error::<T>::OracleNotActive // 可添加专门的错误类型
                        );
                    }
                }

                // 更新请求状态
                request.status = InterpretationStatus::Processing;
                request.processing_started_at = Some(current_block);
                request.oracle_node = Some(who.clone());

                Ok::<_, DispatchError>(())
            })?;

            // 添加到预言机队列
            OracleQueue::<T>::try_mutate(&who, |queue| {
                queue
                    .try_push(request_id)
                    .map_err(|_| Error::<T>::RequestListFull)
            })?;

            // 更新预言机最后活动时间
            Oracles::<T>::mutate(&who, |maybe_oracle| {
                if let Some(oracle) = maybe_oracle {
                    oracle.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            Self::deposit_event(Event::RequestAccepted {
                request_id,
                oracle: who,
            });

            Ok(())
        }

        /// 提交解读结果
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn submit_result(
            origin: OriginFor<T>,
            request_id: u64,
            content_cid: Vec<u8>,
            summary_cid: Option<Vec<u8>>,
            model_version: Vec<u8>,
            language: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证 CID 长度
            ensure!(
                content_cid.len() <= T::MaxCidLength::get() as usize,
                Error::<T>::CidTooLong
            );

            let content_cid_bounded: BoundedVec<u8, T::MaxCidLength> =
                BoundedVec::try_from(content_cid.clone()).map_err(|_| Error::<T>::CidTooLong)?;

            let summary_cid_bounded: Option<BoundedVec<u8, T::MaxCidLength>> = summary_cid
                .map(|cid| BoundedVec::try_from(cid).map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            let model_version_bounded: BoundedVec<u8, ConstU32<32>> =
                BoundedVec::try_from(model_version).map_err(|_| Error::<T>::NameTooLong)?;

            let language_bounded: BoundedVec<u8, ConstU32<8>> =
                BoundedVec::try_from(language).map_err(|_| Error::<T>::NameTooLong)?;

            // 验证请求并更新
            let (fee_paid, divination_type) =
                Requests::<T>::try_mutate(request_id, |maybe_request| {
                    let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;
                    ensure!(
                        request.status == InterpretationStatus::Processing,
                        Error::<T>::InvalidRequestStatus
                    );
                    ensure!(
                        request.oracle_node.as_ref() == Some(&who),
                        Error::<T>::NotOracleOwner
                    );

                    // 检查处理超时
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    let timeout = T::ProcessingTimeout::get();
                    if let Some(started_at) = request.processing_started_at {
                        ensure!(
                            current_block <= started_at + timeout,
                            Error::<T>::RequestExpired
                        );
                    }

                    // 更新状态
                    request.status = InterpretationStatus::Completed;
                    request.completed_at = Some(current_block);

                    Ok::<_, DispatchError>((request.fee_paid, request.divination_type))
                })?;

            // 确保结果未提交
            ensure!(
                !Results::<T>::contains_key(request_id),
                Error::<T>::ResultAlreadySubmitted
            );

            // 🆕 自动 Pin AI 解读内容到 IPFS（在创建 result 之前）
            // 使用 Standard 层级（3副本，24小时巡检）
            <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                b"divination-ai".to_vec(),
                request_id,
                content_cid.clone(),
                pallet_storage_service::PinTier::Standard,
            )?;

            // 如果有摘要，也 Pin 摘要（Temporary 层级）
            if let Some(ref summary) = summary_cid_bounded {
                let _ = <T::ContentRegistry as pallet_storage_service::ContentRegistry>::register_content(
                    b"divination-ai".to_vec(),
                    request_id,
                    summary.to_vec(),
                    pallet_storage_service::PinTier::Temporary,
                );
                // 注意：摘要 Pin 失败不影响主流程，使用 let _ 忽略错误
            }

            // 创建结果
            let result = InterpretationResult {
                request_id,
                content_cid: content_cid_bounded.clone(),
                summary_cid: summary_cid_bounded,
                oracle: who.clone(),
                submitted_at: <frame_system::Pallet<T>>::block_number(),
                quality_score: None,
                user_rating: None,
                model_version: model_version_bounded,
                language: language_bounded,
            };

            Results::<T>::insert(request_id, result);

            // 更新预言机统计
            Oracles::<T>::mutate(&who, |maybe_oracle| {
                if let Some(oracle) = maybe_oracle {
                    oracle.requests_processed += 1;
                    oracle.requests_succeeded += 1;
                    oracle.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            // 从队列移除
            OracleQueue::<T>::mutate(&who, |queue| {
                queue.retain(|&id| id != request_id);
            });

            // 分配费用
            Self::distribute_fees(request_id, &who, fee_paid)?;

            // 更新统计
            Stats::<T>::mutate(|s| s.completed_requests += 1);
            TypeStats::<T>::mutate(divination_type, |s| s.completed_count += 1);

            Self::deposit_event(Event::ResultSubmitted {
                request_id,
                oracle: who,
                content_cid: content_cid_bounded,
            });

            Ok(())
        }

        /// 报告处理失败
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn report_failure(
            origin: OriginFor<T>,
            request_id: u64,
            reason: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let reason_bounded: BoundedVec<u8, ConstU32<128>> =
                BoundedVec::try_from(reason).map_err(|_| Error::<T>::NameTooLong)?;

            // 验证并更新请求
            let (requester, divination_type) =
                Requests::<T>::try_mutate(request_id, |maybe_request| {
                    let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;
                    ensure!(
                        request.status == InterpretationStatus::Processing,
                        Error::<T>::InvalidRequestStatus
                    );
                    ensure!(
                        request.oracle_node.as_ref() == Some(&who),
                        Error::<T>::NotOracleOwner
                    );

                    request.status = InterpretationStatus::Failed;

                    Ok::<_, DispatchError>((request.requester.clone(), request.divination_type))
                })?;

            // 退还费用给用户
            if let Some(request) = Requests::<T>::get(request_id) {
                T::AiCurrency::unreserve(&requester, request.fee_paid);
            }

            // 更新预言机统计（失败不增加成功数）
            Oracles::<T>::mutate(&who, |maybe_oracle| {
                if let Some(oracle) = maybe_oracle {
                    oracle.requests_processed += 1;
                    oracle.last_active_at = <frame_system::Pallet<T>>::block_number();
                }
            });

            // 从队列移除
            OracleQueue::<T>::mutate(&who, |queue| {
                queue.retain(|&id| id != request_id);
            });

            // 更新统计
            Stats::<T>::mutate(|s| s.failed_requests += 1);
            TypeStats::<T>::mutate(divination_type, |s| s.failed_count += 1);

            Self::deposit_event(Event::RequestFailed {
                request_id,
                oracle: who,
                reason: reason_bounded,
            });

            Ok(())
        }

        /// 用户评分
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn rate_result(origin: OriginFor<T>, request_id: u64, rating: u8) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证评分范围
            ensure!(rating >= 1 && rating <= 5, Error::<T>::InvalidRating);

            // 验证请求所有权
            let request = Requests::<T>::get(request_id).ok_or(Error::<T>::RequestNotFound)?;
            ensure!(request.requester == who, Error::<T>::NotRequestOwner);
            ensure!(
                request.status == InterpretationStatus::Completed,
                Error::<T>::InvalidRequestStatus
            );

            // 更新结果评分
            Results::<T>::try_mutate(request_id, |maybe_result| {
                let result = maybe_result.as_mut().ok_or(Error::<T>::ResultNotFound)?;
                ensure!(result.user_rating.is_none(), Error::<T>::AlreadyRated);
                result.user_rating = Some(rating);

                // 更新预言机平均评分
                if let Some(ref oracle) = request.oracle_node {
                    Oracles::<T>::mutate(oracle, |maybe_oracle| {
                        if let Some(o) = maybe_oracle {
                            // 简单移动平均
                            if o.requests_succeeded > 0 {
                                let old_total =
                                    o.average_rating as u64 * (o.requests_succeeded - 1);
                                let new_avg =
                                    (old_total + (rating as u64 * 100)) / o.requests_succeeded;
                                o.average_rating = new_avg as u16;
                            }
                        }
                    });
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ResultRated {
                request_id,
                user: who,
                rating,
            });

            Ok(())
        }

        /// 注册预言机节点
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn register_oracle(
            origin: OriginFor<T>,
            name: Vec<u8>,
            supported_divination_types: u8,
            supported_interpretation_types: u16,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 确保未注册
            ensure!(
                !Oracles::<T>::contains_key(&who),
                Error::<T>::OracleAlreadyRegistered
            );

            let name_bounded: BoundedVec<u8, ConstU32<64>> =
                BoundedVec::try_from(name).map_err(|_| Error::<T>::NameTooLong)?;

            // 质押
            let stake = T::MinOracleStake::get();
            T::AiCurrency::reserve(&who, stake)?;

            let block_number = <frame_system::Pallet<T>>::block_number();

            let oracle = OracleNode {
                account: who.clone(),
                name: name_bounded,
                stake,
                is_active: true,
                registered_at: block_number,
                requests_processed: 0,
                requests_succeeded: 0,
                average_rating: 0,
                last_active_at: block_number,
                supported_divination_types,
                supported_interpretation_types,
            };

            Oracles::<T>::insert(&who, oracle);

            // 添加到活跃列表
            ActiveOracles::<T>::try_mutate(|list| {
                list.try_push(who.clone())
                    .map_err(|_| Error::<T>::OracleListFull)
            })?;

            Self::deposit_event(Event::OracleRegistered {
                oracle: who,
                stake,
            });

            Ok(())
        }

        /// 注销预言机节点
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn unregister_oracle(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let oracle = Oracles::<T>::get(&who).ok_or(Error::<T>::OracleNotFound)?;

            // 确保队列为空
            let queue = OracleQueue::<T>::get(&who);
            ensure!(queue.is_empty(), Error::<T>::InvalidRequestStatus);

            // 退还质押
            T::AiCurrency::unreserve(&who, oracle.stake);

            // 移除预言机
            Oracles::<T>::remove(&who);

            // 从活跃列表移除
            ActiveOracles::<T>::mutate(|list| {
                list.retain(|a| a != &who);
            });

            Self::deposit_event(Event::OracleUnregistered { oracle: who });

            Ok(())
        }

        /// 暂停预言机
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn pause_oracle(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Oracles::<T>::try_mutate(&who, |maybe_oracle| {
                let oracle = maybe_oracle.as_mut().ok_or(Error::<T>::OracleNotFound)?;
                oracle.is_active = false;
                Ok::<_, DispatchError>(())
            })?;

            // 从活跃列表移除
            ActiveOracles::<T>::mutate(|list| {
                list.retain(|a| a != &who);
            });

            Self::deposit_event(Event::OraclePaused { oracle: who });

            Ok(())
        }

        /// 恢复预言机
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn resume_oracle(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Oracles::<T>::try_mutate(&who, |maybe_oracle| {
                let oracle = maybe_oracle.as_mut().ok_or(Error::<T>::OracleNotFound)?;
                oracle.is_active = true;
                oracle.last_active_at = <frame_system::Pallet<T>>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            // 添加到活跃列表
            ActiveOracles::<T>::try_mutate(|list| {
                if !list.contains(&who) {
                    list.try_push(who.clone())
                        .map_err(|_| Error::<T>::OracleListFull)
                } else {
                    Ok(())
                }
            })?;

            Self::deposit_event(Event::OracleResumed { oracle: who });

            Ok(())
        }

        /// 提出争议
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn create_dispute(
            origin: OriginFor<T>,
            request_id: u64,
            reason_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证请求
            let request = Requests::<T>::get(request_id).ok_or(Error::<T>::RequestNotFound)?;
            ensure!(request.requester == who, Error::<T>::NotRequestOwner);
            ensure!(
                request.status == InterpretationStatus::Completed,
                Error::<T>::InvalidRequestStatus
            );

            // 检查争议期
            let current_block = <frame_system::Pallet<T>>::block_number();
            let dispute_period = T::DisputePeriod::get();
            if let Some(completed_at) = request.completed_at {
                ensure!(
                    current_block <= completed_at + dispute_period,
                    Error::<T>::DisputePeriodExpired
                );
            }

            // 收取争议押金（1 USDT 等值的 DUST）
            let deposit = Self::calculate_dispute_deposit();
            T::AiCurrency::reserve(&who, deposit)?;

            // 创建争议
            let dispute_id = NextDisputeId::<T>::get();
            NextDisputeId::<T>::put(dispute_id.saturating_add(1));

            let dispute = InterpretationDispute {
                id: dispute_id,
                request_id,
                disputer: who.clone(),
                reason_hash,
                deposit,
                created_at: current_block,
                status: DisputeStatus::Pending,
                resolution: None,
            };

            Disputes::<T>::insert(dispute_id, dispute);

            // 更新请求状态
            Requests::<T>::mutate(request_id, |maybe_request| {
                if let Some(request) = maybe_request {
                    request.status = InterpretationStatus::Disputed;
                }
            });

            // 更新统计
            Stats::<T>::mutate(|s| s.total_disputes += 1);

            Self::deposit_event(Event::DisputeCreated {
                dispute_id,
                request_id,
                disputer: who,
            });

            Ok(())
        }

        /// 解决争议（仅限仲裁员）
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn resolve_dispute(
            origin: OriginFor<T>,
            dispute_id: u64,
            resolution: DisputeResolution,
        ) -> DispatchResult {
            T::ArbitratorOrigin::ensure_origin(origin)?;

            let dispute = Disputes::<T>::get(dispute_id).ok_or(Error::<T>::DisputeNotFound)?;
            ensure!(
                dispute.status == DisputeStatus::Pending
                    || dispute.status == DisputeStatus::UnderReview,
                Error::<T>::InvalidDisputeStatus
            );

            let request =
                Requests::<T>::get(dispute.request_id).ok_or(Error::<T>::RequestNotFound)?;

            // 处理争议结果
            match resolution {
                DisputeResolution::UserWins => {
                    // 退还争议押金
                    T::AiCurrency::unreserve(&dispute.disputer, dispute.deposit);
                    // 退还解读费用
                    T::AiCurrency::unreserve(&request.requester, request.fee_paid);

                    Stats::<T>::mutate(|s| s.disputes_user_wins += 1);
                }
                DisputeResolution::OracleWins => {
                    // 没收争议押金到国库
                    T::AiCurrency::unreserve(&dispute.disputer, dispute.deposit);
                    let _ = T::AiCurrency::transfer(
                        &dispute.disputer,
                        &T::TreasuryAccount::get(),
                        dispute.deposit,
                        ExistenceRequirement::KeepAlive,
                    );
                }
                DisputeResolution::PartialRefund => {
                    // 退还争议押金
                    T::AiCurrency::unreserve(&dispute.disputer, dispute.deposit);
                    // 退还部分解读费用（50%）
                    let refund = request.fee_paid / 2u32.into();
                    T::AiCurrency::unreserve(&request.requester, refund);
                }
                DisputeResolution::Reinterpret => {
                    // 退还争议押金
                    T::AiCurrency::unreserve(&dispute.disputer, dispute.deposit);
                    // 重置请求状态
                    Requests::<T>::mutate(dispute.request_id, |maybe_request| {
                        if let Some(request) = maybe_request {
                            request.status = InterpretationStatus::Pending;
                            request.oracle_node = None;
                            request.processing_started_at = None;
                            request.completed_at = None;
                        }
                    });
                    // 删除原结果
                    Results::<T>::remove(dispute.request_id);
                }
            }

            // 更新争议状态
            Disputes::<T>::mutate(dispute_id, |maybe_dispute| {
                if let Some(d) = maybe_dispute {
                    d.status = DisputeStatus::Resolved;
                    d.resolution = Some(resolution);
                }
            });

            Self::deposit_event(Event::DisputeResolved {
                dispute_id,
                resolution,
            });

            Ok(())
        }

        /// 更新费用分配配置（仅限治理）
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn update_fee_distribution(
            origin: OriginFor<T>,
            distribution: FeeDistribution,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            ensure!(distribution.is_valid(), Error::<T>::InvalidRating);
            FeeDistributionConfig::<T>::put(distribution);
            Ok(())
        }

        /// 设置占卜类型的模型配置（仅限治理）
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `recommended_model_id`: 推荐的模型标识
        /// - `min_model_version`: 最低模型版本要求
        /// - `fee_multiplier`: 费用倍数（100 = 1x）
        /// - `max_response_length`: 最大响应长度
        /// - `enabled`: 是否启用
        /// - `min_oracle_rating`: 最低 Oracle 评分要求
        /// - `timeout_blocks`: 超时区块数（可选）
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn set_model_config(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            recommended_model_id: Vec<u8>,
            min_model_version: u32,
            fee_multiplier: u32,
            max_response_length: u32,
            enabled: bool,
            min_oracle_rating: u16,
            timeout_blocks: Option<u32>,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            // 验证参数
            ensure!(fee_multiplier > 0 && fee_multiplier <= 10000, Error::<T>::InvalidModelConfig);
            ensure!(max_response_length > 0, Error::<T>::InvalidModelConfig);
            ensure!(min_oracle_rating <= 500, Error::<T>::InvalidModelConfig);

            let model_id_bounded: BoundedVec<u8, ConstU32<64>> =
                BoundedVec::try_from(recommended_model_id).map_err(|_| Error::<T>::NameTooLong)?;

            let config = ModelConfig {
                divination_type,
                recommended_model_id: model_id_bounded,
                min_model_version,
                fee_multiplier,
                max_response_length,
                enabled,
                min_oracle_rating,
                timeout_blocks,
            };

            ModelConfigs::<T>::insert(divination_type, config);

            Self::deposit_event(Event::ModelConfigUpdated {
                divination_type,
                fee_multiplier,
                enabled,
            });

            Ok(())
        }

        /// 移除占卜类型的模型配置（仅限治理）
        ///
        /// 移除后将使用默认配置
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(10_000_000, 0))]
        pub fn remove_model_config(
            origin: OriginFor<T>,
            divination_type: DivinationType,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;
            ModelConfigs::<T>::remove(divination_type);
            Ok(())
        }

        /// Oracle 更新自己的模型支持信息
        ///
        /// # 参数
        /// - `divination_type`: 占卜类型
        /// - `model_id`: 模型标识
        /// - `model_version`: 模型版本
        /// - `is_active`: 是否启用
        #[pallet::call_index(14)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_oracle_model_support(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            model_id: Vec<u8>,
            model_version: u32,
            is_active: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是已注册的 Oracle
            ensure!(Oracles::<T>::contains_key(&who), Error::<T>::OracleNotFound);

            let model_id_bounded: BoundedVec<u8, ConstU32<64>> =
                BoundedVec::try_from(model_id).map_err(|_| Error::<T>::NameTooLong)?;

            OracleModelSupports::<T>::try_mutate(&who, |support| {
                // 查找是否已存在该类型的配置
                let existing_idx = support.models.iter().position(|m| m.divination_type == divination_type);

                if let Some(idx) = existing_idx {
                    // 更新现有配置
                    support.models[idx].model_id = model_id_bounded.clone();
                    support.models[idx].model_version = model_version;
                    support.models[idx].is_active = is_active;
                } else {
                    // 添加新配置
                    let model_info = OracleModelInfo {
                        divination_type,
                        model_id: model_id_bounded.clone(),
                        model_version,
                        accuracy_score: 0,
                        requests_count: 0,
                        is_active,
                    };
                    support.models.try_push(model_info).map_err(|_| Error::<T>::OracleModelListFull)?;
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::OracleModelSupportUpdated {
                oracle: who,
                divination_type,
                model_version,
            });

            Ok(())
        }

        /// Oracle 批量更新模型支持信息
        ///
        /// # 参数
        /// - `models`: 模型信息列表
        #[pallet::call_index(15)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn batch_update_oracle_models(
            origin: OriginFor<T>,
            models: Vec<(DivinationType, Vec<u8>, u32, bool)>, // (type, model_id, version, active)
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证是已注册的 Oracle
            ensure!(Oracles::<T>::contains_key(&who), Error::<T>::OracleNotFound);
            ensure!(models.len() <= 16, Error::<T>::OracleModelListFull);

            OracleModelSupports::<T>::try_mutate(&who, |support| {
                for (divination_type, model_id, model_version, is_active) in models.iter() {
                    let model_id_bounded: BoundedVec<u8, ConstU32<64>> =
                        BoundedVec::try_from(model_id.clone()).map_err(|_| Error::<T>::NameTooLong)?;

                    let existing_idx = support.models.iter().position(|m| m.divination_type == *divination_type);

                    if let Some(idx) = existing_idx {
                        support.models[idx].model_id = model_id_bounded;
                        support.models[idx].model_version = *model_version;
                        support.models[idx].is_active = *is_active;
                    } else {
                        let model_info = OracleModelInfo {
                            divination_type: *divination_type,
                            model_id: model_id_bounded,
                            model_version: *model_version,
                            accuracy_score: 0,
                            requests_count: 0,
                            is_active: *is_active,
                        };
                        support.models.try_push(model_info).map_err(|_| Error::<T>::OracleModelListFull)?;
                    }
                }

                Ok::<_, DispatchError>(())
            })?;

            Ok(())
        }

        /// 取消待处理的 AI 解读请求
        ///
        /// 用户可以取消尚未被预言机接收的待处理请求，
        /// 或者取消已超时的处理中请求。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是请求所有者）
        /// - `request_id`: 请求 ID
        ///
        /// # 取消条件
        /// - 状态为 Pending（未被接收）：可直接取消，全额退款
        /// - 状态为 Processing 且已超时：可取消，全额退款
        /// - 状态为 Completed/Disputed/Failed：不可取消
        ///
        /// # 退款规则
        /// - 全额退还锁定的费用
        #[pallet::call_index(16)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn cancel_request(origin: OriginFor<T>, request_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取请求并验证所有权
            let request = Requests::<T>::get(request_id).ok_or(Error::<T>::RequestNotFound)?;
            ensure!(request.requester == who, Error::<T>::NotRequestOwner);

            let current_block = <frame_system::Pallet<T>>::block_number();
            let fee_paid = request.fee_paid;
            let divination_type = request.divination_type;

            // 检查请求状态和是否可取消
            match request.status {
                InterpretationStatus::Pending => {
                    // 待处理状态：可直接取消
                }
                InterpretationStatus::Processing => {
                    // 处理中：仅在超时后可取消
                    let timeout = T::ProcessingTimeout::get();
                    if let Some(started_at) = request.processing_started_at {
                        ensure!(
                            current_block > started_at + timeout,
                            Error::<T>::RequestStillProcessing
                        );
                    }

                    // 从预言机队列中移除
                    if let Some(ref oracle) = request.oracle_node {
                        OracleQueue::<T>::mutate(oracle, |queue| {
                            queue.retain(|&id| id != request_id);
                        });
                    }
                }
                InterpretationStatus::Completed => {
                    return Err(Error::<T>::RequestAlreadyCompleted.into());
                }
                InterpretationStatus::Disputed => {
                    return Err(Error::<T>::InvalidRequestStatus.into());
                }
                InterpretationStatus::Failed | InterpretationStatus::Expired => {
                    // 已失败或已过期的请求应该已经退款，但仍可清理
                }
                InterpretationStatus::Refunded => {
                    // 已退款的请求，仅清理记录
                }
            }

            // 退还费用
            T::AiCurrency::unreserve(&who, fee_paid);

            // 从用户请求列表中移除
            UserRequests::<T>::mutate(&who, |list| {
                list.retain(|&id| id != request_id);
            });

            // 删除请求记录
            Requests::<T>::remove(request_id);

            // 更新统计
            Stats::<T>::mutate(|s| {
                s.total_requests = s.total_requests.saturating_sub(1);
            });
            TypeStats::<T>::mutate(divination_type, |s| {
                s.request_count = s.request_count.saturating_sub(1);
            });

            Self::deposit_event(Event::RequestCancelled {
                request_id,
                requester: who,
                refunded: fee_paid,
            });

            Ok(())
        }
    }

    // ==================== 内部辅助函数 ====================

    impl<T: Config> Pallet<T> {
        /// 分配费用
        fn distribute_fees(
            request_id: u64,
            oracle: &T::AccountId,
            fee: BalanceOf<T>,
        ) -> DispatchResult {
            let request = Requests::<T>::get(request_id).ok_or(Error::<T>::RequestNotFound)?;

            // 解除用户的费用锁定
            T::AiCurrency::unreserve(&request.requester, fee);

            let distribution = FeeDistributionConfig::<T>::get();

            // 计算各部分金额
            let oracle_amount =
                fee.saturating_mul(distribution.oracle_share.into()) / 10000u32.into();
            let treasury_amount =
                fee.saturating_mul(distribution.treasury_share.into()) / 10000u32.into();

            // 转给预言机
            T::AiCurrency::transfer(
                &request.requester,
                oracle,
                oracle_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 转给国库
            T::AiCurrency::transfer(
                &request.requester,
                &T::TreasuryAccount::get(),
                treasury_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 更新统计
            Stats::<T>::mutate(|s| {
                s.total_fees_collected = s.total_fees_collected.saturating_add(fee.saturated_into());
            });

            Self::deposit_event(Event::FeesDistributed {
                request_id,
                oracle_amount,
                treasury_amount,
            });

            Ok(())
        }

        /// 计算争议押金金额（1 USDT 等值的 DUST）
        /// 
        /// 使用统一的 DepositCalculator trait 计算
        pub fn calculate_dispute_deposit() -> BalanceOf<T> {
            use pallet_trading_common::DepositCalculator;
            T::DepositCalculator::calculate_deposit(
                T::DisputeDepositUsd::get(),
                T::DisputeDeposit::get(),
            )
        }
    }
}
