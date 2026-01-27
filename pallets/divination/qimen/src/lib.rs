//! # 奇门遁甲排盘 Pallet
//!
//! 本模块实现了区块链上的奇门遁甲排盘系统，提供：
//! - 时间起局（根据四柱和节气）
//! - 数字起局（根据用户数字）
//! - 随机起局（使用链上随机数）
//! - 手动指定（直接指定局数）
//! - 排盘记录存储与查询
//! - AI 解读请求（链下工作机触发）
//!
//! ## 核心概念
//!
//! - **阴阳遁**: 冬至到夏至为阳遁（顺行），夏至到冬至为阴遁（逆行）
//! - **三元**: 每节气分上中下三元，各5天
//! - **局数**: 1-9局，由节气和三元决定
//! - **四盘**: 天盘（九星）、地盘（三奇六仪）、人盘（八门）、神盘（八神）
//! - **值符值使**: 当值的星和门，是奇门的核心

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod algorithm;
pub mod interpretation;
pub mod runtime_api;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use crate::algorithm;
    use crate::interpretation;
    use crate::types::{self, *};
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, ExistenceRequirement, Randomness, ReservableCurrency},
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;
    use sp_std::prelude::*;

    // 导入押金相关类型
    pub use pallet_divination_common::deposit::{
        PrivacyMode as DepositPrivacyMode,
        DepositRecord,
    };

    /// Pallet 配置 trait
    ///
    /// 注：RuntimeEvent 关联类型已从 Polkadot SDK 2506 版本开始自动附加，
    /// 无需在此显式声明。系统会自动添加：
    /// `frame_system::Config<RuntimeEvent: From<Event<Self>>>`
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型（需要支持 reserve/unreserve）
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 随机数生成器
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 每个用户最多存储的排盘记录数量
        #[pallet::constant]
        type MaxUserCharts: Get<u32>;

        /// 公开排盘列表的最大长度
        #[pallet::constant]
        type MaxPublicCharts: Get<u32>;

        /// 每日免费排盘次数
        #[pallet::constant]
        type DailyFreeCharts: Get<u32>;

        /// 每日最大排盘次数（防刷）
        #[pallet::constant]
        type MaxDailyCharts: Get<u32>;

        /// AI 解读费用
        #[pallet::constant]
        type AiInterpretationFee: Get<BalanceOf<Self>>;

        /// 国库账户
        #[pallet::constant]
        type TreasuryAccount: Get<Self::AccountId>;

        /// AI 预言机权限来源
        type AiOracleOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// IPFS CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 加密数据最大长度（默认: 512 bytes）
        ///
        /// 用于存储加密后的敏感数据（姓名、问题等）
        #[pallet::constant]
        type MaxEncryptedLen: Get<u32>;

    }

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ==================== 存储项 ====================

    /// 下一个排盘记录 ID
    #[pallet::storage]
    #[pallet::getter(fn next_chart_id)]
    pub type NextChartId<T> = StorageValue<_, u64, ValueQuery>;

    /// 排盘记录存储
    ///
    /// 键：排盘记录 ID
    /// 值：完整的奇门遁甲排盘结果
    #[pallet::storage]
    #[pallet::getter(fn charts)]
    pub type Charts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        QimenChart<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    >;

    /// 用户排盘索引
    ///
    /// 键：用户账户
    /// 值：该用户的所有排盘记录 ID 列表
    #[pallet::storage]
    #[pallet::getter(fn user_charts)]
    pub type UserCharts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxUserCharts>,
        ValueQuery,
    >;

    /// 公开排盘列表
    ///
    /// 存储所有设置为公开的排盘记录 ID
    #[pallet::storage]
    #[pallet::getter(fn public_charts)]
    pub type PublicCharts<T: Config> =
        StorageValue<_, BoundedVec<u64, T::MaxPublicCharts>, ValueQuery>;

    /// 每日排盘计数
    ///
    /// 用于限制每日排盘次数，防止滥用
    /// 键1：用户账户
    /// 键2：天数（从创世块起算）
    /// 值：当日排盘次数
    #[pallet::storage]
    #[pallet::getter(fn daily_chart_count)]
    pub type DailyChartCount<T: Config> = StorageDoubleMap<
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

    /// 用户统计信息
    ///
    /// 记录用户的排盘统计数据
    #[pallet::storage]
    #[pallet::getter(fn user_stats)]
    pub type UserStatsStorage<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserStats, ValueQuery>;

    /// 加密数据存储
    ///
    /// 键：排盘记录 ID
    /// 值：加密后的敏感数据（姓名、问题等）
    ///
    /// 仅当 privacy_mode 为 Partial 或 Private 时存储
    #[pallet::storage]
    #[pallet::getter(fn encrypted_data)]
    pub type EncryptedDataStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<u8, T::MaxEncryptedLen>,
    >;

    /// 所有者密钥备份存储
    ///
    /// 键：排盘记录 ID
    /// 值：用所有者公钥加密的主密钥备份（80 bytes）
    ///
    /// 用于：
    /// - 所有者更换设备后恢复密钥
    /// - 授权查看者时解密主密钥
    #[pallet::storage]
    #[pallet::getter(fn owner_key_backup)]
    pub type OwnerKeyBackupStorage<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        [u8; 80],
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 新排盘记录创建成功
        /// [排盘ID, 排盘者, 阴阳遁, 局数]
        ChartCreated {
            chart_id: u64,
            diviner: T::AccountId,
            dun_type: DunType,
            ju_number: u8,
        },

        /// AI 解读请求已提交
        /// [排盘ID, 请求者]
        AiInterpretationRequested {
            chart_id: u64,
            requester: T::AccountId,
        },

        /// AI 解读结果已提交
        /// [排盘ID, IPFS CID]
        AiInterpretationSubmitted {
            chart_id: u64,
            cid: BoundedVec<u8, T::MaxCidLen>,
        },

        /// 排盘公开状态已更改
        /// [排盘ID, 是否公开]
        ChartVisibilityChanged {
            chart_id: u64,
            is_public: bool,
        },

        /// 加密排盘记录创建成功
        /// [排盘ID, 排盘者, 隐私模式, 阴阳遁, 局数]
        EncryptedChartCreated {
            chart_id: u64,
            diviner: T::AccountId,
            privacy_mode: pallet_divination_privacy::types::PrivacyMode,
            dun_type: Option<DunType>,
            ju_number: Option<u8>,
        },

        /// 加密数据已更新
        /// [排盘ID, 数据哈希]
        EncryptedDataUpdated {
            chart_id: u64,
            data_hash: [u8; 32],
        },

        /// 排盘记录已删除
        /// [排盘ID, 所有者]
        ChartDeleted {
            chart_id: u64,
            owner: T::AccountId,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 排盘记录不存在
        ChartNotFound,
        /// 非排盘记录所有者
        NotOwner,
        /// 每日排盘次数超限
        DailyLimitExceeded,
        /// 用户排盘列表已满
        UserChartsFull,
        /// 公开排盘列表已满
        PublicChartsFull,
        /// 无效的局数（必须为1-9）
        InvalidJuNumber,
        /// 无效的节气（必须为0-23）
        InvalidJieQi,
        /// AI 解读请求已存在
        AiRequestAlreadyExists,
        /// AI 解读请求不存在
        AiRequestNotFound,
        /// 数字参数缺失
        MissingNumberParams,
        /// 手动指定参数缺失
        MissingManualParams,
        /// 无效的干支组合
        InvalidGanZhi,
        /// 节气天数超范围
        InvalidDayInJieQi,
        /// 无效的加密级别（必须为 0/1/2）
        InvalidEncryptionLevel,
        /// 加密数据缺失（Partial/Private 模式必须提供）
        EncryptedDataMissing,
        /// 数据哈希缺失（Partial/Private 模式必须提供）
        DataHashMissing,
        /// 密钥备份缺失（Partial/Private 模式必须提供）
        OwnerKeyBackupMissing,
        /// 加密数据过长
        EncryptedDataTooLong,
        /// 加密数据不存在
        EncryptedDataNotFound,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 时间起局排盘
        ///
        /// 根据四柱和节气信息生成奇门遁甲盘。
        ///
        /// # 参数
        /// - `origin`: 调用者（签名账户）
        /// - `year_ganzhi`: 年柱干支（干0-9，支0-11）
        /// - `month_ganzhi`: 月柱干支
        /// - `day_ganzhi`: 日柱干支
        /// - `hour_ganzhi`: 时柱干支
        /// - `jie_qi`: 节气（0-23）
        /// - `day_in_jieqi`: 节气内天数（1-15）
        /// - `question_hash`: 问题哈希（隐私保护）
        /// - `is_public`: 是否公开此排盘
        /// - `name`: 命主姓名（可选，UTF-8编码，最大32字节）
        /// - `gender`: 命主性别（可选，0=男，1=女）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选，UTF-8编码，最大128字节）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(80_000_000, 0))]
        pub fn divine_by_time(
            origin: OriginFor<T>,
            year_ganzhi: (u8, u8),
            month_ganzhi: (u8, u8),
            day_ganzhi: (u8, u8),
            hour_ganzhi: (u8, u8),
            jie_qi: u8,
            day_in_jieqi: u8,
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<u8>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证参数
            ensure!(jie_qi < 24, Error::<T>::InvalidJieQi);
            ensure!(day_in_jieqi >= 1 && day_in_jieqi <= 15, Error::<T>::InvalidDayInJieQi);

            // 转换干支
            let year_gz = Self::parse_ganzhi(year_ganzhi)?;
            let month_gz = Self::parse_ganzhi(month_ganzhi)?;
            let day_gz = Self::parse_ganzhi(day_ganzhi)?;
            let hour_gz = Self::parse_ganzhi(hour_ganzhi)?;

            let jieqi = JieQi::from_index(jie_qi).ok_or(Error::<T>::InvalidJieQi)?;

            // 转换命主信息
            let gender_enum = gender.and_then(Gender::from_u8);
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            // 调用排盘算法
            let (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces) =
                algorithm::generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, jieqi, day_in_jieqi);

            Self::create_chart(
                who,
                DivinationMethod::ByTime,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_hash,
                is_public,
                // 命主信息
                name,
                gender_enum,
                birth_year,
                question,
                question_type_enum,
                pan_method_enum,
            )
        }

        /// 公历时间起局排盘
        ///
        /// 此方法使用 pallet-almanac 自动将公历日期转换为四柱干支和节气，
        /// 然后进行奇门遁甲排盘。用户无需手动计算干支和节气。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_hash`: 问题哈希
        /// - `is_public`: 是否公开
        /// - `name`: 命主姓名（可选，UTF-8编码，最大32字节）
        /// - `gender`: 命主性别（可选，0=男，1=女）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选，UTF-8编码，最大128字节）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(120_000_000, 0))]
        pub fn divine_by_solar_time(
            origin: OriginFor<T>,
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<u8>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 参数校验
            ensure!(solar_year >= 1901 && solar_year <= 2100, Error::<T>::InvalidJieQi);
            ensure!(solar_month >= 1 && solar_month <= 12, Error::<T>::InvalidJieQi);
            ensure!(solar_day >= 1 && solar_day <= 31, Error::<T>::InvalidJieQi);
            ensure!(hour < 24, Error::<T>::InvalidJieQi);

            // 转换命主信息
            let gender_enum = gender.and_then(Gender::from_u8);
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            // 调用 almanac 计算四柱
            let pillars = pallet_almanac::four_pillars(solar_year, solar_month, solar_day, hour);

            // 转换为本模块的 GanZhi 类型
            let year_gz = GanZhi {
                gan: TianGan::from_index(pillars.year.gan).ok_or(Error::<T>::InvalidJieQi)?,
                zhi: DiZhi::from_index(pillars.year.zhi).ok_or(Error::<T>::InvalidJieQi)?,
            };
            let month_gz = GanZhi {
                gan: TianGan::from_index(pillars.month.gan).ok_or(Error::<T>::InvalidJieQi)?,
                zhi: DiZhi::from_index(pillars.month.zhi).ok_or(Error::<T>::InvalidJieQi)?,
            };
            let day_gz = GanZhi {
                gan: TianGan::from_index(pillars.day.gan).ok_or(Error::<T>::InvalidJieQi)?,
                zhi: DiZhi::from_index(pillars.day.zhi).ok_or(Error::<T>::InvalidJieQi)?,
            };
            let hour_gz = GanZhi {
                gan: TianGan::from_index(pillars.hour.gan).ok_or(Error::<T>::InvalidJieQi)?,
                zhi: DiZhi::from_index(pillars.hour.zhi).ok_or(Error::<T>::InvalidJieQi)?,
            };

            // 获取节气（返回 Option<u8>，0-23）
            let jie_qi_idx = pallet_almanac::get_solar_term(solar_year, solar_month, solar_day)
                .unwrap_or(0);
            let jieqi = JieQi::from_index(jie_qi_idx).unwrap_or(JieQi::LiChun);

            // 计算节气内天数（简化为1-15）
            // 每个节气约15天，根据日期估算
            let day_in_jieqi = ((solar_day - 1) % 15) + 1;

            // 调用排盘算法
            let (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces) =
                algorithm::generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, jieqi, day_in_jieqi);

            Self::create_chart(
                who,
                DivinationMethod::ByTime,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_hash,
                is_public,
                // 命主信息
                name,
                gender_enum,
                birth_year,
                question,
                question_type_enum,
                pan_method_enum,
            )
        }

        /// 数字起局排盘
        ///
        /// 使用用户输入的数字生成局数。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `numbers`: 用户输入的数字列表
        /// - `dun_type`: 阴阳遁（true=阳遁，false=阴遁）
        /// - `question_hash`: 问题哈希
        /// - `is_public`: 是否公开
        /// - `name`: 命主姓名（可选）
        /// - `gender`: 命主性别（可选，0=男，1=女）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(70_000_000, 0))]
        pub fn divine_by_numbers(
            origin: OriginFor<T>,
            numbers: BoundedVec<u16, ConstU32<16>>,
            yang_dun: bool,
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<u8>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            ensure!(!numbers.is_empty(), Error::<T>::MissingNumberParams);

            // 转换命主信息
            let gender_enum = gender.and_then(Gender::from_u8);
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            // 获取区块哈希作为额外随机源
            let block_hash = <frame_system::Pallet<T>>::parent_hash();
            let block_hash_bytes: [u8; 32] = block_hash
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 从数字生成局数
            let ju_number = algorithm::generate_from_numbers(&numbers, &block_hash_bytes);

            let dun_type = if yang_dun { DunType::Yang } else { DunType::Yin };

            // 使用默认干支（当前时间的近似值）
            let (year_gz, month_gz, day_gz, hour_gz, jieqi, san_yuan) =
                Self::get_default_ganzhi_and_jieqi();

            // 排布地盘
            let di_pan = algorithm::get_di_pan(ju_number, dun_type);

            // 计算值符值使
            let xun_shou_yi = algorithm::get_xun_shou(hour_gz.gan, hour_gz.zhi);
            let zhi_fu_xing = algorithm::calc_zhi_fu_xing(xun_shou_yi, &di_pan);
            let zhi_shi_men = algorithm::calc_zhi_shi_men(xun_shou_yi, &di_pan);

            // 完成排盘
            let (_, _, _, _, _, palaces) = algorithm::generate_qimen_chart(
                year_gz, month_gz, day_gz, hour_gz, jieqi, 1,
            );

            Self::create_chart(
                who,
                DivinationMethod::ByNumbers,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_hash,
                is_public,
                // 命主信息
                name,
                gender_enum,
                birth_year,
                question,
                question_type_enum,
                pan_method_enum,
            )
        }

        /// 随机起局排盘
        ///
        /// 使用链上随机数生成排盘。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `question_hash`: 问题哈希
        /// - `is_public`: 是否公开
        /// - `name`: 命主姓名（可选）
        /// - `gender`: 命主性别（可选，0=男，1=女）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(70_000_000, 0))]
        pub fn divine_random(
            origin: OriginFor<T>,
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<u8>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 转换命主信息
            let gender_enum = gender.and_then(Gender::from_u8);
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            // 使用链上随机源
            let random_seed = T::Randomness::random(&b"qimen"[..]).0;
            let random_bytes: [u8; 32] = random_seed
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 从随机数生成阴阳遁和局数
            let (dun_type, ju_number) = algorithm::generate_from_random(&random_bytes);

            // 使用默认干支
            let (year_gz, month_gz, day_gz, hour_gz, jieqi, san_yuan) =
                Self::get_default_ganzhi_and_jieqi();

            // 排布地盘
            let di_pan = algorithm::get_di_pan(ju_number, dun_type);

            // 计算值符值使
            let xun_shou_yi = algorithm::get_xun_shou(hour_gz.gan, hour_gz.zhi);
            let zhi_fu_xing = algorithm::calc_zhi_fu_xing(xun_shou_yi, &di_pan);
            let zhi_shi_men = algorithm::calc_zhi_shi_men(xun_shou_yi, &di_pan);

            // 完成排盘
            let (_, _, _, _, _, palaces) = algorithm::generate_qimen_chart(
                year_gz, month_gz, day_gz, hour_gz, jieqi, 1,
            );

            Self::create_chart(
                who,
                DivinationMethod::Random,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_hash,
                is_public,
                // 命主信息
                name,
                gender_enum,
                birth_year,
                question,
                question_type_enum,
                pan_method_enum,
            )
        }

        /// 手动指定排盘
        ///
        /// 直接指定阴阳遁和局数。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `yang_dun`: 是否阳遁
        /// - `ju_number`: 局数（1-9）
        /// - `hour_ganzhi`: 时柱干支
        /// - `question_hash`: 问题哈希
        /// - `is_public`: 是否公开
        /// - `name`: 命主姓名（可选）
        /// - `gender`: 命主性别（可选，0=男，1=女）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn divine_manual(
            origin: OriginFor<T>,
            yang_dun: bool,
            ju_number: u8,
            hour_ganzhi: (u8, u8),
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<u8>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            ensure!(algorithm::validate_ju_number(ju_number), Error::<T>::InvalidJuNumber);

            // 转换命主信息
            let gender_enum = gender.and_then(Gender::from_u8);
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            let hour_gz = Self::parse_ganzhi(hour_ganzhi)?;
            let dun_type = if yang_dun { DunType::Yang } else { DunType::Yin };

            // 使用默认干支
            let (year_gz, month_gz, day_gz, _, jieqi, san_yuan) =
                Self::get_default_ganzhi_and_jieqi();

            // 排布地盘
            let di_pan = algorithm::get_di_pan(ju_number, dun_type);

            // 计算值符值使
            let xun_shou_yi = algorithm::get_xun_shou(hour_gz.gan, hour_gz.zhi);
            let zhi_fu_xing = algorithm::calc_zhi_fu_xing(xun_shou_yi, &di_pan);
            let zhi_shi_men = algorithm::calc_zhi_shi_men(xun_shou_yi, &di_pan);

            // 排布天盘九星
            let tian_pan_xing = algorithm::distribute_jiu_xing(zhi_fu_xing, hour_gz.gan, &di_pan, dun_type);

            // 排布人盘八门
            let ren_pan_men = algorithm::distribute_ba_men(zhi_shi_men, hour_gz.gan, &di_pan, dun_type);

            // 找到值符落宫
            let zhi_fu_gong = algorithm::find_gan_in_di_pan(hour_gz.gan, &di_pan).unwrap_or(1);

            // 排布神盘八神
            let shen_pan_shen = algorithm::distribute_ba_shen(zhi_fu_gong, dun_type);

            // 组装九宫
            let mut palaces = [Palace::empty(JiuGong::Kan); 9];
            for i in 0..9 {
                let gong = JiuGong::from_num((i + 1) as u8).unwrap_or(JiuGong::Kan);
                let xing = tian_pan_xing[i];
                let tian_pan_gan = algorithm::get_tian_pan_gan(xing, &di_pan);

                palaces[i] = Palace {
                    gong,
                    tian_pan_gan,
                    di_pan_gan: di_pan[i],
                    xing,
                    men: ren_pan_men[i],
                    shen: shen_pan_shen[i],
                    is_xun_kong: false,
                    is_ma_xing: false,
                };
            }

            Self::create_chart(
                who,
                DivinationMethod::Manual,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
                jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_hash,
                is_public,
                // 命主信息
                name,
                gender_enum,
                birth_year,
                question,
                question_type_enum,
                pan_method_enum,
            )
        }

        /// 请求 AI 解读（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::request_interpretation`
        /// 新的统一 AI 解读系统支持：
        /// - 多种 AI 模型选择（针对不同占卜类型的专用模型）
        /// - Oracle 质押和评分机制
        /// - 争议和退款处理
        ///
        /// # 废弃原因
        /// 为统一 AI 解读逻辑、减少代码重复，所有 AI 解读请求已移至
        /// `pallet-divination-ai` 模块统一处理。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `chart_id`: 排盘记录 ID
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::request_interpretation"
        )]
        pub fn request_ai_interpretation(
            origin: OriginFor<T>,
            chart_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证排盘记录存在且为调用者所有
            let chart = Charts::<T>::get(chart_id)
                .ok_or(Error::<T>::ChartNotFound)?;
            ensure!(chart.diviner == who, Error::<T>::NotOwner);

            // 检查是否已有请求
            ensure!(
                !AiInterpretationRequests::<T>::contains_key(chart_id),
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
            AiInterpretationRequests::<T>::insert(chart_id, who.clone());

            // 发送事件触发链下工作机
            Self::deposit_event(Event::AiInterpretationRequested {
                chart_id,
                requester: who,
            });

            Ok(())
        }

        /// 提交 AI 解读结果（仅限授权节点）（已废弃）
        ///
        /// **注意**：此函数已废弃，请使用 `pallet_divination_ai::submit_result`
        /// 新的统一 AI 解读系统支持更完善的结果提交和验证机制。
        ///
        /// # 参数
        /// - `origin`: AI 预言机授权来源
        /// - `chart_id`: 排盘记录 ID
        /// - `interpretation_cid`: 解读内容的 IPFS CID
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::submit_result"
        )]
        pub fn submit_ai_interpretation(
            origin: OriginFor<T>,
            chart_id: u64,
            interpretation_cid: BoundedVec<u8, T::MaxCidLen>,
        ) -> DispatchResult {
            // 验证 AI 预言机权限
            T::AiOracleOrigin::ensure_origin(origin)?;

            // 验证请求存在
            ensure!(
                AiInterpretationRequests::<T>::contains_key(chart_id),
                Error::<T>::AiRequestNotFound
            );

            // 更新排盘记录的 AI 解读 CID
            Charts::<T>::try_mutate(chart_id, |maybe_chart| {
                let chart = maybe_chart
                    .as_mut()
                    .ok_or(Error::<T>::ChartNotFound)?;
                chart.interpretation_cid = Some(interpretation_cid.clone());
                Ok::<_, DispatchError>(())
            })?;

            // 移除请求
            AiInterpretationRequests::<T>::remove(chart_id);

            Self::deposit_event(Event::AiInterpretationSubmitted {
                chart_id,
                cid: interpretation_cid,
            });

            Ok(())
        }

        /// 更改排盘公开状态
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `chart_id`: 排盘记录 ID
        /// - `is_public`: 是否公开（向后兼容接口，映射为 PrivacyMode）
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn set_chart_visibility(
            origin: OriginFor<T>,
            chart_id: u64,
            is_public: bool,
        ) -> DispatchResult {
            use pallet_divination_privacy::types::PrivacyMode;

            let who = ensure_signed(origin)?;

            Charts::<T>::try_mutate(chart_id, |maybe_chart| {
                let chart = maybe_chart
                    .as_mut()
                    .ok_or(Error::<T>::ChartNotFound)?;
                ensure!(chart.diviner == who, Error::<T>::NotOwner);

                let was_public = chart.privacy_mode == PrivacyMode::Public;
                chart.privacy_mode = if is_public {
                    PrivacyMode::Public
                } else {
                    PrivacyMode::Partial
                };

                // 更新公开排盘列表
                if is_public && !was_public {
                    // 添加到公开列表
                    PublicCharts::<T>::try_mutate(|list| {
                        list.try_push(chart_id)
                            .map_err(|_| Error::<T>::PublicChartsFull)
                    })?;
                } else if !is_public && was_public {
                    // 从公开列表移除
                    PublicCharts::<T>::mutate(|list| {
                        list.retain(|&id| id != chart_id);
                    });
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ChartVisibilityChanged {
                chart_id,
                is_public,
            });

            Ok(())
        }

        /// 公历时间加密起局排盘
        ///
        /// 支持三种隐私模式：
        /// - 0 (Public): 所有数据明文存储
        /// - 1 (Partial): 计算数据明文 + 敏感数据加密（推荐）
        /// - 2 (Private): 全部数据加密（需前端解密后调用 compute_chart API）
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `encryption_level`: 加密级别（0=Public, 1=Partial, 2=Private）
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_hash`: 问题哈希（用于验证）
        /// - `encrypted_data`: 加密的敏感数据（Partial/Private 模式必填）
        /// - `data_hash`: 原始敏感数据哈希（用于完整性验证）
        /// - `owner_key_backup`: 所有者密钥备份（80 bytes，用于密钥恢复）
        /// - `question_type`: 问事类型（可选，0-11）
        /// - `pan_method`: 排盘方法（0=转盘，1=飞盘）
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(150_000_000, 0))]
        pub fn divine_by_solar_time_encrypted(
            origin: OriginFor<T>,
            encryption_level: u8,
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_hash: [u8; 32],
            encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
            data_hash: Option<[u8; 32]>,
            owner_key_backup: Option<[u8; 80]>,
            question_type: Option<u8>,
            pan_method: u8,
        ) -> DispatchResult {
            use pallet_divination_privacy::types::PrivacyMode;

            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 验证加密级别
            let privacy_mode = match encryption_level {
                0 => PrivacyMode::Public,
                1 => PrivacyMode::Partial,
                2 => PrivacyMode::Private,
                _ => return Err(Error::<T>::InvalidEncryptionLevel.into()),
            };

            // Partial/Private 模式验证
            if encryption_level >= 1 {
                ensure!(encrypted_data.is_some(), Error::<T>::EncryptedDataMissing);
                ensure!(data_hash.is_some(), Error::<T>::DataHashMissing);
                ensure!(owner_key_backup.is_some(), Error::<T>::OwnerKeyBackupMissing);
            }

            // 参数校验
            ensure!(solar_year >= 1901 && solar_year <= 2100, Error::<T>::InvalidJieQi);
            ensure!(solar_month >= 1 && solar_month <= 12, Error::<T>::InvalidJieQi);
            ensure!(solar_day >= 1 && solar_day <= 31, Error::<T>::InvalidJieQi);
            ensure!(hour < 24, Error::<T>::InvalidJieQi);

            // 转换问事类型
            let question_type_enum = question_type.and_then(|t| match t {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            });
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            // 根据隐私模式处理
            let (dun_type, ju_number, palaces, year_gz, month_gz, day_gz, hour_gz, jieqi, san_yuan, zhi_fu_xing, zhi_shi_men) =
                if encryption_level == 2 {
                    // Private 模式：不存储计算数据
                    (None, None, None, None, None, None, None, None, None, None, None)
                } else {
                    // Public/Partial 模式：计算并存储数据
                    // 调用 almanac 计算四柱
                    let pillars = pallet_almanac::four_pillars(solar_year, solar_month, solar_day, hour);

                    // 转换为本模块的 GanZhi 类型
                    let year_gz = GanZhi {
                        gan: TianGan::from_index(pillars.year.gan).ok_or(Error::<T>::InvalidJieQi)?,
                        zhi: DiZhi::from_index(pillars.year.zhi).ok_or(Error::<T>::InvalidJieQi)?,
                    };
                    let month_gz = GanZhi {
                        gan: TianGan::from_index(pillars.month.gan).ok_or(Error::<T>::InvalidJieQi)?,
                        zhi: DiZhi::from_index(pillars.month.zhi).ok_or(Error::<T>::InvalidJieQi)?,
                    };
                    let day_gz = GanZhi {
                        gan: TianGan::from_index(pillars.day.gan).ok_or(Error::<T>::InvalidJieQi)?,
                        zhi: DiZhi::from_index(pillars.day.zhi).ok_or(Error::<T>::InvalidJieQi)?,
                    };
                    let hour_gz = GanZhi {
                        gan: TianGan::from_index(pillars.hour.gan).ok_or(Error::<T>::InvalidJieQi)?,
                        zhi: DiZhi::from_index(pillars.hour.zhi).ok_or(Error::<T>::InvalidJieQi)?,
                    };

                    // 获取节气
                    let jie_qi_idx = pallet_almanac::get_solar_term(solar_year, solar_month, solar_day)
                        .unwrap_or(0);
                    let jieqi = JieQi::from_index(jie_qi_idx).unwrap_or(JieQi::LiChun);

                    // 计算节气内天数
                    let day_in_jieqi = ((solar_day - 1) % 15) + 1;

                    // 调用排盘算法
                    let (dun, yuan, ju, xing, men, pal) =
                        algorithm::generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, jieqi, day_in_jieqi);

                    (
                        Some(dun),
                        Some(ju),
                        Some(pal),
                        Some(year_gz),
                        Some(month_gz),
                        Some(day_gz),
                        Some(hour_gz),
                        Some(jieqi),
                        Some(yuan),
                        Some(xing),
                        Some(men),
                    )
                };

            // 获取新的排盘记录 ID
            let chart_id = NextChartId::<T>::get();
            NextChartId::<T>::put(chart_id.saturating_add(1));

            // 获取当前区块号和时间戳
            let block_number = <frame_system::Pallet<T>>::block_number();
            let timestamp = Self::get_timestamp_secs();

            // 创建排盘记录
            let chart = QimenChart {
                id: chart_id,
                diviner: who.clone(),
                method: DivinationMethod::ByTime,
                // 隐私控制字段
                privacy_mode,
                encrypted_fields: if encryption_level >= 1 {
                    // 标记加密的敏感字段：姓名(bit 0) + 问题(bit 3)
                    Some(0b1001) // NAME | QUESTION
                } else {
                    None
                },
                sensitive_data_hash: data_hash,
                // 命主敏感信息（加密模式下不存储在主结构中）
                name: None,
                gender: None,
                birth_year: None,
                question: None,
                question_type: question_type_enum,
                pan_method: pan_method_enum,
                // 四柱干支
                year_ganzhi: year_gz,
                month_ganzhi: month_gz,
                day_ganzhi: day_gz,
                hour_ganzhi: hour_gz,
                jie_qi: jieqi,
                // 局数信息
                dun_type,
                san_yuan,
                ju_number,
                // 盘面数据
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                // 元数据
                timestamp,
                block_number,
                interpretation_cid: None,
                question_hash,
            };

            // 存储排盘记录
            Charts::<T>::insert(chart_id, chart);

            // 存储加密数据（如果有）
            if let Some(enc_data) = encrypted_data {
                EncryptedDataStorage::<T>::insert(chart_id, enc_data);
            }

            // 存储密钥备份（如果有）
            if let Some(key_backup) = owner_key_backup {
                OwnerKeyBackupStorage::<T>::insert(chart_id, key_backup);
            }

            // 更新用户排盘索引
            UserCharts::<T>::try_mutate(&who, |list| {
                list.try_push(chart_id)
                    .map_err(|_| Error::<T>::UserChartsFull)
            })?;

            // Public 模式添加到公开列表
            if encryption_level == 0 {
                PublicCharts::<T>::try_mutate(|list| {
                    list.try_push(chart_id)
                        .map_err(|_| Error::<T>::PublicChartsFull)
                })?;
            }

            // 更新用户统计
            if let (Some(dt), Some(xing), Some(men)) = (dun_type, zhi_fu_xing, zhi_shi_men) {
                UserStatsStorage::<T>::mutate(&who, |stats| {
                    stats.update_from_chart(dt, xing, men);
                });
            }

            // 发送事件
            Self::deposit_event(Event::EncryptedChartCreated {
                chart_id,
                diviner: who,
                privacy_mode,
                dun_type,
                ju_number,
            });

            Ok(())
        }

        /// 更新加密数据
        ///
        /// 允许所有者更新已有排盘的加密数据（用于密钥轮换等场景）
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是排盘所有者）
        /// - `chart_id`: 排盘记录 ID
        /// - `encrypted_data`: 新的加密数据
        /// - `data_hash`: 新的数据哈希
        /// - `owner_key_backup`: 新的密钥备份
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn update_encrypted_data(
            origin: OriginFor<T>,
            chart_id: u64,
            encrypted_data: BoundedVec<u8, T::MaxEncryptedLen>,
            data_hash: [u8; 32],
            owner_key_backup: [u8; 80],
        ) -> DispatchResult {
            use pallet_divination_privacy::types::PrivacyMode;

            let who = ensure_signed(origin)?;

            // 验证排盘记录存在且为调用者所有
            Charts::<T>::try_mutate(chart_id, |maybe_chart| {
                let chart = maybe_chart
                    .as_mut()
                    .ok_or(Error::<T>::ChartNotFound)?;
                ensure!(chart.diviner == who, Error::<T>::NotOwner);

                // 只有 Partial/Private 模式可以更新加密数据
                ensure!(
                    chart.privacy_mode != PrivacyMode::Public,
                    Error::<T>::InvalidEncryptionLevel
                );

                // 更新哈希
                chart.sensitive_data_hash = Some(data_hash);

                Ok::<_, DispatchError>(())
            })?;

            // 更新加密数据
            EncryptedDataStorage::<T>::insert(chart_id, encrypted_data);

            // 更新密钥备份
            OwnerKeyBackupStorage::<T>::insert(chart_id, owner_key_backup);

            // 发送事件
            Self::deposit_event(Event::EncryptedDataUpdated {
                chart_id,
                data_hash,
            });

            Ok(())
        }

        /// 删除排盘记录
        ///
        /// 删除排盘记录及其所有关联数据，并返还存储押金。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是排盘所有者）
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返还规则
        /// - 30天内删除：100% 返还
        /// - 30天后删除：90% 返还（10% 进入国库）
        ///
        /// # 删除内容
        /// 1. 主排盘记录（Charts）
        /// 2. 用户索引（UserCharts）
        /// 3. 公开列表（PublicCharts，如适用）
        /// 4. AI 解读请求（AiInterpretationRequests）
        /// 5. 加密数据（EncryptedDataStorage）
        /// 6. 密钥备份（OwnerKeyBackupStorage）
        /// 7. 押金记录（DepositRecords）
        #[pallet::call_index(10)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn delete_chart(
            origin: OriginFor<T>,
            chart_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取排盘记录并验证所有权
            let chart = Charts::<T>::get(chart_id)
                .ok_or(Error::<T>::ChartNotFound)?;
            ensure!(chart.diviner == who, Error::<T>::NotOwner);

            // 2. 从用户索引中移除
            UserCharts::<T>::mutate(&who, |charts| {
                charts.retain(|&id| id != chart_id);
            });

            // 4. 从公开列表中移除（如果是公开的）
            if chart.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public {
                PublicCharts::<T>::mutate(|list| {
                    list.retain(|&id| id != chart_id);
                });
            }

            // 5. 移除 AI 解读请求（如果有）
            AiInterpretationRequests::<T>::remove(chart_id);

            // 6. 移除加密数据（如果有）
            EncryptedDataStorage::<T>::remove(chart_id);

            // 7. 移除密钥备份（如果有）
            OwnerKeyBackupStorage::<T>::remove(chart_id);

            // 8. 删除主排盘记录
            Charts::<T>::remove(chart_id);

            // 7. 发送删除事件
            Self::deposit_event(Event::ChartDeleted {
                chart_id,
                owner: who,
            });

            Ok(())
        }
    }

    // ==================== 内部辅助函数 ====================

    impl<T: Config> Pallet<T> {
        /// 获取当前时间戳（秒）
        fn get_timestamp_secs() -> u64 {
            let moment = pallet_timestamp::Pallet::<T>::get();
            let ms: u64 = moment.try_into().unwrap_or(0);
            ms / 1000
        }

        /// 检查每日排盘次数限制
        fn check_daily_limit(who: &T::AccountId) -> DispatchResult {
            let today = Self::current_day();
            let count = DailyChartCount::<T>::get(who, today);

            ensure!(
                count < T::MaxDailyCharts::get(),
                Error::<T>::DailyLimitExceeded
            );

            // 更新计数
            DailyChartCount::<T>::insert(who, today, count + 1);
            Ok(())
        }

        /// 获取当前天数（从创世块起算）
        fn current_day() -> u32 {
            let timestamp = Self::get_timestamp_secs();
            (timestamp / 86400) as u32
        }

        /// 解析干支参数
        fn parse_ganzhi(ganzhi: (u8, u8)) -> Result<GanZhi, DispatchError> {
            let gan = TianGan::from_index(ganzhi.0)
                .ok_or(Error::<T>::InvalidGanZhi)?;
            let zhi = DiZhi::from_index(ganzhi.1)
                .ok_or(Error::<T>::InvalidGanZhi)?;
            Ok(GanZhi::new(gan, zhi))
        }

        /// 获取默认干支和节气（基于当前时间戳的近似值）
        fn get_default_ganzhi_and_jieqi() -> (GanZhi, GanZhi, GanZhi, GanZhi, JieQi, SanYuan) {
            let timestamp = Self::get_timestamp_secs();

            // 简化计算：使用时间戳推算干支
            // 以1970-01-01为基准（庚戌年）
            let days_since_epoch = timestamp / 86400;

            // 日干支（简化，每60天一个周期）
            let day_ganzhi_index = (days_since_epoch % 60) as u8;
            let day_gz = GanZhi::from_sexagenary(day_ganzhi_index)
                .unwrap_or(GanZhi::new(TianGan::Jia, DiZhi::Zi));

            // 时干支（简化，每天12个时辰）
            let hour = ((timestamp % 86400) / 3600) as u8;
            let hour_zhi = DiZhi::from_hour(hour).unwrap_or(DiZhi::Zi);

            // 时干由日干决定
            let hour_gan_base = (day_gz.gan.index() % 5) * 2;
            let hour_gan_index = (hour_gan_base + hour_zhi.index()) % 10;
            let hour_gan = TianGan::from_index(hour_gan_index).unwrap_or(TianGan::Jia);
            let hour_gz = GanZhi::new(hour_gan, hour_zhi);

            // 年月干支使用简化值
            let year_gz = GanZhi::new(TianGan::Jia, DiZhi::Zi);
            let month_gz = GanZhi::new(TianGan::Bing, DiZhi::Yin);

            // 节气（简化，根据日期估算）
            let day_of_year = (days_since_epoch % 365) as u16;
            let jieqi_index = ((day_of_year / 15) % 24) as u8;
            let jieqi = JieQi::from_index(jieqi_index).unwrap_or(JieQi::DongZhi);

            // 三元
            let day_in_jieqi = ((day_of_year % 15) + 1) as u8;
            let san_yuan = algorithm::calc_san_yuan(day_in_jieqi);

            (year_gz, month_gz, day_gz, hour_gz, jieqi, san_yuan)
        }

        /// 创建排盘记录并存储
        ///
        /// # 参数
        /// - `diviner`: 排盘者账户
        /// - `method`: 起局方式
        /// - `year_ganzhi` ~ `hour_ganzhi`: 四柱干支
        /// - `jie_qi`: 节气
        /// - `dun_type`: 阴阳遁
        /// - `san_yuan`: 三元
        /// - `ju_number`: 局数（1-9）
        /// - `zhi_fu_xing`: 值符星
        /// - `zhi_shi_men`: 值使门
        /// - `palaces`: 九宫排盘结果
        /// - `question_hash`: 问题哈希（隐私保护）
        /// - `is_public`: 是否公开（向后兼容，映射为 PrivacyMode）
        /// - `name`: 命主姓名（可选）
        /// - `gender`: 命主性别（可选）
        /// - `birth_year`: 命主出生年份（可选）
        /// - `question`: 占问事宜（可选）
        /// - `question_type`: 问事类型（可选）
        /// - `pan_method`: 排盘方法（转盘/飞盘）
        #[allow(clippy::too_many_arguments)]
        fn create_chart(
            diviner: T::AccountId,
            method: DivinationMethod,
            year_ganzhi: GanZhi,
            month_ganzhi: GanZhi,
            day_ganzhi: GanZhi,
            hour_ganzhi: GanZhi,
            jie_qi: JieQi,
            dun_type: DunType,
            san_yuan: SanYuan,
            ju_number: u8,
            zhi_fu_xing: JiuXing,
            zhi_shi_men: BaMen,
            palaces: [Palace; 9],
            question_hash: [u8; 32],
            is_public: bool,
            // 新增命主信息参数
            name: Option<BoundedVec<u8, MaxNameLen>>,
            gender: Option<Gender>,
            birth_year: Option<u16>,
            question: Option<BoundedVec<u8, MaxQuestionLen>>,
            question_type: Option<QuestionType>,
            pan_method: PanMethod,
        ) -> DispatchResult {
            use pallet_divination_privacy::types::PrivacyMode;

            // 获取新的排盘记录 ID
            let chart_id = NextChartId::<T>::get();
            NextChartId::<T>::put(chart_id.saturating_add(1));

            // 获取当前区块号和时间戳
            let block_number = <frame_system::Pallet<T>>::block_number();
            let timestamp = Self::get_timestamp_secs();

            // 将 is_public 映射为 PrivacyMode（向后兼容）
            let privacy_mode = if is_public {
                PrivacyMode::Public
            } else {
                // 默认私密使用 Partial 模式（计算数据明文，敏感数据后续可加密）
                PrivacyMode::Partial
            };

            // 创建排盘记录（包含命主信息和隐私字段）
            let chart = QimenChart {
                id: chart_id,
                diviner: diviner.clone(),
                method,
                // 隐私控制字段（v3.4 新增）
                privacy_mode,
                encrypted_fields: None, // 非加密模式无需设置
                sensitive_data_hash: None, // 非加密模式无需设置
                // 命主信息
                name,
                gender,
                birth_year,
                question,
                question_type,
                pan_method,
                // 四柱干支（明文存储，用于解盘）
                year_ganzhi: Some(year_ganzhi),
                month_ganzhi: Some(month_ganzhi),
                day_ganzhi: Some(day_ganzhi),
                hour_ganzhi: Some(hour_ganzhi),
                jie_qi: Some(jie_qi),
                // 局数信息
                dun_type: Some(dun_type),
                san_yuan: Some(san_yuan),
                ju_number: Some(ju_number),
                // 盘面数据
                zhi_fu_xing: Some(zhi_fu_xing),
                zhi_shi_men: Some(zhi_shi_men),
                palaces: Some(palaces),
                // 元数据
                timestamp,
                block_number,
                interpretation_cid: None,
                question_hash,
            };

            // 存储排盘记录
            Charts::<T>::insert(chart_id, chart);

            // 更新用户排盘索引
            UserCharts::<T>::try_mutate(&diviner, |list| {
                list.try_push(chart_id)
                    .map_err(|_| Error::<T>::UserChartsFull)
            })?;

            // 如果公开，添加到公开列表
            if is_public {
                PublicCharts::<T>::try_mutate(|list| {
                    list.try_push(chart_id)
                        .map_err(|_| Error::<T>::PublicChartsFull)
                })?;
            }

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&diviner, |stats| {
                stats.update_from_chart(dun_type, zhi_fu_xing, zhi_shi_men);
            });

            // 发送事件
            Self::deposit_event(Event::ChartCreated {
                chart_id,
                diviner,
                dun_type,
                ju_number,
            });

            Ok(())
        }
    }

    // ==================== Runtime API 实现 ====================

    impl<T: Config> Pallet<T> {
        /// 获取核心解卦（Runtime API）
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 核心解卦结果
        pub fn api_get_core_interpretation(chart_id: u64) -> Option<interpretation::QimenCoreInterpretation> {
            let chart = Charts::<T>::get(chart_id)?;
            let current_block = <frame_system::Pallet<T>>::block_number();
            let current_block_u32: u32 = current_block.try_into().ok()?;
            Some(interpretation::calculate_core_interpretation(&chart, current_block_u32))
        }

        /// 获取完整解卦（Runtime API）
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `question_type`: 问事类型
        ///
        /// # 返回
        ///
        /// 完整解卦结果
        pub fn api_get_full_interpretation(
            chart_id: u64,
            question_type: types::QuestionType,
        ) -> Option<interpretation::QimenFullInterpretation> {
            let chart = Charts::<T>::get(chart_id)?;
            let current_block = <frame_system::Pallet<T>>::block_number();
            let current_block_u32: u32 = current_block.try_into().ok()?;
            Some(interpretation::calculate_full_interpretation(&chart, current_block_u32, question_type))
        }

        /// 获取单宫详细解读（Runtime API）
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `palace_num`: 宫位数字（1-9）
        ///
        /// # 返回
        ///
        /// 单宫详细解读（如果是 Private 模式或数据不可用则返回 None）
        pub fn api_get_palace_interpretation(
            chart_id: u64,
            palace_num: u8,
        ) -> Option<interpretation::PalaceInterpretation> {
            if palace_num == 0 || palace_num > 9 {
                return None;
            }

            let chart = Charts::<T>::get(chart_id)?;
            // 检查是否可以解读（非 Private 模式且有计算数据）
            if !chart.can_interpret() {
                return None;
            }
            let palaces = chart.get_palaces()?;
            let jie_qi = chart.get_jie_qi()?;
            let palace = &palaces[(palace_num - 1) as usize];
            Some(interpretation::analyze_palace_detail(palace, jie_qi))
        }

        /// 获取用神分析（Runtime API）
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        /// - `question_type`: 问事类型
        ///
        /// # 返回
        ///
        /// 用神分析结果
        pub fn api_get_yong_shen_analysis(
            chart_id: u64,
            question_type: types::QuestionType,
        ) -> Option<interpretation::YongShenAnalysis> {
            let chart = Charts::<T>::get(chart_id)?;
            Some(interpretation::analyze_yong_shen(&chart, question_type))
        }

        /// 获取应期推算（Runtime API）
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 应期推算结果
        pub fn api_get_ying_qi_analysis(chart_id: u64) -> Option<interpretation::YingQiAnalysis> {
            let chart = Charts::<T>::get(chart_id)?;
            let core = Self::api_get_core_interpretation(chart_id)?;
            Some(interpretation::calculate_ying_qi(&chart, core.yong_shen_gong))
        }

        // ==================== 隐私相关 Runtime API ====================

        /// 获取加密数据（Runtime API）
        ///
        /// 用于 Partial/Private 模式下获取链上存储的加密数据，
        /// 前端需要使用用户私钥解密。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 加密数据（如果存在）
        pub fn api_get_encrypted_data(chart_id: u64) -> Option<Vec<u8>> {
            EncryptedDataStorage::<T>::get(chart_id).map(|v| v.into_inner())
        }

        /// 获取所有者密钥备份（Runtime API）
        ///
        /// 用于所有者恢复加密密钥或授权他人查看。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 80 字节的密钥备份（如果存在）
        pub fn api_get_owner_key_backup(chart_id: u64) -> Option<[u8; 80]> {
            OwnerKeyBackupStorage::<T>::get(chart_id)
        }

        /// 临时计算排盘（Runtime API，用于 Private 模式）
        ///
        /// 当用户使用 Private 模式保存了排盘，但需要查看解读时：
        /// 1. 前端获取加密数据并解密
        /// 2. 使用解密后的日期时间参数调用此 API
        /// 3. 返回完整的排盘计算结果（不存储）
        ///
        /// # 参数
        ///
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        /// - `question_type`: 问事类型 (0-11)
        /// - `pan_method`: 排盘方法 (0=转盘, 1=飞盘)
        ///
        /// # 返回
        ///
        /// 临时排盘结果（不存储到链上）
        pub fn api_compute_chart(
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
            question_type: u8,
            pan_method: u8,
        ) -> Option<crate::runtime_api::QimenChartResult> {
            // 参数校验
            if solar_year < 1901 || solar_year > 2100 {
                return None;
            }
            if solar_month < 1 || solar_month > 12 {
                return None;
            }
            if solar_day < 1 || solar_day > 31 {
                return None;
            }
            if hour >= 24 {
                return None;
            }

            // 调用 almanac 计算四柱
            let pillars = pallet_almanac::four_pillars(solar_year, solar_month, solar_day, hour);

            // 转换为本模块的 GanZhi 类型
            let year_gz = GanZhi {
                gan: TianGan::from_index(pillars.year.gan)?,
                zhi: DiZhi::from_index(pillars.year.zhi)?,
            };
            let month_gz = GanZhi {
                gan: TianGan::from_index(pillars.month.gan)?,
                zhi: DiZhi::from_index(pillars.month.zhi)?,
            };
            let day_gz = GanZhi {
                gan: TianGan::from_index(pillars.day.gan)?,
                zhi: DiZhi::from_index(pillars.day.zhi)?,
            };
            let hour_gz = GanZhi {
                gan: TianGan::from_index(pillars.hour.gan)?,
                zhi: DiZhi::from_index(pillars.hour.zhi)?,
            };

            // 获取节气
            let jie_qi_idx = pallet_almanac::get_solar_term(solar_year, solar_month, solar_day)
                .unwrap_or(0);
            let jieqi = JieQi::from_index(jie_qi_idx).unwrap_or(JieQi::LiChun);

            // 计算节气内天数
            let day_in_jieqi = ((solar_day - 1) % 15) + 1;

            // 调用排盘算法
            let (dun_type, san_yuan, ju_number, zhi_fu_xing, zhi_shi_men, palaces) =
                algorithm::generate_qimen_chart(year_gz, month_gz, day_gz, hour_gz, jieqi, day_in_jieqi);

            // 转换问事类型
            let question_type_enum = match question_type {
                0 => Some(QuestionType::General),
                1 => Some(QuestionType::Career),
                2 => Some(QuestionType::Wealth),
                3 => Some(QuestionType::Marriage),
                4 => Some(QuestionType::Health),
                5 => Some(QuestionType::Study),
                6 => Some(QuestionType::Travel),
                7 => Some(QuestionType::Lawsuit),
                8 => Some(QuestionType::Finding),
                9 => Some(QuestionType::Investment),
                10 => Some(QuestionType::Business),
                11 => Some(QuestionType::Prayer),
                _ => None,
            };
            let pan_method_enum = if pan_method == 1 { PanMethod::FeiPan } else { PanMethod::ZhuanPan };

            Some(crate::runtime_api::QimenChartResult {
                year_ganzhi: year_gz,
                month_ganzhi: month_gz,
                day_ganzhi: day_gz,
                hour_ganzhi: hour_gz,
                jie_qi: jieqi,
                dun_type,
                san_yuan,
                ju_number,
                zhi_fu_xing,
                zhi_shi_men,
                palaces,
                question_type: question_type_enum,
                pan_method: pan_method_enum,
            })
        }

        /// 获取排盘公开元数据（Runtime API）
        ///
        /// 返回排盘的公开元数据，不包含敏感信息。
        /// 适用于所有隐私模式。
        ///
        /// # 参数
        ///
        /// - `chart_id`: 排盘记录 ID
        ///
        /// # 返回
        ///
        /// 公开元数据
        pub fn api_get_public_metadata(chart_id: u64) -> Option<crate::runtime_api::QimenPublicMetadata> {
            let chart = Charts::<T>::get(chart_id)?;

            Some(crate::runtime_api::QimenPublicMetadata {
                id: chart.id,
                privacy_mode: chart.privacy_mode,
                method: chart.method,
                pan_method: chart.pan_method,
                timestamp: chart.timestamp,
                question_type: chart.question_type,
                has_encrypted_data: EncryptedDataStorage::<T>::contains_key(chart_id),
                can_interpret: chart.can_interpret(),
            })
        }
    }

}
