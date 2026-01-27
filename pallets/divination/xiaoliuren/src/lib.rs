//! # 小六壬排盘 Pallet
//!
//! 本模块实现了区块链上的小六壬排盘系统，提供：
//! - 时间起课（使用农历月日时起课）
//! - 数字起课（活数起课法）
//! - 随机起课（使用链上随机数）
//! - 手动指定起课
//! - 课盘存储与查询
//! - AI 解读请求（链下工作机触发）
//!
//! ## 小六壬概述
//!
//! 小六壬，又称"诸葛亮马前课"或"掐指速算"，是中国古代流传的一种简易占卜术。
//! 通过六宫（大安、留连、速喜、赤口、小吉、空亡）来预测吉凶。
//!
//! ## 六宫含义
//!
//! - **大安**：属木，临青龙，吉祥安康
//! - **留连**：属水，临玄武，延迟纠缠
//! - **速喜**：属火，临朱雀，快速喜庆
//! - **赤口**：属金，临白虎，口舌是非
//! - **小吉**：属木，临六合，和合吉利
//! - **空亡**：属土，临勾陈，无果忧虑
//!
//! ## 起课方法
//!
//! ### 1. 时间起课（传统方法）
//! 按农历月日时起课：
//! - 月宫：从大安起正月，顺数至所求月份
//! - 日宫：从月宫起初一，顺数至所求日期
//! - 时宫：从日宫起子时，顺数至所求时辰
//!
//! ### 2. 数字起课（活数起课法）
//! 取三个数字 x、y、z：
//! - 月宫 = (x - 1) % 6
//! - 日宫 = (x + y - 2) % 6
//! - 时宫 = (x + y + z - 3) % 6
//!
//! ### 3. 随机起课
//! 使用链上随机数生成三个数字，然后按数字起课法计算。

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod algorithm;
pub mod interpretation;
pub mod ocw_tee;
pub mod runtime_api;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::algorithm;
    use crate::types::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use sp_std::prelude::*;

    // ============================================================================
    // Pallet 配置
    // ============================================================================

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_timestamp::Config {
        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 随机数生成器
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 每个用户最多存储的课盘数量
        #[pallet::constant]
        type MaxUserPans: Get<u32>;

        /// 公开课盘列表的最大长度
        #[pallet::constant]
        type MaxPublicPans: Get<u32>;

        /// 问题 CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 每日免费起课次数
        #[pallet::constant]
        type DailyFreeDivinations: Get<u32>;

        /// 每日最大起课次数（防刷）
        #[pallet::constant]
        type MaxDailyDivinations: Get<u32>;

        /// 加密数据最大长度（默认: 512 bytes）
        #[pallet::constant]
        type MaxEncryptedLen: Get<u32>;

        /// AI 解读费用
        #[pallet::constant]
        type AiInterpretationFee: Get<BalanceOf<Self>>;

        /// 国库账户
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// AI 预言机权限来源
        type AiOracleOrigin: EnsureOrigin<Self::RuntimeOrigin>;

    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ============================================================================
    // 存储项
    // ============================================================================

    /// 下一个课盘 ID
    #[pallet::storage]
    #[pallet::getter(fn next_pan_id)]
    pub type NextPanId<T> = StorageValue<_, u64, ValueQuery>;

    /// 课盘存储
    ///
    /// 键：课盘 ID
    /// 值：完整课盘结构
    #[pallet::storage]
    #[pallet::getter(fn pans)]
    pub type Pans<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        XiaoLiuRenPan<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    >;

    /// 用户课盘索引
    ///
    /// 键：用户账户
    /// 值：该用户的所有课盘 ID 列表
    #[pallet::storage]
    #[pallet::getter(fn user_pans)]
    pub type UserPans<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxUserPans>,
        ValueQuery,
    >;

    /// 公开课盘列表
    ///
    /// 存储所有设置为公开的课盘 ID
    #[pallet::storage]
    #[pallet::getter(fn public_pans)]
    pub type PublicPans<T: Config> =
        StorageValue<_, BoundedVec<u64, T::MaxPublicPans>, ValueQuery>;

    /// 每日起课计数
    ///
    /// 用于限制每日起课次数，防止滥用
    /// 键1：用户账户
    /// 键2：天数（从创世块起算）
    /// 值：当日起课次数
    #[pallet::storage]
    #[pallet::getter(fn daily_divination_count)]
    pub type DailyDivinationCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u32, // day number
        u32, // count
        ValueQuery,
    >;

    /// AI 解读请求队列
    ///
    /// 存储待处理的 AI 解读请求
    #[pallet::storage]
    #[pallet::getter(fn ai_interpretation_requests)]
    pub type AiInterpretationRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

    /// 用户统计数据
    #[pallet::storage]
    #[pallet::getter(fn user_stats)]
    pub type UserStatsStorage<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserStats, ValueQuery>;

    /// 加密数据存储
    #[pallet::storage]
    #[pallet::getter(fn encrypted_data)]
    pub type EncryptedDataStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<u8, T::MaxEncryptedLen>,
    >;

    /// 所有者密钥备份存储
    #[pallet::storage]
    #[pallet::getter(fn owner_key_backup)]
    pub type OwnerKeyBackupStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        [u8; 80],
    >;

    /// 课盘解卦数据
    ///
    /// 采用懒加载：首次查询时计算并缓存
    #[pallet::storage]
    #[pallet::getter(fn interpretations)]
    pub type Interpretations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // pan_id
        crate::interpretation::XiaoLiuRenInterpretation,
    >;

    // ============================================================================
    // 事件
    // ============================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 新课盘创建成功
        /// [课盘ID, 占卜者, 起课方式]
        PanCreated {
            pan_id: u64,
            creator: T::AccountId,
            method: DivinationMethod,
        },

        /// AI 解读请求已提交
        /// [课盘ID, 请求者]
        AiInterpretationRequested {
            pan_id: u64,
            requester: T::AccountId,
        },

        /// AI 解读结果已提交
        /// [课盘ID, IPFS CID]
        AiInterpretationSubmitted {
            pan_id: u64,
            cid: BoundedVec<u8, T::MaxCidLen>,
        },

        /// 课盘公开状态已更改
        /// [课盘ID, 是否公开]
        PanVisibilityChanged {
            pan_id: u64,
            is_public: bool,
        },

        /// 加密课盘创建成功
        EncryptedPanCreated {
            pan_id: u64,
            creator: T::AccountId,
            privacy_mode: pallet_divination_privacy::types::PrivacyMode,
            method: DivinationMethod,
        },

        /// 加密数据已更新
        EncryptedDataUpdated {
            pan_id: u64,
            data_hash: [u8; 32],
        },

        /// 课盘已删除
        /// [课盘ID, 所有者]
        PanDeleted {
            pan_id: u64,
            owner: T::AccountId,
        },
    }

    // ============================================================================
    // 错误
    // ============================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 课盘不存在
        PanNotFound,
        /// 非课盘所有者
        NotOwner,
        /// 每日起课次数超限
        DailyLimitExceeded,
        /// 无效的农历月份（应为 1-12）
        InvalidLunarMonth,
        /// 无效的农历日期（应为 1-30）
        InvalidLunarDay,
        /// 无效的时辰（应为 0-23 小时）
        InvalidHour,
        /// 用户课盘列表已满
        UserPansFull,
        /// 公开课盘列表已满
        PublicPansFull,
        /// AI 解读费用不足
        InsufficientFee,
        /// AI 解读请求已存在
        AiRequestAlreadyExists,
        /// AI 解读请求不存在
        AiRequestNotFound,
        /// 无效的起课参数
        InvalidParams,
        /// 数字起课参数必须大于 0
        NumberMustBePositive,
        /// 无效的加密级别
        InvalidEncryptionLevel,
        /// 加密数据缺失
        EncryptedDataMissing,
        /// 数据哈希缺失
        DataHashMissing,
        /// 密钥备份缺失
        OwnerKeyBackupMissing,
        /// 加密数据过长
        EncryptedDataTooLong,
    }

    // ============================================================================
    // 可调用函数
    // ============================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 时间起课
        ///
        /// 使用农历月日时起课，这是最传统的小六壬起课方法。
        ///
        /// # 参数
        /// - `origin`: 调用者（签名账户）
        /// - `lunar_month`: 农历月份（1-12）
        /// - `lunar_day`: 农历日期（1-30）
        /// - `hour`: 当前小时（0-23，用于计算时辰）
        /// - `question_cid`: 占卜问题的 IPFS CID（可选，隐私保护）
        /// - `is_public`: 是否公开此课盘
        ///
        /// # 算法
        /// 1. 月宫：从大安起正月，顺数至所求月份
        /// 2. 日宫：从月宫起初一，顺数至所求日期
        /// 3. 时宫：从日宫起子时，顺数至所求时辰
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_time(
            origin: OriginFor<T>,
            lunar_month: u8,
            lunar_day: u8,
            hour: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数
            ensure!(lunar_month >= 1 && lunar_month <= 12, Error::<T>::InvalidLunarMonth);
            ensure!(lunar_day >= 1 && lunar_day <= 30, Error::<T>::InvalidLunarDay);
            ensure!(hour <= 23, Error::<T>::InvalidHour);

            // 计算时辰
            let shi_chen = ShiChen::from_hour(hour);

            // 使用时间起课算法
            let san_gong = algorithm::divine_by_time(lunar_month, lunar_day, shi_chen);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::TimeMethod,
                san_gong,
                lunar_month,
                lunar_day,
                hour,
                Some(shi_chen),
                question_cid,
                is_public,
            )
        }

        /// 公历时间起课
        ///
        /// 此方法使用 pallet-almanac 自动将公历日期转换为农历，
        /// 然后进行小六壬起课。用户无需手动计算农历。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn divine_by_solar_time(
            origin: OriginFor<T>,
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 参数校验
            ensure!(solar_year >= 1901 && solar_year <= 2100, Error::<T>::InvalidLunarMonth);
            ensure!(solar_month >= 1 && solar_month <= 12, Error::<T>::InvalidLunarMonth);
            ensure!(solar_day >= 1 && solar_day <= 31, Error::<T>::InvalidLunarDay);
            ensure!(hour <= 23, Error::<T>::InvalidHour);

            // 调用 almanac 转农历
            let lunar = pallet_almanac::solar_to_lunar(solar_year, solar_month, solar_day)
                .ok_or(Error::<T>::InvalidLunarMonth)?;

            // 计算时辰
            let shi_chen = ShiChen::from_hour(hour);

            // 使用时间起课算法
            let san_gong = algorithm::divine_by_time(lunar.month, lunar.day, shi_chen);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::TimeMethod,
                san_gong,
                lunar.month,
                lunar.day,
                hour,
                Some(shi_chen),
                question_cid,
                is_public,
            )
        }

        /// 数字起课（活数起课法）
        ///
        /// 使用三个数字进行起课，适合即兴占卜。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `x`: 第一个数字（≥1）
        /// - `y`: 第二个数字（≥1）
        /// - `z`: 第三个数字（≥1）
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        ///
        /// # 算法
        /// - 月宫 = (x - 1) % 6
        /// - 日宫 = (x + y - 2) % 6
        /// - 时宫 = (x + y + z - 3) % 6
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_number(
            origin: OriginFor<T>,
            x: u8,
            y: u8,
            z: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数必须大于 0
            ensure!(x >= 1 && y >= 1 && z >= 1, Error::<T>::NumberMustBePositive);

            // 使用数字起课算法
            let san_gong = algorithm::divine_by_number(x, y, z);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::NumberMethod,
                san_gong,
                x,
                y,
                z,
                None,
                question_cid,
                is_public,
            )
        }

        /// 随机起课
        ///
        /// 使用链上随机数生成卦象，适合无特定数字时使用。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_random(
            origin: OriginFor<T>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 使用链上随机源
            let random_seed = T::Randomness::random(&b"xiaoliuren"[..]).0;
            let random_bytes: [u8; 32] = random_seed
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 获取起课参数
            let (x, y, z) = algorithm::random_to_params(&random_bytes);

            // 使用随机起课算法
            let san_gong = algorithm::divine_random(&random_bytes);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::RandomMethod,
                san_gong,
                x,
                y,
                z,
                None,
                question_cid,
                is_public,
            )
        }

        /// 手动指定起课
        ///
        /// 直接指定三宫结果，用于已知课盘的记录。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `yue_index`: 月宫索引（0-5）
        /// - `ri_index`: 日宫索引（0-5）
        /// - `shi_index`: 时宫索引（0-5）
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_manual(
            origin: OriginFor<T>,
            yue_index: u8,
            ri_index: u8,
            shi_index: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数（索引 0-5）
            ensure!(yue_index <= 5 && ri_index <= 5 && shi_index <= 5, Error::<T>::InvalidParams);

            // 使用手动指定算法
            let san_gong = algorithm::divine_manual(yue_index, ri_index, shi_index);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::ManualMethod,
                san_gong,
                yue_index,
                ri_index,
                shi_index,
                None,
                question_cid,
                is_public,
            )
        }

        /// 请求 AI 解读（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::request_interpretation`
        /// 新的统一 AI 解读系统支持多种 AI 模型选择、Oracle 质押评分、争议退款。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `pan_id`: 课盘 ID
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::request_interpretation"
        )]
        pub fn request_ai_interpretation(
            origin: OriginFor<T>,
            pan_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证课盘存在且为调用者所有
            let pan = Pans::<T>::get(pan_id)
                .ok_or(Error::<T>::PanNotFound)?;
            ensure!(pan.creator == who, Error::<T>::NotOwner);

            // 检查是否已有请求
            ensure!(
                !AiInterpretationRequests::<T>::contains_key(pan_id),
                Error::<T>::AiRequestAlreadyExists
            );

            // 扣除 AI 解读费用
            T::Currency::transfer(
                &who,
                &T::TreasuryAccount::get(),
                T::AiInterpretationFee::get(),
                ExistenceRequirement::KeepAlive,
            )?;

            // 记录请求
            AiInterpretationRequests::<T>::insert(pan_id, who.clone());

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                stats.ai_interpretations = stats.ai_interpretations.saturating_add(1);
            });

            // 发送事件触发链下工作机
            Self::deposit_event(Event::AiInterpretationRequested {
                pan_id,
                requester: who,
            });

            Ok(())
        }

        /// 提交 AI 解读结果（仅限授权节点）（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::submit_result`
        ///
        /// # 参数
        /// - `origin`: AI 预言机授权来源
        /// - `pan_id`: 课盘 ID
        /// - `interpretation_cid`: 解读内容的 IPFS CID
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::submit_result"
        )]
        pub fn submit_ai_interpretation(
            origin: OriginFor<T>,
            pan_id: u64,
            interpretation_cid: BoundedVec<u8, T::MaxCidLen>,
        ) -> DispatchResult {
            // 验证 AI 预言机权限
            T::AiOracleOrigin::ensure_origin(origin)?;

            // 验证请求存在
            ensure!(
                AiInterpretationRequests::<T>::contains_key(pan_id),
                Error::<T>::AiRequestNotFound
            );

            // 更新课盘的 AI 解读 CID
            Pans::<T>::try_mutate(pan_id, |maybe_pan| {
                let pan = maybe_pan
                    .as_mut()
                    .ok_or(Error::<T>::PanNotFound)?;
                pan.ai_interpretation_cid = Some(interpretation_cid.clone());
                Ok::<_, DispatchError>(())
            })?;

            // 移除请求
            AiInterpretationRequests::<T>::remove(pan_id);

            Self::deposit_event(Event::AiInterpretationSubmitted {
                pan_id,
                cid: interpretation_cid,
            });

            Ok(())
        }

        /// 更改课盘公开状态
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `pan_id`: 课盘 ID
        /// - `is_public`: 是否公开
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn set_pan_visibility(
            origin: OriginFor<T>,
            pan_id: u64,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Pans::<T>::try_mutate(pan_id, |maybe_pan| {
                let pan = maybe_pan
                    .as_mut()
                    .ok_or(Error::<T>::PanNotFound)?;
                ensure!(pan.creator == who, Error::<T>::NotOwner);

                let was_public = pan.is_public();

                // 更新隐私模式
                pan.privacy_mode = if is_public {
                    pallet_divination_privacy::types::PrivacyMode::Public
                } else {
                    pallet_divination_privacy::types::PrivacyMode::Partial
                };

                // 更新公开课盘列表
                if is_public && !was_public {
                    // 添加到公开列表
                    PublicPans::<T>::try_mutate(|list| {
                        list.try_push(pan_id)
                            .map_err(|_| Error::<T>::PublicPansFull)
                    })?;
                } else if !is_public && was_public {
                    // 从公开列表移除
                    PublicPans::<T>::mutate(|list| {
                        list.retain(|&id| id != pan_id);
                    });
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::PanVisibilityChanged {
                pan_id,
                is_public,
            });

            Ok(())
        }

        /// 时刻分起课（道家流派）
        ///
        /// 使用时辰、刻、分进行起课，这是道家小六壬的特色起课方法。
        ///
        /// # 参数
        /// - `origin`: 调用者（签名账户）
        /// - `hour`: 当前小时（0-23）
        /// - `minute`: 当前分钟（0-59）
        /// - `question_cid`: 占卜问题的 IPFS CID（可选）
        /// - `is_public`: 是否公开此课盘
        ///
        /// # 算法
        /// 1. 时辰值：根据小时计算时辰（1-12）
        /// 2. 刻值：每个时辰分为8刻，计算当前刻数（1-8）
        /// 3. 分值：取分钟数除以15的余数（1-15）
        ///
        /// 然后按数字起课法计算：
        /// - 月宫 = (时辰 - 1) % 6
        /// - 日宫 = (时辰 + 刻 - 2) % 6
        /// - 时宫 = (时辰 + 刻 + 分 - 3) % 6
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_hour_ke(
            origin: OriginFor<T>,
            hour: u8,
            minute: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数
            ensure!(hour <= 23, Error::<T>::InvalidHour);
            ensure!(minute <= 59, Error::<T>::InvalidParams);

            // 计算时辰值（1-12）
            let shi_chen = ShiChen::from_hour(hour);
            let shi_chen_value = shi_chen.index();

            // 计算刻数
            // 每个时辰2小时=120分钟，分为8刻，每刻15分钟
            // 刻数 = (分钟 / 15) + 1，但需要考虑时辰边界
            // 对于偶数小时（如2点），属于丑时后半段，刻数从5开始
            // 对于奇数小时（如1点），属于丑时前半段，刻数从1开始
            let ke = if hour % 2 == 0 {
                // 偶数小时：属于时辰后半段，刻数为5-8
                let base_ke = (minute / 15) as u8;
                if base_ke == 0 { 5 } else { base_ke + 4 }
            } else {
                // 奇数小时：属于时辰前半段，刻数为1-4
                let base_ke = (minute / 15) as u8;
                if base_ke == 0 { 1 } else { base_ke }
            };

            // 计算分值（1-15）
            let fen = {
                let remainder = minute % 15;
                if remainder == 0 { 1 } else { remainder }
            };

            // 使用时刻分起课算法
            let san_gong = algorithm::divine_by_hour_ke_fen(shi_chen_value, ke, fen);

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::TimeKeMethod,
                san_gong,
                shi_chen_value,
                ke,
                fen,
                Some(shi_chen),
                question_cid,
                is_public,
            )
        }

        /// 多位数字起课（活数起课法扩展）
        ///
        /// 输入一个多位数字，将各位数字相加求和后进行起课。
        /// 这是活数起课法的便捷版本，适用于看到手机号、车牌号等数字时起课。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `digits`: 多位数字（如 1436 表示看到时间 14:36）
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        ///
        /// # 算法
        /// 1. 将数字拆分为各位：如 1436 → [1, 4, 3, 6]
        /// 2. 计算各位数字之和：1 + 4 + 3 + 6 = 14
        /// 3. 减去 (位数 - 1)：14 - 3 = 11
        /// 4. 对 6 取模得到六神索引：11 % 6 = 5 → 空亡
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_digits(
            origin: OriginFor<T>,
            digits: u32,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数
            ensure!(digits >= 1, Error::<T>::NumberMustBePositive);

            // 使用多位数字起课算法
            let san_gong = algorithm::divine_by_multi_digit(digits);

            // 存储原始数字（拆分为三个字节存储）
            let param1 = ((digits >> 16) & 0xFF) as u8;
            let param2 = ((digits >> 8) & 0xFF) as u8;
            let param3 = (digits & 0xFF) as u8;

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::NumberMethod,
                san_gong,
                param1,
                param2,
                param3,
                None,
                question_cid,
                is_public,
            )
        }

        /// 三数字起课（活数起课法标准版）
        ///
        /// 使用三个任意大小的数字进行起课，数字可以是任意正整数。
        /// 这是活数起课法的完整版本，适用于有明确三个数字时使用。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `num1`: 第一个数字（≥1）
        /// - `num2`: 第二个数字（≥1）
        /// - `num3`: 第三个数字（≥1）
        /// - `question_cid`: 问题 CID（可选）
        /// - `is_public`: 是否公开
        ///
        /// # 算法
        /// 采用递推法：
        /// - 月宫 = num1 对应的六神
        /// - 日宫 = 从月宫起，前进 num2 步
        /// - 时宫 = 从日宫起，前进 num3 步
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_three_numbers(
            origin: OriginFor<T>,
            num1: u32,
            num2: u32,
            num3: u32,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数必须大于 0
            ensure!(num1 >= 1 && num2 >= 1 && num3 >= 1, Error::<T>::NumberMustBePositive);

            // 使用递推法计算
            // 月宫：num1 对应的六神
            let yue_index = ((num1 - 1) % 6) as u8;
            // 日宫：从月宫起，前进 num2-1 步
            let ri_index = ((yue_index as u32 + num2 - 1) % 6) as u8;
            // 时宫：从日宫起，前进 num3-1 步
            let shi_index = ((ri_index as u32 + num3 - 1) % 6) as u8;

            let san_gong = algorithm::divine_manual(yue_index, ri_index, shi_index);

            // 存储参数（取低8位）
            let param1 = (num1 & 0xFF) as u8;
            let param2 = (num2 & 0xFF) as u8;
            let param3 = (num3 & 0xFF) as u8;

            // 创建课盘
            Self::create_pan(
                who,
                DivinationMethod::NumberMethod,
                san_gong,
                param1,
                param2,
                param3,
                None,
                question_cid,
                is_public,
            )
        }

        // ============================================================================
        // 加密模式接口
        // ============================================================================

        /// 时间起课（加密模式）
        ///
        /// 支持三种隐私模式：
        /// - Public (0): 所有数据明文存储
        /// - Partial (1): 计算数据明文，敏感数据加密
        /// - Private (2): 所有数据加密，仅存储元数据
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `encryption_level`: 加密级别（0=Public, 1=Partial, 2=Private）
        /// - `lunar_month`: 农历月份（Public/Partial 模式必填）
        /// - `lunar_day`: 农历日期（Public/Partial 模式必填）
        /// - `hour`: 小时（Public/Partial 模式必填）
        /// - `question_cid`: 问题 CID（可选）
        /// - `encrypted_data`: 加密数据（Partial/Private 模式必填）
        /// - `data_hash`: 数据哈希（Partial/Private 模式必填）
        /// - `owner_key_backup`: 所有者密钥备份（Partial/Private 模式必填）
        #[pallet::call_index(11)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn divine_by_time_encrypted(
            origin: OriginFor<T>,
            encryption_level: u8,
            lunar_month: Option<u8>,
            lunar_day: Option<u8>,
            hour: Option<u8>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
            data_hash: Option<[u8; 32]>,
            owner_key_backup: Option<[u8; 80]>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 解析隐私模式
            let privacy_mode = match encryption_level {
                0 => pallet_divination_privacy::types::PrivacyMode::Public,
                1 => pallet_divination_privacy::types::PrivacyMode::Partial,
                2 => pallet_divination_privacy::types::PrivacyMode::Private,
                _ => return Err(Error::<T>::InvalidEncryptionLevel.into()),
            };

            // 根据隐私模式验证参数和计算三宫
            let (san_gong, param1, param2, param3, shi_chen) = match privacy_mode {
                pallet_divination_privacy::types::PrivacyMode::Public
                | pallet_divination_privacy::types::PrivacyMode::Partial => {
                    // 必须提供明文数据
                    let month = lunar_month.ok_or(Error::<T>::InvalidLunarMonth)?;
                    let day = lunar_day.ok_or(Error::<T>::InvalidLunarDay)?;
                    let h = hour.ok_or(Error::<T>::InvalidHour)?;

                    // 验证参数
                    ensure!(month >= 1 && month <= 12, Error::<T>::InvalidLunarMonth);
                    ensure!(day >= 1 && day <= 30, Error::<T>::InvalidLunarDay);
                    ensure!(h <= 23, Error::<T>::InvalidHour);

                    // 计算时辰和三宫
                    let sc = ShiChen::from_hour(h);
                    let sg = algorithm::divine_by_time(month, day, sc);

                    (Some(sg), Some(month), Some(day), Some(h), Some(sc))
                }
                pallet_divination_privacy::types::PrivacyMode::Private => {
                    // Private 模式不存储计算数据
                    (None, None, None, None, None)
                }
            };

            // Partial/Private 模式需要加密数据
            if privacy_mode != pallet_divination_privacy::types::PrivacyMode::Public {
                ensure!(encrypted_data.is_some(), Error::<T>::EncryptedDataMissing);
                ensure!(data_hash.is_some(), Error::<T>::DataHashMissing);
                ensure!(owner_key_backup.is_some(), Error::<T>::OwnerKeyBackupMissing);
            }

            // 创建课盘
            let pan_id = NextPanId::<T>::get();

            Self::create_pan_with_privacy(
                who.clone(),
                DivinationMethod::TimeMethod,
                san_gong,
                param1,
                param2,
                param3,
                shi_chen,
                question_cid,
                privacy_mode,
                if privacy_mode != pallet_divination_privacy::types::PrivacyMode::Public {
                    Some(0x01) // bit 0: question encrypted
                } else {
                    None
                },
                data_hash,
            )?;

            // 存储加密数据
            if let Some(data) = encrypted_data {
                EncryptedDataStorage::<T>::insert(pan_id, data);
            }
            if let Some(key_backup) = owner_key_backup {
                OwnerKeyBackupStorage::<T>::insert(pan_id, key_backup);
            }

            // 发送加密课盘创建事件
            Self::deposit_event(Event::EncryptedPanCreated {
                pan_id,
                creator: who,
                privacy_mode,
                method: DivinationMethod::TimeMethod,
            });

            Ok(())
        }

        /// 更新加密数据
        ///
        /// 允许课盘所有者更新加密数据（例如更换加密密钥）
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `pan_id`: 课盘 ID
        /// - `encrypted_data`: 新的加密数据
        /// - `data_hash`: 新的数据哈希
        /// - `owner_key_backup`: 新的密钥备份
        #[pallet::call_index(12)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_encrypted_data(
            origin: OriginFor<T>,
            pan_id: u64,
            encrypted_data: BoundedVec<u8, T::MaxEncryptedLen>,
            data_hash: [u8; 32],
            owner_key_backup: [u8; 80],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证课盘存在且为调用者所有
            Pans::<T>::try_mutate(pan_id, |maybe_pan| {
                let pan = maybe_pan.as_mut().ok_or(Error::<T>::PanNotFound)?;
                ensure!(pan.creator == who, Error::<T>::NotOwner);

                // 更新哈希
                pan.sensitive_data_hash = Some(data_hash);

                Ok::<_, DispatchError>(())
            })?;

            // 更新加密数据
            EncryptedDataStorage::<T>::insert(pan_id, encrypted_data);
            OwnerKeyBackupStorage::<T>::insert(pan_id, owner_key_backup);

            Self::deposit_event(Event::EncryptedDataUpdated {
                pan_id,
                data_hash,
            });

            Ok(())
        }

        /// 删除课盘
        ///
        /// 删除课盘记录及其所有关联数据，并返还存储押金。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是课盘所有者）
        /// - `pan_id`: 课盘 ID
        ///
        /// # 返还规则
        /// - 30天内删除：100% 返还
        /// - 30天后删除：90% 返还（10% 进入国库）
        ///
        /// # 删除内容
        /// 1. 主课盘记录（Pans）
        /// 2. 用户索引（UserPans）
        /// 3. 公开列表（PublicPans，如适用）
        /// 4. 解卦数据（Interpretations）
        /// 5. AI 解读请求（AiInterpretationRequests）
        /// 6. 加密数据（EncryptedDataStorage）
        /// 7. 密钥备份（OwnerKeyBackupStorage）
        /// 8. 押金记录（DepositRecords）
        #[pallet::call_index(13)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn delete_pan(
            origin: OriginFor<T>,
            pan_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取课盘记录并验证所有权
            let pan = Pans::<T>::get(pan_id)
                .ok_or(Error::<T>::PanNotFound)?;
            ensure!(pan.creator == who, Error::<T>::NotOwner);

            // 2. 从用户索引中移除
            UserPans::<T>::mutate(&who, |pans| {
                pans.retain(|&id| id != pan_id);
            });

            // 4. 从公开列表中移除（如果是公开的）
            if pan.is_public() {
                PublicPans::<T>::mutate(|list| {
                    list.retain(|&id| id != pan_id);
                });
            }

            // 5. 移除解卦数据（如果有）
            Interpretations::<T>::remove(pan_id);

            // 6. 移除 AI 解读请求（如果有）
            AiInterpretationRequests::<T>::remove(pan_id);

            // 7. 移除加密数据（如果有）
            EncryptedDataStorage::<T>::remove(pan_id);

            // 8. 移除密钥备份（如果有）
            OwnerKeyBackupStorage::<T>::remove(pan_id);

            // 9. 删除主课盘记录
            Pans::<T>::remove(pan_id);

            // 10. 更新用户统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                stats.total_pans = stats.total_pans.saturating_sub(1);
            });

            // 9. 发送删除事件
            Self::deposit_event(Event::PanDeleted {
                pan_id,
                owner: who,
            });

            Ok(())
        }
    }

    // ============================================================================
    // 内部辅助函数
    // ============================================================================

    impl<T: Config> Pallet<T> {
        /// 获取当前时间戳（秒）
        fn get_timestamp_secs() -> u64 {
            let moment = pallet_timestamp::Pallet::<T>::get();
            let ms: u64 = moment.try_into().unwrap_or(0);
            ms / 1000
        }

        /// 检查每日起课次数限制
        fn check_daily_limit(who: &T::AccountId) -> DispatchResult {
            let today = Self::current_day();
            let count = DailyDivinationCount::<T>::get(who, today);

            ensure!(
                count < T::MaxDailyDivinations::get(),
                Error::<T>::DailyLimitExceeded
            );

            // 更新计数
            DailyDivinationCount::<T>::insert(who, today, count + 1);
            Ok(())
        }

        /// 获取当前天数（从创世块起算）
        fn current_day() -> u32 {
            let timestamp = Self::get_timestamp_secs();
            (timestamp / 86400) as u32
        }

        /// 创建课盘并存储（内部函数，默认 Public 模式）
        #[allow(clippy::too_many_arguments)]
        fn create_pan(
            creator: T::AccountId,
            method: DivinationMethod,
            san_gong: SanGong,
            param1: u8,
            param2: u8,
            param3: u8,
            shi_chen: Option<ShiChen>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            is_public: bool,
        ) -> DispatchResult {
            // 转换为隐私模式
            let privacy_mode = if is_public {
                pallet_divination_privacy::types::PrivacyMode::Public
            } else {
                pallet_divination_privacy::types::PrivacyMode::Partial
            };

            Self::create_pan_with_privacy(
                creator,
                method,
                Some(san_gong),
                Some(param1),
                Some(param2),
                Some(param3),
                shi_chen,
                question_cid,
                privacy_mode,
                None, // encrypted_fields
                None, // sensitive_data_hash
            )
        }

        /// 创建课盘并存储（支持隐私模式）
        ///
        /// # 参数
        /// - `creator`: 创建者账户
        /// - `method`: 起课方式
        /// - `san_gong`: 三宫结果（Private 模式时为 None）
        /// - `param1/2/3`: 起课参数（Private 模式时为 None）
        /// - `shi_chen`: 时辰信息
        /// - `question_cid`: 问题 CID
        /// - `privacy_mode`: 隐私模式
        /// - `encrypted_fields`: 加密字段位图
        /// - `sensitive_data_hash`: 敏感数据哈希
        #[allow(clippy::too_many_arguments)]
        fn create_pan_with_privacy(
            creator: T::AccountId,
            method: DivinationMethod,
            san_gong: Option<SanGong>,
            param1: Option<u8>,
            param2: Option<u8>,
            param3: Option<u8>,
            shi_chen: Option<ShiChen>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            privacy_mode: pallet_divination_privacy::types::PrivacyMode,
            encrypted_fields: Option<u8>,
            sensitive_data_hash: Option<[u8; 32]>,
        ) -> DispatchResult {
            // 获取新的课盘 ID
            let pan_id = NextPanId::<T>::get();
            NextPanId::<T>::put(pan_id.saturating_add(1));

            // 获取当前区块号
            let block_number = <frame_system::Pallet<T>>::block_number();

            // 提取农历信息（如果是时间起课）
            let (lunar_month, lunar_day) = if method == DivinationMethod::TimeMethod {
                (param1, param2)
            } else {
                (None, None)
            };

            // 创建课盘结构
            let pan = XiaoLiuRenPan {
                id: pan_id,
                creator: creator.clone(),
                created_at: block_number,
                privacy_mode,
                encrypted_fields,
                sensitive_data_hash,
                method: method.clone(),
                question_cid,
                param1,
                param2,
                param3,
                lunar_month,
                lunar_day,
                shi_chen,
                san_gong,
                ai_interpretation_cid: None,
            };

            // 存储课盘
            Pans::<T>::insert(pan_id, pan);

            // 更新用户课盘索引
            UserPans::<T>::try_mutate(&creator, |list| {
                list.try_push(pan_id)
                    .map_err(|_| Error::<T>::UserPansFull)
            })?;

            // 如果公开，添加到公开列表
            if privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public {
                PublicPans::<T>::try_mutate(|list| {
                    list.try_push(pan_id)
                        .map_err(|_| Error::<T>::PublicPansFull)
                })?;
            }

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&creator, |stats| {
                if stats.total_pans == 0 {
                    // 首次起课
                    let block_num: u32 = block_number.try_into().unwrap_or(0);
                    stats.first_pan_block = block_num;
                }
                stats.total_pans = stats.total_pans.saturating_add(1);
            });

            // 发送事件
            Self::deposit_event(Event::PanCreated {
                pan_id,
                creator,
                method,
            });

            Ok(())
        }

        /// 获取课盘详细分析
        pub fn get_pan_analysis(pan_id: u64) -> Option<algorithm::SanGongAnalysis> {
            Pans::<T>::get(pan_id).and_then(|pan| {
                pan.san_gong.map(|sg| algorithm::analyze_san_gong(&sg))
            })
        }

        /// 获取或创建解卦数据（懒加载）
        ///
        /// 首次查询时计算解卦结果并缓存，之后直接从缓存读取。
        /// Private 模式的课盘无法解读（san_gong 为 None）。
        ///
        /// # 参数
        /// - `pan_id`: 课盘ID
        ///
        /// # 返回
        /// 解卦核心数据，如果课盘不存在或无法解读则返回 None
        pub fn get_or_create_interpretation(
            pan_id: u64,
        ) -> Option<crate::interpretation::XiaoLiuRenInterpretation> {
            // 1. 检查缓存
            if let Some(interpretation) = Interpretations::<T>::get(pan_id) {
                return Some(interpretation);
            }

            // 2. 获取课盘
            let pan = Pans::<T>::get(pan_id)?;

            // 3. 检查是否可解读（Private 模式无法解读）
            let san_gong = pan.san_gong?;

            // 4. 计算解卦（使用道家流派）
            let interpretation = crate::interpretation::interpret(
                &san_gong,
                pan.shi_chen,
                crate::types::XiaoLiuRenSchool::DaoJia,
            );

            // 5. 缓存结果
            Interpretations::<T>::insert(pan_id, interpretation);

            Some(interpretation)
        }

        /// 批量获取解卦数据
        ///
        /// # 参数
        /// - `pan_ids`: 课盘ID列表
        ///
        /// # 返回
        /// 解卦结果列表
        pub fn get_interpretations_batch(
            pan_ids: Vec<u64>,
        ) -> Vec<Option<crate::interpretation::XiaoLiuRenInterpretation>> {
            pan_ids
                .into_iter()
                .map(Self::get_or_create_interpretation)
                .collect()
        }
    }

}

// ============================================================================
// DivinationProvider trait 实现（供 pallet-divination-common 使用）
// ============================================================================

use pallet_divination_common::{
    DivinationProvider, DivinationType, RarityInput,
};

/// 小六壬占卜提供者实现
pub struct XiaoLiuRenDivinationProvider<T>(sp_std::marker::PhantomData<T>);

impl<T: pallet::Config> DivinationProvider<T::AccountId> for XiaoLiuRenDivinationProvider<T> {
    /// 检查结果是否存在
    fn result_exists(divination_type: DivinationType, result_id: u64) -> bool {
        if divination_type == DivinationType::XiaoLiuRen {
            pallet::Pans::<T>::contains_key(result_id)
        } else {
            false
        }
    }

    /// 获取结果创建者
    fn result_creator(divination_type: DivinationType, result_id: u64) -> Option<T::AccountId> {
        if divination_type == DivinationType::XiaoLiuRen {
            pallet::Pans::<T>::get(result_id).map(|pan| pan.creator)
        } else {
            None
        }
    }

    /// 获取稀有度数据
    fn rarity_data(divination_type: DivinationType, result_id: u64) -> Option<RarityInput> {
        if divination_type == DivinationType::XiaoLiuRen {
            pallet::Pans::<T>::get(result_id).and_then(|pan| {
                // Private 模式无法计算稀有度
                let san_gong = pan.san_gong?;

                // 计算稀有度分数
                let primary_score = if san_gong.is_pure() {
                    // 纯宫（三宫相同）非常稀有
                    90u8
                } else if san_gong.is_all_auspicious() {
                    // 全吉
                    70u8
                } else if san_gong.is_all_inauspicious() {
                    // 全凶
                    60u8
                } else {
                    // 普通
                    30u8
                };

                let secondary_score = san_gong.fortune_level() * 10;

                Some(RarityInput {
                    primary_score,
                    secondary_score,
                    is_special_date: false, // 可以扩展检查特殊日期
                    is_special_combination: san_gong.is_pure(),
                    custom_factors: [0, 0, 0, 0],
                })
            })
        } else {
            None
        }
    }

    /// 获取占卜结果摘要
    fn result_summary(divination_type: DivinationType, result_id: u64) -> Option<sp_std::vec::Vec<u8>> {
        if divination_type == DivinationType::XiaoLiuRen {
            pallet::Pans::<T>::get(result_id).and_then(|pan| {
                // Private 模式无法获取摘要
                let san_gong = pan.san_gong?;

                // 返回三宫结果的简要描述
                Some(sp_std::vec![
                    san_gong.yue_gong.index(),
                    san_gong.ri_gong.index(),
                    san_gong.shi_gong.index(),
                    san_gong.fortune_level(),
                    if san_gong.is_pure() { 1 } else { 0 },
                    if san_gong.is_all_auspicious() { 1 } else { 0 },
                ])
            })
        } else {
            None
        }
    }

    /// 检查占卜结果是否可以铸造为 NFT
    fn is_nftable(divination_type: DivinationType, result_id: u64) -> bool {
        if divination_type == DivinationType::XiaoLiuRen {
            // 检查结果存在且公开
            pallet::Pans::<T>::get(result_id)
                .map(|pan| pan.is_public())
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// 标记占卜结果已被铸造为 NFT
    fn mark_as_nfted(_divination_type: DivinationType, _result_id: u64) {
        // 小六壬暂不实现 NFT 标记，因为课盘结构中没有 is_nfted 字段
        // 如需此功能，可以添加额外存储项
    }
}
