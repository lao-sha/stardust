//! # 大六壬排盘模块 (Da Liu Ren Divination Pallet)
//!
//! 本模块实现区块链大六壬排盘系统，提供式盘生成、存储和AI解读功能。
//!
//! ## 概述
//!
//! 大六壬是中国古代三式之一（太乙、奇门、六壬），以天人合一、
//! 阴阳五行为理论基础，通过起课、定三传来预测吉凶。
//!
//! ## 主要功能
//!
//! - **起课方式**: 支持时间起课、随机起课、手动指定三种方式
//! - **式盘计算**: 天盘、四课、三传、天将自动排布
//! - **九种课式**: 贼克、比用、涉害、遥克、昂星、别责、八专、伏吟、返吟
//! - **AI解读**: 支持AI解读请求和结果存储
//! - **NFT铸造**: 可将式盘铸造为NFT
//!
//! ## 大六壬核心概念
//!
//! - **天盘**: 月将加占时，十二地支顺时针旋转
//! - **四课**: 日干阳神、干阴神、日支阳神、支阴神
//! - **三传**: 初传、中传、末传（根据九种课式推导）
//! - **天将**: 十二天将（贵人为首，顺逆排布）
//! - **空亡**: 旬空计算
//!
//! ## 使用示例
//!
//! ```ignore
//! // 时间起课
//! DaLiuRen::divine_by_time(origin, year_gz, month_gz, day_gz, hour_gz, yue_jiang, zhan_shi, is_day, question_cid)?;
//!
//! // 随机起课
//! DaLiuRen::divine_random(origin, day_gz, question_cid)?;
//!
//! // 请求AI解读
//! DaLiuRen::request_ai_interpretation(origin, pan_id)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

mod algorithm;
mod interpretation;
mod interpretation_algorithm;
pub mod ocw_tee;
mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use algorithm::*;
pub use types::*;
pub use interpretation::*;
pub use interpretation_algorithm::*;

/// 权重 trait
pub trait DaLiuRenWeightInfo {
    fn divine_by_time() -> frame_support::weights::Weight;
    fn divine_random() -> frame_support::weights::Weight;
    fn divine_manual() -> frame_support::weights::Weight;
    fn request_ai_interpretation() -> frame_support::weights::Weight;
    fn submit_ai_interpretation() -> frame_support::weights::Weight;
    fn set_pan_visibility() -> frame_support::weights::Weight;
}

/// 默认权重实现
impl DaLiuRenWeightInfo for () {
    fn divine_by_time() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(50_000_000, 0)
    }
    fn divine_random() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(60_000_000, 0)
    }
    fn divine_manual() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(45_000_000, 0)
    }
    fn request_ai_interpretation() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(30_000_000, 0)
    }
    fn submit_ai_interpretation() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(35_000_000, 0)
    }
    fn set_pan_visibility() -> frame_support::weights::Weight {
        frame_support::weights::Weight::from_parts(20_000_000, 0)
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, Randomness, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Zero, Saturating};
    

    // 导入押金相关类型
    pub use pallet_divination_common::deposit::{
        PrivacyMode as DepositPrivacyMode,
        DepositRecord,
    };

    /// 货币类型别名
    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 随机数生成器
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 每日每用户最大起课次数
        #[pallet::constant]
        type MaxDailyDivinations: Get<u32>;

        /// 加密数据最大长度（默认: 512 bytes）
        #[pallet::constant]
        type MaxEncryptedLen: Get<u32>;

        /// 起课费用
        #[pallet::constant]
        type DivinationFee: Get<BalanceOf<Self>>;

        /// AI 解读费用
        #[pallet::constant]
        type AiInterpretationFee: Get<BalanceOf<Self>>;

        /// AI 解读提交者（可信任的 AI 服务账户）
        type AiSubmitter: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

        /// 权重信息
        type WeightInfo: DaLiuRenWeightInfo;

    }

    // ========================================================================
    // 存储项
    // ========================================================================

    /// 下一个式盘 ID
    #[pallet::storage]
    pub type NextPanId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 式盘存储
    /// 键: 式盘 ID
    /// 值: 大六壬式盘
    #[pallet::storage]
    pub type Pans<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        DaLiuRenPan<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    >;

    /// 用户式盘索引
    /// 键: (用户账户, 式盘 ID)
    /// 值: 是否存在
    #[pallet::storage]
    pub type UserPans<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, u64, bool, ValueQuery>;

    /// 公开式盘索引
    /// 键: 式盘 ID
    /// 值: 创建区块
    #[pallet::storage]
    pub type PublicPans<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>>;

    /// 每日起课计数
    /// 键: (用户账户, 日期戳)
    /// 值: 起课次数
    #[pallet::storage]
    pub type DailyPanCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        u32,
        ValueQuery,
    >;

    /// AI 解读请求队列
    /// 键: 式盘 ID
    /// 值: 请求区块
    #[pallet::storage]
    pub type AiInterpretationRequests<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BlockNumberFor<T>>;

    /// 用户统计数据
    #[pallet::storage]
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

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 式盘已创建
        PanCreated {
            pan_id: u64,
            creator: T::AccountId,
            ke_shi: u8,
            ge_ju: u8,
        },

        /// AI 解读已请求
        AiInterpretationRequested {
            pan_id: u64,
            requester: T::AccountId,
        },

        /// AI 解读已提交
        AiInterpretationSubmitted {
            pan_id: u64,
            cid: BoundedVec<u8, T::MaxCidLen>,
        },

        /// 式盘可见性已更改
        PanVisibilityChanged {
            pan_id: u64,
            is_public: bool,
        },

        /// 加密式盘创建成功
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

        /// 式盘已删除
        /// [式盘ID, 所有者]
        PanDeleted {
            pan_id: u64,
            owner: T::AccountId,
        },
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 式盘不存在
        PanNotFound,

        /// 无权操作
        NotAuthorized,

        /// 超出每日限额
        DailyLimitExceeded,

        /// 余额不足
        InsufficientBalance,

        /// CID 过长
        CidTooLong,

        /// AI 解读已请求
        AiInterpretationAlreadyRequested,

        /// AI 解读未请求
        AiInterpretationNotRequested,

        /// AI 解读已完成
        AiInterpretationAlreadySubmitted,

        /// 无效的干支组合
        InvalidGanZhi,

        /// 无效的月将
        InvalidYueJiang,

        /// 无效的占时
        InvalidZhanShi,

        /// 无效的隐私模式
        InvalidPrivacyMode,

        /// 加密数据缺失
        EncryptedDataMissing,

        /// 公开模式不能存储加密数据
        PublicModeNoEncryptedData,

        /// 私有模式需要加密数据
        PrivateModeRequiresEncryptedData,

        /// 无法解读（私有模式无计算数据）
        CannotInterpretPrivateMode,
    }

    // ========================================================================
    // 调用
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 时间起课
        ///
        /// 根据指定的年月日时干支和月将、占时进行起课。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `year_gz`: 年干支 (天干索引, 地支索引)
        /// - `month_gz`: 月干支
        /// - `day_gz`: 日干支
        /// - `hour_gz`: 时干支
        /// - `yue_jiang`: 月将（地支索引）
        /// - `zhan_shi`: 占时（地支索引）
        /// - `is_day`: 是否昼占
        /// - `question_cid`: 占问事项 CID（可选）
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::divine_by_time())]
        pub fn divine_by_time(
            origin: OriginFor<T>,
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
            yue_jiang: u8,
            zhan_shi: u8,
            is_day: bool,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查每日限额
            Self::check_daily_limit(&who)?;

            // 收取费用
            Self::charge_fee(&who, T::DivinationFee::get())?;

            // 转换参数
            let year = (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1));
            let month = (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1));
            let day = (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1));
            let hour = (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1));
            let yj = DiZhi::from_index(yue_jiang);
            let zs = DiZhi::from_index(zhan_shi);

            // 执行起课
            Self::do_divine(
                who,
                DivinationMethod::TimeMethod,
                year,
                month,
                day,
                hour,
                yj,
                zs,
                is_day,
                question_cid,
            )
        }

        /// 公历时间起课
        ///
        /// 此方法使用 pallet-almanac 自动将公历日期转换为四柱干支，
        /// 然后进行大六壬起课。用户无需手动计算干支和月将。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_cid`: 占问事项 CID（可选）
        ///
        /// # 说明
        /// - 月将由当前月份自动推算（以中气为准）
        /// - 占时由小时转换为地支时辰
        /// - 昼夜判断由小时自动计算（6-18时为昼）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn divine_by_solar_time(
            origin: OriginFor<T>,
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            ensure!(solar_year >= 1901 && solar_year <= 2100, Error::<T>::InvalidGanZhi);
            ensure!(solar_month >= 1 && solar_month <= 12, Error::<T>::InvalidGanZhi);
            ensure!(solar_day >= 1 && solar_day <= 31, Error::<T>::InvalidGanZhi);
            ensure!(hour < 24, Error::<T>::InvalidGanZhi);

            // 检查每日限额
            Self::check_daily_limit(&who)?;

            // 收取费用
            Self::charge_fee(&who, T::DivinationFee::get())?;

            // 调用 pallet-almanac 计算四柱干支
            let pillars = pallet_almanac::four_pillars(solar_year, solar_month, solar_day, hour);

            // 转换为本模块的干支类型
            let year_gz = (
                TianGan::from_index(pillars.year.gan),
                DiZhi::from_index(pillars.year.zhi),
            );
            let month_gz = (
                TianGan::from_index(pillars.month.gan),
                DiZhi::from_index(pillars.month.zhi),
            );
            let day_gz = (
                TianGan::from_index(pillars.day.gan),
                DiZhi::from_index(pillars.day.zhi),
            );
            let hour_gz = (
                TianGan::from_index(pillars.hour.gan),
                DiZhi::from_index(pillars.hour.zhi),
            );

            // 计算月将（根据节气，以中气为准）
            // 月将是太阳所在宫位对应的地支
            // 简化算法：使用月份推算
            // 正月雨水后用亥将，二月春分后用戌将...以此类推
            let yue_jiang = Self::calc_yue_jiang_from_month(solar_month);

            // 占时为时辰对应的地支
            let zhan_shi = DiZhi::from_index(pallet_almanac::hour_to_dizhi_num(hour).saturating_sub(1));

            // 昼夜判断：6-18时为昼
            let is_day = hour >= 6 && hour < 18;

            // 执行起课
            Self::do_divine(
                who,
                DivinationMethod::TimeMethod,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                yue_jiang,
                zhan_shi,
                is_day,
                question_cid,
            )
        }

        /// 随机起课
        ///
        /// 使用链上随机数生成月将和占时进行起课。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `day_gz`: 日干支
        /// - `question_cid`: 占问事项 CID（可选）
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::divine_random())]
        pub fn divine_random(
            origin: OriginFor<T>,
            day_gz: (u8, u8),
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查每日限额
            Self::check_daily_limit(&who)?;

            // 收取费用
            Self::charge_fee(&who, T::DivinationFee::get())?;

            // 生成随机数
            let (random_hash, _) = T::Randomness::random(&b"daliuren"[..]);
            let random_bytes: [u8; 32] = random_hash.as_ref().try_into().unwrap_or([0u8; 32]);

            // 从随机数生成参数
            let (yue_jiang, zhan_shi, is_day) = random_to_params(&random_bytes);

            // 使用日干支作为基础
            let day = (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1));

            // 简化处理：年月时使用默认值
            let year = (TianGan::Jia, DiZhi::Zi);
            let month = (TianGan::Jia, DiZhi::Zi);
            let hour = (TianGan::from_index(random_bytes[3] % 10), zhan_shi);

            Self::do_divine(
                who,
                DivinationMethod::RandomMethod,
                year,
                month,
                day,
                hour,
                yue_jiang,
                zhan_shi,
                is_day,
                question_cid,
            )
        }

        /// 手动指定起课
        ///
        /// 完全手动指定所有参数进行起课，用于复盘或教学。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `year_gz`: 年干支
        /// - `month_gz`: 月干支
        /// - `day_gz`: 日干支
        /// - `hour_gz`: 时干支
        /// - `yue_jiang`: 月将
        /// - `zhan_shi`: 占时
        /// - `is_day`: 是否昼占
        /// - `question_cid`: 占问事项 CID（可选）
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::divine_manual())]
        pub fn divine_manual(
            origin: OriginFor<T>,
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
            yue_jiang: u8,
            zhan_shi: u8,
            is_day: bool,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查每日限额
            Self::check_daily_limit(&who)?;

            // 收取费用
            Self::charge_fee(&who, T::DivinationFee::get())?;

            // 转换参数
            let year = (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1));
            let month = (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1));
            let day = (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1));
            let hour = (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1));
            let yj = DiZhi::from_index(yue_jiang);
            let zs = DiZhi::from_index(zhan_shi);

            Self::do_divine(
                who,
                DivinationMethod::ManualMethod,
                year,
                month,
                day,
                hour,
                yj,
                zs,
                is_day,
                question_cid,
            )
        }

        /// 请求 AI 解读（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::request_interpretation`
        /// 新的统一 AI 解读系统支持多种 AI 模型选择、Oracle 质押评分、争议退款。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `pan_id`: 式盘 ID
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::request_ai_interpretation())]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::request_interpretation"
        )]
        pub fn request_ai_interpretation(origin: OriginFor<T>, pan_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查式盘存在
            let pan = Pans::<T>::get(pan_id).ok_or(Error::<T>::PanNotFound)?;

            // 检查权限
            ensure!(pan.creator == who, Error::<T>::NotAuthorized);

            // 检查是否已请求
            ensure!(
                !AiInterpretationRequests::<T>::contains_key(pan_id),
                Error::<T>::AiInterpretationAlreadyRequested
            );

            // 检查是否已有解读
            ensure!(
                pan.ai_interpretation_cid.is_none(),
                Error::<T>::AiInterpretationAlreadySubmitted
            );

            // 收取费用
            Self::charge_fee(&who, T::AiInterpretationFee::get())?;

            // 记录请求
            let current_block = <frame_system::Pallet<T>>::block_number();
            AiInterpretationRequests::<T>::insert(pan_id, current_block);

            // 更新统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                stats.ai_interpretations = stats.ai_interpretations.saturating_add(1);
            });

            // 发出事件
            Self::deposit_event(Event::AiInterpretationRequested {
                pan_id,
                requester: who,
            });

            Ok(())
        }

        /// 提交 AI 解读结果（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::submit_result`
        ///
        /// # 参数
        /// - `origin`: AI 服务来源
        /// - `pan_id`: 式盘 ID
        /// - `interpretation_cid`: 解读内容 CID
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::submit_ai_interpretation())]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::submit_result"
        )]
        pub fn submit_ai_interpretation(
            origin: OriginFor<T>,
            pan_id: u64,
            interpretation_cid: BoundedVec<u8, T::MaxCidLen>,
        ) -> DispatchResult {
            // 验证 AI 提交者权限
            let _submitter = T::AiSubmitter::ensure_origin(origin)?;

            // 检查式盘存在
            ensure!(Pans::<T>::contains_key(pan_id), Error::<T>::PanNotFound);

            // 检查是否已请求
            ensure!(
                AiInterpretationRequests::<T>::contains_key(pan_id),
                Error::<T>::AiInterpretationNotRequested
            );

            // 更新式盘
            Pans::<T>::try_mutate(pan_id, |maybe_pan| -> DispatchResult {
                let pan = maybe_pan.as_mut().ok_or(Error::<T>::PanNotFound)?;
                pan.ai_interpretation_cid = Some(interpretation_cid.clone());
                Ok(())
            })?;

            // 移除请求
            AiInterpretationRequests::<T>::remove(pan_id);

            // 发出事件
            Self::deposit_event(Event::AiInterpretationSubmitted {
                pan_id,
                cid: interpretation_cid,
            });

            Ok(())
        }

        /// 设置式盘可见性
        ///
        /// 设置式盘是否公开可见。
        /// 注意：此方法通过设置 privacy_mode 来控制可见性
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `pan_id`: 式盘 ID
        /// - `is_public`: 是否公开
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::set_pan_visibility())]
        pub fn set_pan_visibility(
            origin: OriginFor<T>,
            pan_id: u64,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 更新式盘
            Pans::<T>::try_mutate(pan_id, |maybe_pan| -> DispatchResult {
                let pan = maybe_pan.as_mut().ok_or(Error::<T>::PanNotFound)?;

                // 检查权限
                ensure!(pan.creator == who, Error::<T>::NotAuthorized);

                // 使用 privacy_mode 控制可见性
                if is_public {
                    pan.privacy_mode = pallet_divination_privacy::types::PrivacyMode::Public;
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    PublicPans::<T>::insert(pan_id, current_block);
                } else {
                    pan.privacy_mode = pallet_divination_privacy::types::PrivacyMode::Partial;
                    PublicPans::<T>::remove(pan_id);
                }

                Ok(())
            })?;

            // 发出事件
            Self::deposit_event(Event::PanVisibilityChanged { pan_id, is_public });

            Ok(())
        }

        /// 加密时间起课
        ///
        /// 支持三种隐私模式的时间起课：
        /// - Public (0): 所有数据明文存储
        /// - Partial (1): 计算数据明文，敏感数据加密
        /// - Private (2): 所有数据加密，仅存储元数据
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `privacy_mode`: 隐私模式 (0=Public, 1=Partial, 2=Private)
        /// - `year_gz`: 年干支（Private 模式为 None）
        /// - `month_gz`: 月干支
        /// - `day_gz`: 日干支
        /// - `hour_gz`: 时干支
        /// - `yue_jiang`: 月将
        /// - `zhan_shi`: 占时
        /// - `is_day`: 是否昼占
        /// - `question_cid`: 问题 CID（可选）
        /// - `encrypted_data`: 加密数据（Partial/Private 模式必需）
        /// - `data_hash`: 敏感数据哈希（用于完整性验证）
        /// - `owner_key_backup`: 所有者密钥备份
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn divine_by_time_encrypted(
            origin: OriginFor<T>,
            privacy_mode: u8,
            year_gz: Option<(u8, u8)>,
            month_gz: Option<(u8, u8)>,
            day_gz: Option<(u8, u8)>,
            hour_gz: Option<(u8, u8)>,
            yue_jiang: Option<u8>,
            zhan_shi: Option<u8>,
            is_day: Option<bool>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
            data_hash: Option<[u8; 32]>,
            owner_key_backup: Option<[u8; 80]>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 转换隐私模式
            let mode = match privacy_mode {
                0 => pallet_divination_privacy::types::PrivacyMode::Public,
                1 => pallet_divination_privacy::types::PrivacyMode::Partial,
                2 => pallet_divination_privacy::types::PrivacyMode::Private,
                _ => return Err(Error::<T>::InvalidPrivacyMode.into()),
            };

            // 校验参数
            match mode {
                pallet_divination_privacy::types::PrivacyMode::Public => {
                    // Public 模式不能有加密数据
                    ensure!(encrypted_data.is_none(), Error::<T>::PublicModeNoEncryptedData);
                    // Public 模式需要所有明文数据
                    ensure!(year_gz.is_some() && month_gz.is_some() && day_gz.is_some() && hour_gz.is_some(), Error::<T>::InvalidGanZhi);
                    ensure!(yue_jiang.is_some() && zhan_shi.is_some() && is_day.is_some(), Error::<T>::InvalidYueJiang);
                },
                pallet_divination_privacy::types::PrivacyMode::Partial => {
                    // Partial 模式需要计算数据
                    ensure!(year_gz.is_some() && month_gz.is_some() && day_gz.is_some() && hour_gz.is_some(), Error::<T>::InvalidGanZhi);
                    ensure!(yue_jiang.is_some() && zhan_shi.is_some() && is_day.is_some(), Error::<T>::InvalidYueJiang);
                },
                pallet_divination_privacy::types::PrivacyMode::Private => {
                    // Private 模式需要加密数据
                    ensure!(encrypted_data.is_some(), Error::<T>::PrivateModeRequiresEncryptedData);
                },
            }

            // 检查每日限额
            Self::check_daily_limit(&who)?;

            // 收取费用
            Self::charge_fee(&who, T::DivinationFee::get())?;

            // 执行加密起课
            Self::do_divine_encrypted(
                who,
                mode,
                DivinationMethod::TimeMethod,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                yue_jiang,
                zhan_shi,
                is_day,
                question_cid,
                encrypted_data,
                data_hash,
                owner_key_backup,
            )
        }

        /// 更新加密数据
        ///
        /// 更新式盘的加密数据（仅限 Partial/Private 模式）
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是式盘所有者）
        /// - `pan_id`: 式盘 ID
        /// - `encrypted_data`: 新的加密数据
        /// - `data_hash`: 新的数据哈希
        /// - `owner_key_backup`: 新的密钥备份
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn update_encrypted_data(
            origin: OriginFor<T>,
            pan_id: u64,
            encrypted_data: BoundedVec<u8, T::MaxEncryptedLen>,
            data_hash: [u8; 32],
            owner_key_backup: [u8; 80],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查式盘存在并验证权限
            Pans::<T>::try_mutate(pan_id, |maybe_pan| -> DispatchResult {
                let pan = maybe_pan.as_mut().ok_or(Error::<T>::PanNotFound)?;

                // 检查权限
                ensure!(pan.creator == who, Error::<T>::NotAuthorized);

                // 公开模式不能存储加密数据
                ensure!(
                    !pan.is_public(),
                    Error::<T>::PublicModeNoEncryptedData
                );

                // 更新敏感数据哈希
                pan.sensitive_data_hash = Some(data_hash);

                Ok(())
            })?;

            // 存储加密数据
            EncryptedDataStorage::<T>::insert(pan_id, encrypted_data);
            OwnerKeyBackupStorage::<T>::insert(pan_id, owner_key_backup);

            // 发出事件
            Self::deposit_event(Event::EncryptedDataUpdated {
                pan_id,
                data_hash,
            });

            Ok(())
        }

        /// 删除式盘
        ///
        /// 删除式盘记录及其所有关联数据，并返还存储押金。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是式盘所有者）
        /// - `pan_id`: 式盘 ID
        ///
        /// # 返还规则
        /// - 30天内删除：100% 返还
        /// - 30天后删除：90% 返还（10% 进入国库）
        ///
        /// # 删除内容
        /// 1. 主式盘记录（Pans）
        /// 2. 用户索引（UserPans）
        /// 3. 公开列表（PublicPans，如适用）
        /// 4. AI 解读请求（AiInterpretationRequests）
        /// 5. 加密数据（EncryptedDataStorage）
        /// 6. 密钥备份（OwnerKeyBackupStorage）
        /// 7. 押金记录（DepositRecords）
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn delete_pan(
            origin: OriginFor<T>,
            pan_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取式盘记录并验证所有权
            let pan = Pans::<T>::get(pan_id)
                .ok_or(Error::<T>::PanNotFound)?;
            ensure!(pan.creator == who, Error::<T>::NotAuthorized);

            // 2. 从用户索引中移除
            UserPans::<T>::remove(&who, pan_id);

            // 4. 从公开列表中移除（如果是公开的）
            if pan.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public {
                PublicPans::<T>::remove(pan_id);
            }

            // 5. 移除 AI 解读请求（如果有）
            AiInterpretationRequests::<T>::remove(pan_id);

            // 6. 移除加密数据（如果有）
            EncryptedDataStorage::<T>::remove(pan_id);

            // 7. 移除密钥备份（如果有）
            OwnerKeyBackupStorage::<T>::remove(pan_id);

            // 8. 删除主式盘记录
            Pans::<T>::remove(pan_id);

            // 7. 发送删除事件
            Self::deposit_event(Event::PanDeleted {
                pan_id,
                owner: who,
            });

            Ok(())
        }
    }

    // ========================================================================
    // 内部函数
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 执行起课
        ///
        /// 核心起课逻辑，计算天盘、四课、三传等。
        /// 默认使用 Public 模式（向后兼容）
        fn do_divine(
            who: T::AccountId,
            method: DivinationMethod,
            year_gz: (TianGan, DiZhi),
            month_gz: (TianGan, DiZhi),
            day_gz: (TianGan, DiZhi),
            hour_gz: (TianGan, DiZhi),
            yue_jiang: DiZhi,
            zhan_shi: DiZhi,
            is_day: bool,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        ) -> DispatchResult {
            let current_block = <frame_system::Pallet<T>>::block_number();

            // 计算天盘
            let tian_pan = calculate_tian_pan(yue_jiang, zhan_shi);

            // 计算天将盘
            let tian_jiang_pan = calculate_tian_jiang_pan(&tian_pan, day_gz.0, is_day);

            // 计算四课
            let si_ke = calculate_si_ke(&tian_pan, &tian_jiang_pan, day_gz.0, day_gz.1);

            // 计算三传
            let (san_chuan, ke_shi, ge_ju) =
                calculate_san_chuan(&tian_pan, &tian_jiang_pan, &si_ke, day_gz.0, day_gz.1);

            // 计算空亡
            let xun_kong = calculate_xun_kong(day_gz.0, day_gz.1);

            // 生成式盘 ID
            let pan_id = NextPanId::<T>::get();
            NextPanId::<T>::put(pan_id.saturating_add(1));

            // 创建式盘（使用 Private 模式，所有字段使用 Some 包装）
            let pan = DaLiuRenPan {
                id: pan_id,
                creator: who.clone(),
                created_at: current_block,
                // 隐私控制字段（默认 Private，用户可后续更改）
                privacy_mode: pallet_divination_privacy::types::PrivacyMode::Private,
                encrypted_fields: None,
                sensitive_data_hash: None,
                // 起课信息
                method,
                question_cid,
                // 时间信息（使用 Some 包装）
                year_gz: Some(year_gz),
                month_gz: Some(month_gz),
                day_gz: Some(day_gz),
                hour_gz: Some(hour_gz),
                // 起课参数
                yue_jiang: Some(yue_jiang),
                zhan_shi: Some(zhan_shi),
                is_day: Some(is_day),
                // 式盘信息
                tian_pan: Some(tian_pan),
                tian_jiang_pan: Some(tian_jiang_pan),
                si_ke: Some(si_ke),
                san_chuan: Some(san_chuan),
                // 课式与格局
                ke_shi: Some(ke_shi),
                ge_ju: Some(ge_ju),
                // 空亡
                xun_kong: Some(xun_kong),
                // AI 解读
                ai_interpretation_cid: None,
            };

            // 存储式盘
            Pans::<T>::insert(pan_id, pan);
            UserPans::<T>::insert(&who, pan_id, true);

            // 更新每日计数
            let day_stamp = Self::get_day_stamp();
            DailyPanCount::<T>::mutate(&who, day_stamp, |count| {
                *count = count.saturating_add(1);
            });

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                stats.total_pans = stats.total_pans.saturating_add(1);
                if stats.first_pan_block == 0 {
                    stats.first_pan_block = Self::block_to_u32(current_block);
                }
            });

            // 发出事件
            Self::deposit_event(Event::PanCreated {
                pan_id,
                creator: who,
                ke_shi: ke_shi as u8,
                ge_ju: ge_ju as u8,
            });

            Ok(())
        }

        /// 执行加密起课
        ///
        /// 支持三种隐私模式的起课逻辑
        #[allow(clippy::too_many_arguments)]
        fn do_divine_encrypted(
            who: T::AccountId,
            privacy_mode: pallet_divination_privacy::types::PrivacyMode,
            method: DivinationMethod,
            year_gz: Option<(u8, u8)>,
            month_gz: Option<(u8, u8)>,
            day_gz: Option<(u8, u8)>,
            hour_gz: Option<(u8, u8)>,
            yue_jiang: Option<u8>,
            zhan_shi: Option<u8>,
            is_day: Option<bool>,
            question_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
            data_hash: Option<[u8; 32]>,
            owner_key_backup: Option<[u8; 80]>,
        ) -> DispatchResult {
            let current_block = <frame_system::Pallet<T>>::block_number();

            // 生成式盘 ID
            let pan_id = NextPanId::<T>::get();
            NextPanId::<T>::put(pan_id.saturating_add(1));

            // 根据隐私模式创建式盘
            let (pan, ke_shi_val, ge_ju_val) = match privacy_mode {
                pallet_divination_privacy::types::PrivacyMode::Public |
                pallet_divination_privacy::types::PrivacyMode::Partial => {
                    // Public/Partial 模式：计算所有数据
                    // 安全检查：确保所有必需参数存在
                    let year_gz_val = year_gz.ok_or(Error::<T>::InvalidGanZhi)?;
                    let month_gz_val = month_gz.ok_or(Error::<T>::InvalidGanZhi)?;
                    let day_gz_val = day_gz.ok_or(Error::<T>::InvalidGanZhi)?;
                    let hour_gz_val = hour_gz.ok_or(Error::<T>::InvalidGanZhi)?;
                    let yue_jiang_val = yue_jiang.ok_or(Error::<T>::InvalidYueJiang)?;
                    let zhan_shi_val = zhan_shi.ok_or(Error::<T>::InvalidZhanShi)?;
                    let is_day_val = is_day.ok_or(Error::<T>::InvalidPrivacyMode)?;
                    
                    let year = (
                        TianGan::from_index(year_gz_val.0),
                        DiZhi::from_index(year_gz_val.1),
                    );
                    let month = (
                        TianGan::from_index(month_gz_val.0),
                        DiZhi::from_index(month_gz_val.1),
                    );
                    let day = (
                        TianGan::from_index(day_gz_val.0),
                        DiZhi::from_index(day_gz_val.1),
                    );
                    let hour = (
                        TianGan::from_index(hour_gz_val.0),
                        DiZhi::from_index(hour_gz_val.1),
                    );
                    let yj = DiZhi::from_index(yue_jiang_val);
                    let zs = DiZhi::from_index(zhan_shi_val);
                    let is_d = is_day_val;

                    // 计算天盘
                    let tian_pan = calculate_tian_pan(yj, zs);
                    // 计算天将盘
                    let tian_jiang_pan = calculate_tian_jiang_pan(&tian_pan, day.0, is_d);
                    // 计算四课
                    let si_ke = calculate_si_ke(&tian_pan, &tian_jiang_pan, day.0, day.1);
                    // 计算三传
                    let (san_chuan, ke_shi, ge_ju) =
                        calculate_san_chuan(&tian_pan, &tian_jiang_pan, &si_ke, day.0, day.1);
                    // 计算空亡
                    let xun_kong = calculate_xun_kong(day.0, day.1);

                    let pan = DaLiuRenPan {
                        id: pan_id,
                        creator: who.clone(),
                        created_at: current_block,
                        privacy_mode,
                        encrypted_fields: if privacy_mode == pallet_divination_privacy::types::PrivacyMode::Partial {
                            Some(0x01) // bit 0: question_cid 已加密
                        } else {
                            None
                        },
                        sensitive_data_hash: data_hash,
                        method,
                        question_cid,
                        year_gz: Some(year),
                        month_gz: Some(month),
                        day_gz: Some(day),
                        hour_gz: Some(hour),
                        yue_jiang: Some(yj),
                        zhan_shi: Some(zs),
                        is_day: Some(is_d),
                        tian_pan: Some(tian_pan),
                        tian_jiang_pan: Some(tian_jiang_pan),
                        si_ke: Some(si_ke),
                        san_chuan: Some(san_chuan),
                        ke_shi: Some(ke_shi),
                        ge_ju: Some(ge_ju),
                        xun_kong: Some(xun_kong),
                        ai_interpretation_cid: None,
                    };

                    (pan, ke_shi as u8, ge_ju as u8)
                },
                pallet_divination_privacy::types::PrivacyMode::Private => {
                    // Private 模式：不存储任何计算数据
                    let pan = DaLiuRenPan {
                        id: pan_id,
                        creator: who.clone(),
                        created_at: current_block,
                        privacy_mode,
                        encrypted_fields: Some(0x03), // bit 0-1: 所有敏感数据已加密
                        sensitive_data_hash: data_hash,
                        method,
                        question_cid: None, // Private 模式不存储问题
                        year_gz: None,
                        month_gz: None,
                        day_gz: None,
                        hour_gz: None,
                        yue_jiang: None,
                        zhan_shi: None,
                        is_day: None,
                        tian_pan: None,
                        tian_jiang_pan: None,
                        si_ke: None,
                        san_chuan: None,
                        ke_shi: None,
                        ge_ju: None,
                        xun_kong: None,
                        ai_interpretation_cid: None,
                    };

                    (pan, 0u8, 0u8)
                },
            };

            // 存储式盘
            Pans::<T>::insert(pan_id, pan);
            UserPans::<T>::insert(&who, pan_id, true);

            // 存储加密数据（如果提供）
            if let Some(data) = encrypted_data {
                EncryptedDataStorage::<T>::insert(pan_id, data);
            }
            if let Some(backup) = owner_key_backup {
                OwnerKeyBackupStorage::<T>::insert(pan_id, backup);
            }

            // 更新公开索引（仅 Public 模式）
            if privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public {
                PublicPans::<T>::insert(pan_id, current_block);
            }

            // 更新每日计数
            let day_stamp = Self::get_day_stamp();
            DailyPanCount::<T>::mutate(&who, day_stamp, |count| {
                *count = count.saturating_add(1);
            });

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                stats.total_pans = stats.total_pans.saturating_add(1);
                if stats.first_pan_block == 0 {
                    stats.first_pan_block = Self::block_to_u32(current_block);
                }
            });

            // 发出事件
            Self::deposit_event(Event::EncryptedPanCreated {
                pan_id,
                creator: who.clone(),
                privacy_mode,
                method,
            });

            // 如果是 Public/Partial 模式，也发出标准事件
            if privacy_mode != pallet_divination_privacy::types::PrivacyMode::Private {
                Self::deposit_event(Event::PanCreated {
                    pan_id,
                    creator: who,
                    ke_shi: ke_shi_val,
                    ge_ju: ge_ju_val,
                });
            }

            Ok(())
        }

        /// 检查每日限额
        fn check_daily_limit(who: &T::AccountId) -> DispatchResult {
            let day_stamp = Self::get_day_stamp();
            let count = DailyPanCount::<T>::get(who, day_stamp);

            ensure!(
                count < T::MaxDailyDivinations::get(),
                Error::<T>::DailyLimitExceeded
            );

            Ok(())
        }

        /// 收取费用
        fn charge_fee(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
            if !amount.is_zero() {
                // 销毁费用（或转入国库）
                let _ = T::Currency::withdraw(
                    who,
                    amount,
                    frame_support::traits::WithdrawReasons::FEE,
                    frame_support::traits::ExistenceRequirement::KeepAlive,
                )?;
            }
            Ok(())
        }

        /// 获取日期戳（简化为区块号除以一天的区块数）
        fn get_day_stamp() -> u32 {
            let current_block = <frame_system::Pallet<T>>::block_number();
            // 假设 6 秒一个区块，一天 14400 个区块
            Self::block_to_u32(current_block) / 14400
        }

        /// 区块号转 u32
        fn block_to_u32(block: BlockNumberFor<T>) -> u32 {
            use sp_runtime::traits::UniqueSaturatedInto;
            block.unique_saturated_into()
        }

        /// 根据公历月份计算月将
        ///
        /// 大六壬月将遵循"以中气为准"的原则：
        /// - 雨水后（约2月）用亥将（登明）
        /// - 春分后（约3月）用戌将（河魁）
        /// - 谷雨后（约4月）用酉将（从魁）
        /// - 小满后（约5月）用申将（传送）
        /// - 夏至后（约6月）用未将（小吉）
        /// - 大暑后（约7月）用午将（胜光）
        /// - 处暑后（约8月）用巳将（太乙）
        /// - 秋分后（约9月）用辰将（天罡）
        /// - 霜降后（约10月）用卯将（太冲）
        /// - 小雪后（约11月）用寅将（功曹）
        /// - 冬至后（约12月）用丑将（大吉）
        /// - 大寒后（约1月）用子将（神后）
        ///
        /// # 参数
        /// - `solar_month`: 公历月份 (1-12)
        ///
        /// # 返回
        /// - 月将对应的地支
        fn calc_yue_jiang_from_month(solar_month: u8) -> DiZhi {
            // 简化映射：根据公历月份推算月将
            // 月将与中气对应，这里使用近似值
            match solar_month {
                1 => DiZhi::Zi,    // 大寒后 - 神后
                2 => DiZhi::Hai,   // 雨水后 - 登明
                3 => DiZhi::Xu,    // 春分后 - 河魁
                4 => DiZhi::You,   // 谷雨后 - 从魁
                5 => DiZhi::Shen,  // 小满后 - 传送
                6 => DiZhi::Wei,   // 夏至后 - 小吉
                7 => DiZhi::Wu,    // 大暑后 - 胜光
                8 => DiZhi::Si,    // 处暑后 - 太乙
                9 => DiZhi::Chen,  // 秋分后 - 天罡
                10 => DiZhi::Mao,  // 霜降后 - 太冲
                11 => DiZhi::Yin,  // 小雪后 - 功曹
                12 => DiZhi::Chou, // 冬至后 - 大吉
                _ => DiZhi::Zi,    // 默认
            }
        }
    }

    // ========================================================================
    // 查询函数
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 获取式盘
        pub fn get_pan(
            pan_id: u64,
        ) -> Option<DaLiuRenPan<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>> {
            Pans::<T>::get(pan_id)
        }

        /// 获取用户统计
        pub fn get_user_stats(who: &T::AccountId) -> UserStats {
            UserStatsStorage::<T>::get(who)
        }

        /// 检查式盘是否属于用户
        pub fn is_user_pan(who: &T::AccountId, pan_id: u64) -> bool {
            UserPans::<T>::get(who, pan_id)
        }

        /// 检查式盘是否有待处理的 AI 解读请求
        pub fn has_pending_ai_request(pan_id: u64) -> bool {
            AiInterpretationRequests::<T>::contains_key(pan_id)
        }
    }

    // ========================================================================
    // Runtime API - 解盘查询函数
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 获取核心解盘结果
        ///
        /// 根据式盘ID计算核心解盘指标（约20字节）
        /// 此方法为免费查询，不消耗gas
        ///
        /// # 参数
        /// - `pan_id`: 式盘ID
        ///
        /// # 返回
        /// - `Option<CoreInterpretation>`: 核心解盘结果
        pub fn get_core_interpretation(pan_id: u64) -> Option<CoreInterpretation> {
            let pan = Pans::<T>::get(pan_id)?;
            let current_block = Self::block_to_u32(<frame_system::Pallet<T>>::block_number());

            Some(calculate_core_interpretation(&pan, current_block))
        }

        /// 获取完整解盘结果
        ///
        /// 根据式盘ID计算完整解盘数据，包括：
        /// - 核心解盘指标
        /// - 三传分析
        /// - 四课分析
        /// - 天将分析
        /// - 神煞分析
        /// - 应期分析
        ///
        /// # 参数
        /// - `pan_id`: 式盘ID
        /// - `shi_xiang_type`: 占问类型（可选）
        ///
        /// # 返回
        /// - `Option<FullInterpretation>`: 完整解盘结果
        pub fn get_full_interpretation(
            pan_id: u64,
            shi_xiang_type: Option<ShiXiangType>,
        ) -> Option<FullInterpretation> {
            let pan = Pans::<T>::get(pan_id)?;
            let current_block = Self::block_to_u32(<frame_system::Pallet<T>>::block_number());

            Some(calculate_full_interpretation(&pan, current_block, shi_xiang_type))
        }

        /// 获取三传分析
        ///
        /// 分析三传的旺衰、空亡、递生递克关系
        /// 注意：Private 模式无法进行解读
        ///
        /// # 参数
        /// - `pan_id`: 式盘ID
        ///
        /// # 返回
        /// - `Option<SanChuanAnalysis>`: 三传分析结果
        pub fn get_san_chuan_analysis(pan_id: u64) -> Option<SanChuanAnalysis> {
            let pan = Pans::<T>::get(pan_id)?;

            // 检查是否可解读（Private 模式无计算数据）
            if !pan.can_interpret() {
                return None;
            }

            let month_gz = pan.month_gz?;
            let san_chuan = pan.san_chuan.as_ref()?;
            let tian_jiang_pan = pan.tian_jiang_pan.as_ref()?;
            let day_gz = pan.day_gz?;
            let xun_kong = pan.xun_kong?;

            Some(analyze_san_chuan(
                san_chuan,
                tian_jiang_pan,
                day_gz.0,
                month_gz.1.wu_xing(),
                xun_kong,
            ))
        }

        /// 获取应期分析
        ///
        /// 计算多种应期：三传相加法、空亡填实、六冲应期等
        /// 注意：Private 模式无法进行解读
        ///
        /// # 参数
        /// - `pan_id`: 式盘ID
        /// - `shi_xiang_type`: 占问类型（可选）
        ///
        /// # 返回
        /// - `Option<YingQiAnalysis>`: 应期分析结果
        pub fn get_ying_qi_analysis(
            pan_id: u64,
            shi_xiang_type: Option<ShiXiangType>,
        ) -> Option<YingQiAnalysis> {
            let pan = Pans::<T>::get(pan_id)?;

            // 检查是否可解读（Private 模式无计算数据）
            if !pan.can_interpret() {
                return None;
            }

            let san_chuan = pan.san_chuan.as_ref()?;
            let xun_kong = pan.xun_kong?;

            Some(calculate_ying_qi_analysis(
                san_chuan,
                xun_kong,
                shi_xiang_type,
            ))
        }

        /// 批量获取用户式盘的解盘摘要
        ///
        /// 获取用户最近的式盘解盘摘要（核心指标）
        ///
        /// # 参数
        /// - `who`: 用户账户
        /// - `limit`: 返回数量限制
        ///
        /// # 返回
        /// - `Vec<(u64, CoreInterpretation)>`: (式盘ID, 核心解盘)列表
        #[cfg(feature = "std")]
        pub fn get_user_interpretations_summary(
            who: &T::AccountId,
            limit: u32,
        ) -> Vec<(u64, CoreInterpretation)> {
            let mut results = Vec::new();
            let current_block = Self::block_to_u32(<frame_system::Pallet<T>>::block_number());
            let mut count = 0u32;

            // 遍历用户式盘
            for (pan_id, exists) in UserPans::<T>::iter_prefix(who) {
                if !exists || count >= limit {
                    break;
                }

                if let Some(pan) = Pans::<T>::get(pan_id) {
                    let interpretation = calculate_core_interpretation(&pan, current_block);
                    results.push((pan_id, interpretation));
                    count += 1;
                }
            }

            results
        }

        /// 根据吉凶等级筛选式盘
        ///
        /// 获取指定吉凶等级的式盘列表
        ///
        /// # 参数
        /// - `fortune_level`: 吉凶等级
        /// - `limit`: 返回数量限制
        ///
        /// # 返回
        /// - `Vec<u64>`: 符合条件的式盘ID列表
        #[cfg(feature = "std")]
        pub fn get_pans_by_fortune(
            fortune_level: FortuneLevel,
            limit: u32,
        ) -> Vec<u64> {
            let mut results = Vec::new();
            let current_block = Self::block_to_u32(<frame_system::Pallet<T>>::block_number());
            let mut count = 0u32;

            // 遍历公开式盘
            for (pan_id, _) in PublicPans::<T>::iter() {
                if count >= limit {
                    break;
                }

                if let Some(pan) = Pans::<T>::get(pan_id) {
                    let interpretation = calculate_core_interpretation(&pan, current_block);
                    if interpretation.fortune == fortune_level {
                        results.push(pan_id);
                        count += 1;
                    }
                }
            }

            results
        }
    }

}
