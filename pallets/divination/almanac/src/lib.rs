//! # 黄历模块 (pallet-almanac)
//!
//! ## 概述
//! 该模块通过 Off-chain Worker 获取黄历数据并存储到链上，
//! 为占卜系统提供日期相关的黄历信息查询服务。
//!
//! ## 功能特性
//! - 通过 OCW 定期从阿里云黄历 API 获取数据
//! - 支持手动设置黄历数据 (需要权限)
//! - 提供按日期、月份、年份查询黄历的接口
//! - 支持查询节气、节日等信息
//!
//! ## 存储优化
//! - 使用紧凑的 AlmanacInfo 结构 (~50 bytes/天)
//! - 支持批量获取和设置
//! - 自动清理过期数据 (可选)
//!
//! ## 安全设计
//! - AppCode 通过环境变量配置，不在链上存储
//! - 数据提交需要权限验证
//! - OCW 使用独立账户签名

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

mod types;
pub use types::*;

/// 本地农历计算模块（预存储200年数据）
///
/// 提供公历转农历、干支计算等核心功能，供其他占卜模块统一调用
pub mod lunar;

/// 农历数据表 (1901-2100)
/// 数据来源：香港天文台
pub mod lunar_data;

// 重新导出常用类型和函数，方便其他模块使用
pub use lunar::{
    // 核心类型
    LunarDate, GanZhi, FourPillars,
    // 梅花易数专用类型
    MeihuaLunarDate, LunarConvertError,
    // 转换函数
    solar_to_lunar, lunar_to_solar, is_leap_year, julian_day, from_julian_day,
    timestamp_to_meihua_lunar, hour_to_dizhi_num, year_to_dizhi_num,
    // 干支计算
    year_ganzhi, month_ganzhi, day_ganzhi, hour_ganzhi, four_pillars,
    // 生肖
    get_zodiac, zodiac_name,
    // 节气
    get_solar_term, solar_term_name,
    // 常量
    TIANGAN, DIZHI, SHENGXIAO, LUNAR_MONTHS, LUNAR_DAYS, SOLAR_TERMS,
    // 数据范围常量
    LUNAR_START_YEAR, LUNAR_END_YEAR,
};

#[cfg(feature = "std")]
mod offchain;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_timestamp::Config {
        /// 权重信息
        type WeightInfo: WeightInfo;

        /// 最大批量设置数量
        #[pallet::constant]
        type MaxBatchSize: Get<u32>;

        /// 最大历史数据年限 (默认 3 年)
        #[pallet::constant]
        type MaxHistoryYears: Get<u32>;
    }

    /// 权重信息 trait
    pub trait WeightInfo {
        fn set_almanac() -> Weight;
        fn batch_set_almanac(n: u32) -> Weight;
        fn configure_ocw() -> Weight;
        fn add_authority() -> Weight;
        fn remove_authority() -> Weight;
        fn remove_almanac() -> Weight;
    }

    /// 默认权重实现
    impl WeightInfo for () {
        fn set_almanac() -> Weight {
            Weight::from_parts(10_000, 0)
        }
        fn batch_set_almanac(n: u32) -> Weight {
            Weight::from_parts(10_000 * n as u64, 0)
        }
        fn configure_ocw() -> Weight {
            Weight::from_parts(5_000, 0)
        }
        fn add_authority() -> Weight {
            Weight::from_parts(5_000, 0)
        }
        fn remove_authority() -> Weight {
            Weight::from_parts(5_000, 0)
        }
        fn remove_almanac() -> Weight {
            Weight::from_parts(5_000, 0)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ========================================================================
    // 存储定义
    // ========================================================================

    /// 黄历数据存储
    /// 键: (公历年, 月, 日) => AlmanacInfo
    #[pallet::storage]
    #[pallet::getter(fn almanac_data)]
    pub type AlmanacData<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        DateKey, // (year, month, day)
        AlmanacInfo,
        OptionQuery,
    >;

    /// OCW 配置
    #[pallet::storage]
    #[pallet::getter(fn ocw_config)]
    pub type OcwConfigStorage<T: Config> = StorageValue<_, OcwConfig, ValueQuery>;

    /// 有权限提交数据的账户
    #[pallet::storage]
    #[pallet::getter(fn data_authorities)]
    pub type DataAuthorities<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        bool,
        ValueQuery,
    >;

    /// 年度数据统计
    /// 键: 年份 => (总天数, OCW更新数, 手动更新数)
    #[pallet::storage]
    #[pallet::getter(fn data_stats)]
    pub type DataStats<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u16, // year
        (u32, u32, u32),
        ValueQuery,
    >;

    /// 最近更新日期
    #[pallet::storage]
    #[pallet::getter(fn last_updated_date)]
    pub type LastUpdatedDate<T: Config> = StorageValue<_, DateKey, OptionQuery>;

    // ========================================================================
    // 事件定义
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 黄历数据已更新
        AlmanacUpdated {
            /// 日期 (年, 月, 日)
            date: DateKey,
            /// 数据来源
            source: u8,
            /// 更新者
            updater: T::AccountId,
        },

        /// 批量更新黄历数据
        AlmanacBatchUpdated {
            /// 更新数量
            count: u32,
            /// 更新者
            updater: T::AccountId,
        },

        /// 黄历数据已删除
        AlmanacRemoved {
            /// 日期
            date: DateKey,
        },

        /// OCW 配置已更新
        OcwConfigured,

        /// 添加了数据提交权限
        AuthorityAdded {
            /// 账户
            account: T::AccountId,
        },

        /// 移除了数据提交权限
        AuthorityRemoved {
            /// 账户
            account: T::AccountId,
        },

        /// OCW 获取数据成功
        OcwFetchSuccess {
            /// 获取的日期范围起始
            start_date: DateKey,
            /// 获取数量
            count: u32,
        },

        /// OCW 获取数据失败
        OcwFetchFailed {
            /// 错误信息
            reason: Vec<u8>,
        },
    }

    // ========================================================================
    // 错误定义
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 无操作权限
        NoPermission,

        /// 无效的日期
        InvalidDate,

        /// 无效的配置
        InvalidConfig,

        /// 批量操作数量超限
        BatchTooLarge,

        /// 数据不存在
        DataNotFound,

        /// 数据已存在
        DataAlreadyExists,

        /// OCW 未启用
        OcwNotEnabled,

        /// AppCode 未配置
        AppCodeNotConfigured,

        /// API 调用失败
        ApiCallFailed,

        /// JSON 解析失败
        JsonParseFailed,

        /// 数据验证失败
        DataValidationFailed,
    }

    // ========================================================================
    // Hooks (生命周期)
    // ========================================================================

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Off-chain Worker 入口
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            #[cfg(feature = "std")]
            {
                // 检查是否应该触发 OCW
                if Self::should_trigger_ocw(block_number) {
                    log::info!(
                        target: "almanac-ocw",
                        "🗓️ Almanac OCW triggered at block {:?}",
                        block_number
                    );

                    // 执行获取和提交逻辑
                    if let Err(e) = Self::fetch_and_submit_almanac() {
                        log::error!(
                            target: "almanac-ocw",
                            "❌ Almanac OCW error: {:?}",
                            e
                        );
                    }
                }
            }
        }

        /// 区块初始化
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            Weight::zero()
        }
    }

    // ========================================================================
    // 交易方法
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 设置单日黄历数据
        ///
        /// # 参数
        /// - `origin`: 调用者，需要是 Authority 或 Root
        /// - `year`: 公历年份
        /// - `month`: 公历月份 (1-12)
        /// - `day`: 公历日期 (1-31)
        /// - `info`: 黄历数据
        ///
        /// # 权限
        /// 需要 Root 或 DataAuthority 权限
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::set_almanac())]
        pub fn set_almanac(
            origin: OriginFor<T>,
            year: u16,
            month: u8,
            day: u8,
            info: AlmanacInfo,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查权限
            ensure!(
                Self::has_authority(&who),
                Error::<T>::NoPermission
            );

            // 验证日期
            ensure!(
                validate_date(year, month, day),
                Error::<T>::InvalidDate
            );

            let date_key: DateKey = (year, month, day);

            // 存储数据
            AlmanacData::<T>::insert(date_key, info.clone());

            // 更新统计
            Self::update_stats(year, info.source);

            // 更新最近更新日期
            LastUpdatedDate::<T>::put(date_key);

            // 发出事件
            Self::deposit_event(Event::AlmanacUpdated {
                date: date_key,
                source: info.source,
                updater: who,
            });

            Ok(())
        }

        /// 批量设置黄历数据
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `data`: 日期和黄历数据的数组
        ///
        /// # 限制
        /// 单次最多设置 MaxBatchSize 条数据
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::batch_set_almanac(data.len() as u32))]
        pub fn batch_set_almanac(
            origin: OriginFor<T>,
            data: Vec<(DateKey, AlmanacInfo)>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查权限
            ensure!(
                Self::has_authority(&who),
                Error::<T>::NoPermission
            );

            // 检查批量大小
            ensure!(
                data.len() as u32 <= T::MaxBatchSize::get(),
                Error::<T>::BatchTooLarge
            );

            let mut last_date = None;

            for ((year, month, day), info) in data.iter() {
                // 验证日期
                if !validate_date(*year, *month, *day) {
                    continue; // 跳过无效日期
                }

                let date_key: DateKey = (*year, *month, *day);
                AlmanacData::<T>::insert(date_key, info.clone());
                Self::update_stats(*year, info.source);
                last_date = Some(date_key);
            }

            // 更新最近更新日期
            if let Some(date) = last_date {
                LastUpdatedDate::<T>::put(date);
            }

            Self::deposit_event(Event::AlmanacBatchUpdated {
                count: data.len() as u32,
                updater: who,
            });

            Ok(())
        }

        /// 配置 OCW 参数
        ///
        /// # 参数
        /// - `origin`: 必须是 Root
        /// - `config`: OCW 配置
        #[pallet::call_index(2)]
        #[pallet::weight(<T as Config>::WeightInfo::configure_ocw())]
        pub fn configure_ocw(
            origin: OriginFor<T>,
            config: OcwConfig,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // 验证配置
            ensure!(config.update_hour < 24, Error::<T>::InvalidConfig);
            ensure!(
                config.batch_days > 0 && config.batch_days <= 90,
                Error::<T>::InvalidConfig
            );

            OcwConfigStorage::<T>::put(config);

            Self::deposit_event(Event::OcwConfigured);

            Ok(())
        }

        /// 添加数据提交权限
        ///
        /// # 参数
        /// - `origin`: 必须是 Root
        /// - `account`: 要添加权限的账户
        #[pallet::call_index(3)]
        #[pallet::weight(<T as Config>::WeightInfo::add_authority())]
        pub fn add_authority(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            DataAuthorities::<T>::insert(&account, true);

            Self::deposit_event(Event::AuthorityAdded { account });

            Ok(())
        }

        /// 移除数据提交权限
        ///
        /// # 参数
        /// - `origin`: 必须是 Root
        /// - `account`: 要移除权限的账户
        #[pallet::call_index(4)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_authority())]
        pub fn remove_authority(
            origin: OriginFor<T>,
            account: T::AccountId,
        ) -> DispatchResult {
            ensure_root(origin)?;

            DataAuthorities::<T>::remove(&account);

            Self::deposit_event(Event::AuthorityRemoved { account });

            Ok(())
        }

        /// 删除特定日期的黄历数据
        ///
        /// # 参数
        /// - `origin`: 必须是 Root
        /// - `year`, `month`, `day`: 要删除的日期
        #[pallet::call_index(5)]
        #[pallet::weight(<T as Config>::WeightInfo::remove_almanac())]
        pub fn remove_almanac(
            origin: OriginFor<T>,
            year: u16,
            month: u8,
            day: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            let date_key: DateKey = (year, month, day);

            ensure!(
                AlmanacData::<T>::contains_key(date_key),
                Error::<T>::DataNotFound
            );

            AlmanacData::<T>::remove(date_key);

            Self::deposit_event(Event::AlmanacRemoved { date: date_key });

            Ok(())
        }
    }

    // ========================================================================
    // 内部辅助方法
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 检查账户是否有数据提交权限
        pub fn has_authority(account: &T::AccountId) -> bool {
            DataAuthorities::<T>::get(account)
        }

        /// 更新年度统计数据
        fn update_stats(year: u16, source: u8) {
            DataStats::<T>::mutate(year, |(total, ocw, manual)| {
                *total += 1;
                match source {
                    0 => *ocw += 1,     // OCW API
                    1 => *manual += 1,  // 手动
                    _ => {}
                }
            });
        }

        /// 获取指定月份的所有黄历数据
        pub fn get_month_almanac(year: u16, month: u8) -> Vec<(u8, AlmanacInfo)> {
            let mut result = Vec::new();

            for day in 1..=31 {
                if let Some(info) = AlmanacData::<T>::get((year, month, day)) {
                    result.push((day, info));
                }
            }

            result
        }

        /// 获取指定年份的节气列表
        pub fn get_solar_terms(year: u16) -> Vec<((u8, u8), u8)> {
            let mut result = Vec::new();

            for month in 1..=12 {
                for day in 1..=31 {
                    if let Some(info) = AlmanacData::<T>::get((year, month, day)) {
                        if info.solar_term > 0 {
                            result.push(((month, day), info.solar_term));
                        }
                    }
                }
            }

            result
        }

        /// 检查是否应该触发 OCW
        #[cfg(feature = "std")]
        fn should_trigger_ocw(_block_number: BlockNumberFor<T>) -> bool {
            let config = Self::ocw_config();

            // 检查是否启用
            if !config.enabled {
                return false;
            }

            // 简单的触发逻辑：每 100 个区块检查一次
            // 实际生产环境应该根据时间判断
            true // 暂时总是返回 true，便于测试
        }

        /// 从 API 获取数据并提交到链上
        #[cfg(feature = "std")]
        fn fetch_and_submit_almanac() -> Result<(), &'static str> {
            // 这里调用 offchain 模块的实现
            offchain::fetch_and_submit::<T>()
        }
    }
}
