//! # pallet-liuyao
//!
//! ## 六爻排盘系统 - 区块链纳甲六爻占卜模块
//!
//! 本模块实现完整的六爻排盘算法，支持链上卦象生成与存储。
//!
//! ### 核心功能
//!
//! - **铜钱起卦**：模拟三枚铜钱法
//! - **数字起卦**：报数法起卦
//! - **时间起卦**：根据时辰自动起卦
//! - **随机起卦**：使用链上随机数
//! - **手动指定**：直接输入六爻
//! - **纳甲装卦**：自动装配天干地支
//! - **六亲六神**：自动计算六亲和六神
//! - **世应伏神**：自动安世应、查伏神
//! - **AI 解读**：集成通用占卜 AI 解读系统
//!
//! ### 技术架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    pallet-liuyao                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Extrinsics:                                                 │
//! │  - divine_by_coins: 铜钱起卦                                  │
//! │  - divine_by_numbers: 数字起卦                                │
//! │  - divine_by_time: 时间起卦                                   │
//! │  - divine_random: 随机起卦                                    │
//! │  - divine_manual: 手动指定                                    │
//! │  - request_ai_interpretation: 请求AI解读                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Algorithm:                                                  │
//! │  - 纳甲算法（八卦配天干地支）                                   │
//! │  - 世应计算（寻世诀）                                          │
//! │  - 卦宫归属（认宫诀）                                          │
//! │  - 六亲配置                                                   │
//! │  - 六神排布                                                   │
//! │  - 旬空计算                                                   │
//! │  - 伏神查找                                                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod algorithm;
pub mod interpretation;
pub mod ocw_tee;
pub mod runtime_api;
pub mod shensha;
pub mod types;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub use shensha::*;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::algorithm::*;
    use frame_support::{
        pallet_prelude::*,
        traits::Randomness,
        BoundedVec,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置 trait
    ///
    /// 注：RuntimeEvent 关联类型已从 Polkadot SDK 2506 版本开始自动附加
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 随机数生成器
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 每用户最大卦象数量
        #[pallet::constant]
        type MaxUserGuas: Get<u32>;

        /// 公开列表最大长度
        #[pallet::constant]
        type MaxPublicGuas: Get<u32>;

        /// 每日免费起卦次数
        #[pallet::constant]
        type DailyFreeGuas: Get<u32>;

        /// 每日最大起卦次数
        #[pallet::constant]
        type MaxDailyGuas: Get<u32>;

        /// IPFS CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 加密数据最大长度（默认: 512 bytes）
        #[pallet::constant]
        type MaxEncryptedLen: Get<u32>;

        // ================================
        // 存储押金相关配置
        // ================================

        /// 货币类型（需要支持 reserve/unreserve）
        type Currency: frame_support::traits::Currency<Self::AccountId>
            + frame_support::traits::ReservableCurrency<Self::AccountId>;
    }

    // ========================================================================
    // 存储项
    // ========================================================================

    /// 下一个卦象 ID
    #[pallet::storage]
    #[pallet::getter(fn next_gua_id)]
    pub type NextGuaId<T> = StorageValue<_, u64, ValueQuery>;

    /// 所有卦象数据
    #[pallet::storage]
    #[pallet::getter(fn guas)]
    pub type Guas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        LiuYaoGua<T::AccountId, BlockNumberFor<T>, T::MaxCidLen>,
    >;

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

    /// 用户的卦象列表
    #[pallet::storage]
    #[pallet::getter(fn user_guas)]
    pub type UserGuas<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxUserGuas>,
        ValueQuery,
    >;

    /// 公开的卦象列表
    #[pallet::storage]
    #[pallet::getter(fn public_guas)]
    pub type PublicGuas<T: Config> = StorageValue<_, BoundedVec<u64, T::MaxPublicGuas>, ValueQuery>;

    /// 用户每日起卦次数
    #[pallet::storage]
    #[pallet::getter(fn daily_gua_count)]
    pub type DailyGuaCount<T: Config> =
        StorageMap<_, Blake2_128Concat, (T::AccountId, u32), u32, ValueQuery>;

    /// 用户统计数据
    #[pallet::storage]
    #[pallet::getter(fn user_stats)]
    pub type UserStatsStorage<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserStats, ValueQuery>;

    /// 货币余额类型别名
    pub type BalanceOf<T> =
        <<T as Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance;

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 卦象创建成功
        GuaCreated {
            gua_id: u64,
            creator: T::AccountId,
            method: DivinationMethod,
            original_name_idx: Option<u8>,
        },
        /// 可见性变更
        VisibilityChanged {
            gua_id: u64,
            is_public: bool,
        },
        /// 加密卦象创建成功
        EncryptedGuaCreated {
            gua_id: u64,
            creator: T::AccountId,
            privacy_mode: pallet_divination_privacy::types::PrivacyMode,
            method: DivinationMethod,
        },
        /// 加密数据已更新
        EncryptedDataUpdated {
            gua_id: u64,
            data_hash: [u8; 32],
        },
        /// 卦象已删除
        GuaDeleted {
            gua_id: u64,
            owner: T::AccountId,
        },
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 卦象不存在
        GuaNotFound,
        /// 无权操作
        NotGuaOwner,
        /// 无效的铜钱数（应为0-3）
        InvalidCoinCount,
        /// 无效的数字（应大于0）
        InvalidNumber,
        /// 无效的动爻位置（应为1-6）
        InvalidDongYao,
        /// 超过每日起卦上限
        DailyLimitExceeded,
        /// 超过用户存储上限
        UserGuaLimitExceeded,
        /// 超过公开列表上限
        PublicGuaLimitExceeded,
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

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 铜钱起卦 - 模拟三枚铜钱法
        ///
        /// # 参数
        /// - `coins`: 六次摇卦结果，每个值为阳面个数(0-3)
        /// - `year_gz`: 年干支
        /// - `month_gz`: 月干支
        /// - `day_gz`: 日干支
        /// - `hour_gz`: 时干支
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn divine_by_coins(
            origin: OriginFor<T>,
            coins: [u8; 6],
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            for &coin in coins.iter() {
                ensure!(coin <= 3, Error::<T>::InvalidCoinCount);
            }

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 从铜钱结果生成六爻
            let yaos = coins_to_yaos(&coins);

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yaos,
                DivinationMethod::CoinMethod,
                (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1)),
                (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1)),
                (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1)),
                (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1)),
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::CoinMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 数字起卦 - 报数法
        ///
        /// # 参数
        /// - `upper_num`: 上卦数（对应外卦，用户报的第一个数）
        /// - `lower_num`: 下卦数（对应内卦，用户报的第二个数）
        /// - `dong`: 动爻位置（1-6，从初爻到上爻）
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn divine_by_numbers(
            origin: OriginFor<T>,
            upper_num: u16,
            lower_num: u16,
            dong: u8,
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            ensure!(upper_num > 0 && lower_num > 0, Error::<T>::InvalidNumber);
            ensure!(dong >= 1 && dong <= 6, Error::<T>::InvalidDongYao);

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 从数字生成六爻
            let yaos = numbers_to_yaos(upper_num, lower_num, dong);

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yaos,
                DivinationMethod::NumberMethod,
                (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1)),
                (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1)),
                (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1)),
                (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1)),
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::NumberMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 随机起卦 - 使用链上随机数
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn divine_random(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 生成随机数据
            let (random_hash, _) = T::Randomness::random(&who.encode());
            let random_bytes: [u8; 32] = random_hash.as_ref().try_into().unwrap_or([0u8; 32]);

            // 从随机数生成六爻
            let yaos = random_to_yaos(&random_bytes);

            // 生成随机干支
            let year_gz = (
                TianGan::from_index(random_bytes[24] % 10),
                DiZhi::from_index(random_bytes[25] % 12),
            );
            let month_gz = (
                TianGan::from_index(random_bytes[26] % 10),
                DiZhi::from_index(random_bytes[27] % 12),
            );
            let day_gz = (
                TianGan::from_index(random_bytes[28] % 10),
                DiZhi::from_index(random_bytes[29] % 12),
            );
            let hour_gz = (
                TianGan::from_index(random_bytes[30] % 10),
                DiZhi::from_index(random_bytes[31] % 12),
            );

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yaos,
                DivinationMethod::RandomMethod,
                year_gz,
                month_gz,
                day_gz,
                hour_gz,
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::RandomMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 手动起卦 - 直接输入六爻
        ///
        /// # 参数
        /// - `yaos`: 六爻类型（0=少阴, 1=少阳, 2=老阴, 3=老阳）
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn divine_manual(
            origin: OriginFor<T>,
            yaos: [u8; 6],
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            for &yao in yaos.iter() {
                ensure!(yao <= 3, Error::<T>::InvalidCoinCount);
            }

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 转换为Yao类型
            let mut yao_array = [Yao::ShaoYin; 6];
            for i in 0..6 {
                yao_array[i] = match yaos[i] {
                    0 => Yao::ShaoYin,
                    1 => Yao::ShaoYang,
                    2 => Yao::LaoYin,
                    _ => Yao::LaoYang,
                };
            }

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yao_array,
                DivinationMethod::ManualMethod,
                (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1)),
                (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1)),
                (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1)),
                (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1)),
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::ManualMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 时间起卦 - 根据年月日时起卦
        ///
        /// # 参数
        /// - `year_zhi`: 年地支索引 (0-11，子=0)
        /// - `month_num`: 月数 (1-12)
        /// - `day_num`: 日数 (1-31)
        /// - `hour_zhi`: 时辰地支索引 (0-11)
        /// - `year_gz`: 年干支（用于排盘）
        /// - `month_gz`: 月干支
        /// - `day_gz`: 日干支
        /// - `hour_gz`: 时干支
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(100_000_000, 0))]
        pub fn divine_by_time(
            origin: OriginFor<T>,
            year_zhi: u8,
            month_num: u8,
            day_num: u8,
            hour_zhi: u8,
            year_gz: (u8, u8),
            month_gz: (u8, u8),
            day_gz: (u8, u8),
            hour_gz: (u8, u8),
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            ensure!(year_zhi < 12, Error::<T>::InvalidNumber);
            ensure!(month_num >= 1 && month_num <= 12, Error::<T>::InvalidNumber);
            ensure!(day_num >= 1 && day_num <= 31, Error::<T>::InvalidNumber);
            ensure!(hour_zhi < 12, Error::<T>::InvalidNumber);

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 调用时间起卦算法
            let yaos = time_to_yaos(year_zhi, month_num, day_num, hour_zhi);

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yaos,
                DivinationMethod::TimeMethod,
                (TianGan::from_index(year_gz.0), DiZhi::from_index(year_gz.1)),
                (TianGan::from_index(month_gz.0), DiZhi::from_index(month_gz.1)),
                (TianGan::from_index(day_gz.0), DiZhi::from_index(day_gz.1)),
                (TianGan::from_index(hour_gz.0), DiZhi::from_index(hour_gz.1)),
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::TimeMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 公历时间起卦 - 根据公历日期时间自动计算干支后起卦
        ///
        /// 此方法使用 pallet-almanac 自动将公历日期转换为农历和四柱干支，
        /// 然后进行时间起卦。用户无需手动计算干支。
        ///
        /// # 参数
        /// - `solar_year`: 公历年份 (1901-2100)
        /// - `solar_month`: 公历月份 (1-12)
        /// - `solar_day`: 公历日期 (1-31)
        /// - `hour`: 小时 (0-23)
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(120_000_000, 0))]
        pub fn divine_by_solar_time(
            origin: OriginFor<T>,
            solar_year: u16,
            solar_month: u8,
            solar_day: u8,
            hour: u8,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 参数校验
            ensure!(solar_year >= 1901 && solar_year <= 2100, Error::<T>::InvalidNumber);
            ensure!(solar_month >= 1 && solar_month <= 12, Error::<T>::InvalidNumber);
            ensure!(solar_day >= 1 && solar_day <= 31, Error::<T>::InvalidNumber);
            ensure!(hour < 24, Error::<T>::InvalidNumber);

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 调用 almanac 计算四柱
            let pillars = pallet_almanac::four_pillars(solar_year, solar_month, solar_day, hour);

            // 调用 almanac 转农历（用于起卦公式）
            let lunar = pallet_almanac::solar_to_lunar(solar_year, solar_month, solar_day)
                .ok_or(Error::<T>::InvalidNumber)?;

            // 计算时辰地支数（0-11）
            let hour_zhi = pallet_almanac::hour_to_dizhi_num(hour).saturating_sub(1); // 1-based 转 0-based

            // 调用时间起卦算法
            let year_zhi = pallet_almanac::year_to_dizhi_num(lunar.year).saturating_sub(1); // 1-based 转 0-based
            let yaos = time_to_yaos(year_zhi, lunar.month, lunar.day, hour_zhi);

            // 执行排卦
            let gua_id = Self::do_divine(
                &who,
                yaos,
                DivinationMethod::TimeMethod,
                (TianGan::from_index(pillars.year.gan), DiZhi::from_index(pillars.year.zhi)),
                (TianGan::from_index(pillars.month.gan), DiZhi::from_index(pillars.month.zhi)),
                (TianGan::from_index(pillars.day.gan), DiZhi::from_index(pillars.day.zhi)),
                (TianGan::from_index(pillars.hour.gan), DiZhi::from_index(pillars.hour.zhi)),
            )?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 发出事件
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            Self::deposit_event(Event::GuaCreated {
                gua_id,
                creator: who,
                method: DivinationMethod::TimeMethod,
                original_name_idx: gua.original_name_idx,
            });

            Ok(())
        }

        /// 设置卦象可见性
        ///
        /// # 参数
        /// - `gua_id`: 卦象 ID
        /// - `is_public`: 是否公开（true = Public 模式，false = Partial 模式）
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn set_gua_visibility(
            origin: OriginFor<T>,
            gua_id: u64,
            is_public: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查卦象存在且属于调用者
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            ensure!(gua.creator == who, Error::<T>::NotGuaOwner);

            // 更新可见性（使用 privacy_mode）
            Guas::<T>::mutate(gua_id, |maybe_gua| {
                if let Some(gua) = maybe_gua {
                    gua.privacy_mode = if is_public {
                        pallet_divination_privacy::types::PrivacyMode::Public
                    } else {
                        pallet_divination_privacy::types::PrivacyMode::Partial
                    };
                }
            });

            // 更新公开列表
            if is_public {
                PublicGuas::<T>::try_mutate(|list| {
                    if !list.contains(&gua_id) {
                        list.try_push(gua_id).map_err(|_| Error::<T>::PublicGuaLimitExceeded)
                    } else {
                        Ok(())
                    }
                })?;
            } else {
                PublicGuas::<T>::mutate(|list| {
                    list.retain(|&id| id != gua_id);
                });
            }

            Self::deposit_event(Event::VisibilityChanged { gua_id, is_public });

            Ok(())
        }

        /// 加密铜钱起卦 - 支持三种隐私模式
        ///
        /// # 隐私模式
        /// - 0 (Public): 所有数据明文存储
        /// - 1 (Partial): 计算数据明文，敏感数据加密
        /// - 2 (Private): 所有数据加密，仅存储加密数据和元数据
        ///
        /// # 参数
        /// - `encryption_level`: 加密级别（0-2）
        /// - `coins`: 六次摇卦结果（Public/Partial 模式需要）
        /// - `year_gz`, `month_gz`, `day_gz`, `hour_gz`: 干支信息
        /// - `encrypted_data`: 加密的敏感数据（Partial/Private 模式需要）
        /// - `data_hash`: 敏感数据哈希
        /// - `owner_key_backup`: 所有者密钥备份
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(120_000_000, 0))]
        pub fn divine_by_coins_encrypted(
            origin: OriginFor<T>,
            encryption_level: u8,
            coins: Option<[u8; 6]>,
            year_gz: Option<(u8, u8)>,
            month_gz: Option<(u8, u8)>,
            day_gz: Option<(u8, u8)>,
            hour_gz: Option<(u8, u8)>,
            encrypted_data: Option<BoundedVec<u8, T::MaxEncryptedLen>>,
            data_hash: Option<[u8; 32]>,
            owner_key_backup: Option<[u8; 80]>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 校验加密级别
            ensure!(encryption_level <= 2, Error::<T>::InvalidEncryptionLevel);

            // 检查每日限制
            Self::check_daily_limit(&who)?;

            // 根据隐私模式处理
            let privacy_mode = match encryption_level {
                0 => pallet_divination_privacy::types::PrivacyMode::Public,
                1 => pallet_divination_privacy::types::PrivacyMode::Partial,
                _ => pallet_divination_privacy::types::PrivacyMode::Private,
            };

            // Partial/Private 模式需要加密数据
            if encryption_level >= 1 {
                ensure!(encrypted_data.is_some(), Error::<T>::EncryptedDataMissing);
                ensure!(data_hash.is_some(), Error::<T>::DataHashMissing);
                ensure!(owner_key_backup.is_some(), Error::<T>::OwnerKeyBackupMissing);
            }

            // 获取新 ID
            let gua_id = NextGuaId::<T>::get();
            NextGuaId::<T>::put(gua_id + 1);

            // 检查用户存储上限
            let user_guas = UserGuas::<T>::get(&who);
            ensure!(
                user_guas.len() < T::MaxUserGuas::get() as usize,
                Error::<T>::UserGuaLimitExceeded
            );

            // 根据模式创建卦象
            let gua = if encryption_level == 2 {
                // Private 模式：不存储计算数据
                LiuYaoGua {
                    id: gua_id,
                    creator: who.clone(),
                    created_at: <frame_system::Pallet<T>>::block_number(),
                    privacy_mode,
                    encrypted_fields: Some(0xFF), // 所有字段加密
                    sensitive_data_hash: data_hash,
                    method: DivinationMethod::CoinMethod,
                    question_cid: None,
                    year_gz: None,
                    month_gz: None,
                    day_gz: None,
                    hour_gz: None,
                    original_yaos: None,
                    original_inner: None,
                    original_outer: None,
                    original_name_idx: None,
                    gong: None,
                    gua_xu: None,
                    has_bian_gua: false,
                    changed_yaos: None,
                    changed_inner: None,
                    changed_outer: None,
                    changed_name_idx: None,
                    hu_inner: None,
                    hu_outer: None,
                    hu_name_idx: None,
                    gua_shen: None,
                    moving_yaos: None,
                    xun_kong: None,
                    fu_shen: None,
                }
            } else {
                // Public/Partial 模式：存储计算数据
                let coins_data = coins.ok_or(Error::<T>::InvalidCoinCount)?;

                // 参数校验
                for &coin in coins_data.iter() {
                    ensure!(coin <= 3, Error::<T>::InvalidCoinCount);
                }

                let year = year_gz.ok_or(Error::<T>::InvalidNumber)?;
                let month = month_gz.ok_or(Error::<T>::InvalidNumber)?;
                let day = day_gz.ok_or(Error::<T>::InvalidNumber)?;
                let hour = hour_gz.ok_or(Error::<T>::InvalidNumber)?;

                // 从铜钱结果生成六爻
                let yaos = coins_to_yaos(&coins_data);

                let year_gz_val = (TianGan::from_index(year.0), DiZhi::from_index(year.1));
                let month_gz_val = (TianGan::from_index(month.0), DiZhi::from_index(month.1));
                let day_gz_val = (TianGan::from_index(day.0), DiZhi::from_index(day.1));
                let hour_gz_val = (TianGan::from_index(hour.0), DiZhi::from_index(hour.1));

                // 计算内外卦
                let (original_inner, original_outer) = yaos_to_trigrams(&yaos);
                let (gua_xu, gong) = calculate_shi_ying_gong(original_inner, original_outer);
                let liu_shen_array = calculate_liu_shen(day_gz_val.0);
                let xun_kong = calculate_xun_kong(day_gz_val.0, day_gz_val.1);
                let original_name_idx = calculate_gua_index(original_inner, original_outer);

                // 构建本卦六爻信息
                let gong_wx = gong.wu_xing();
                let mut original_yaos_arr = [YaoInfo::default(); 6];
                let mut liu_qin_array = [LiuQin::XiongDi; 6];

                for i in 0..6 {
                    let (gan, zhi) = if i < 3 {
                        get_inner_najia(original_inner, i as u8)
                    } else {
                        get_outer_najia(original_outer, (i - 3) as u8)
                    };
                    let yao_wx = zhi.wu_xing();
                    let liu_qin = LiuQin::from_wu_xing(gong_wx, yao_wx);
                    liu_qin_array[i] = liu_qin;

                    let shi_pos = gua_xu.shi_yao_pos() as usize;
                    let ying_pos = gua_xu.ying_yao_pos() as usize;

                    original_yaos_arr[i] = YaoInfo {
                        yao: yaos[i],
                        tian_gan: gan,
                        di_zhi: zhi,
                        wu_xing: yao_wx,
                        liu_qin,
                        liu_shen: liu_shen_array[i],
                        is_shi: i + 1 == shi_pos,
                        is_ying: i + 1 == ying_pos,
                    };
                }

                // 计算变卦
                let (changed_inner, changed_outer, has_bian_gua) = calculate_bian_gua(&yaos);
                let changed_name_idx = calculate_gua_index(changed_inner, changed_outer);

                let mut changed_yaos_arr = [YaoInfo::default(); 6];
                if has_bian_gua {
                    for i in 0..6 {
                        let (gan, zhi) = if i < 3 {
                            get_inner_najia(changed_inner, i as u8)
                        } else {
                            get_outer_najia(changed_outer, (i - 3) as u8)
                        };
                        let yao_wx = zhi.wu_xing();
                        let liu_qin = LiuQin::from_wu_xing(gong_wx, yao_wx);

                        changed_yaos_arr[i] = YaoInfo {
                            yao: if yaos[i].is_moving() {
                                if yaos[i].is_yang() { Yao::ShaoYin } else { Yao::ShaoYang }
                            } else {
                                yaos[i]
                            },
                            tian_gan: gan,
                            di_zhi: zhi,
                            wu_xing: yao_wx,
                            liu_qin,
                            liu_shen: liu_shen_array[i],
                            is_shi: false,
                            is_ying: false,
                        };
                    }
                }

                let moving_yaos = calculate_moving_bitmap(&yaos);
                let (hu_inner, hu_outer) = calculate_hu_gua(&yaos);
                let hu_name_idx = calculate_gua_index(hu_inner, hu_outer);
                let shi_pos = gua_xu.shi_yao_pos();
                let shi_is_yang = yaos[(shi_pos - 1) as usize].is_yang();
                let gua_shen = calculate_gua_shen(shi_pos, shi_is_yang);
                let fu_shen = find_fu_shen(gong, &liu_qin_array);

                LiuYaoGua {
                    id: gua_id,
                    creator: who.clone(),
                    created_at: <frame_system::Pallet<T>>::block_number(),
                    privacy_mode,
                    encrypted_fields: if encryption_level == 1 { Some(0x03) } else { None },
                    sensitive_data_hash: data_hash,
                    method: DivinationMethod::CoinMethod,
                    question_cid: None,
                    year_gz: Some(year_gz_val),
                    month_gz: Some(month_gz_val),
                    day_gz: Some(day_gz_val),
                    hour_gz: Some(hour_gz_val),
                    original_yaos: Some(original_yaos_arr),
                    original_inner: Some(original_inner),
                    original_outer: Some(original_outer),
                    original_name_idx: Some(original_name_idx),
                    gong: Some(gong),
                    gua_xu: Some(gua_xu),
                    has_bian_gua,
                    changed_yaos: Some(changed_yaos_arr),
                    changed_inner: Some(changed_inner),
                    changed_outer: Some(changed_outer),
                    changed_name_idx: Some(changed_name_idx),
                    hu_inner: Some(hu_inner),
                    hu_outer: Some(hu_outer),
                    hu_name_idx: Some(hu_name_idx),
                    gua_shen: Some(gua_shen),
                    moving_yaos: Some(moving_yaos),
                    xun_kong: Some(xun_kong),
                    fu_shen: Some(fu_shen),
                }
            };

            // 存储卦象
            Guas::<T>::insert(gua_id, gua);

            // 存储加密数据
            if let Some(enc_data) = encrypted_data {
                EncryptedDataStorage::<T>::insert(gua_id, enc_data);
            }
            if let Some(key_backup) = owner_key_backup {
                OwnerKeyBackupStorage::<T>::insert(gua_id, key_backup);
            }

            // 更新用户卦象列表
            UserGuas::<T>::try_mutate(&who, |list| {
                list.try_push(gua_id).map_err(|_| Error::<T>::UserGuaLimitExceeded)
            })?;

            // 更新每日计数
            Self::increment_daily_count(&who);

            // 更新用户统计
            UserStatsStorage::<T>::mutate(&who, |stats| {
                if stats.total_guas == 0 {
                    stats.first_gua_block = Self::block_to_day(<frame_system::Pallet<T>>::block_number());
                }
                stats.total_guas = stats.total_guas.saturating_add(1);
            });

            // 发出事件
            Self::deposit_event(Event::EncryptedGuaCreated {
                gua_id,
                creator: who,
                privacy_mode,
                method: DivinationMethod::CoinMethod,
            });

            Ok(())
        }

        /// 更新加密数据
        ///
        /// 仅卦象所有者可更新加密数据
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn update_encrypted_data(
            origin: OriginFor<T>,
            gua_id: u64,
            encrypted_data: BoundedVec<u8, T::MaxEncryptedLen>,
            data_hash: [u8; 32],
            owner_key_backup: [u8; 80],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查卦象存在且属于调用者
            let gua = Guas::<T>::get(gua_id).ok_or(Error::<T>::GuaNotFound)?;
            ensure!(gua.creator == who, Error::<T>::NotGuaOwner);

            // 更新加密数据
            EncryptedDataStorage::<T>::insert(gua_id, encrypted_data);
            OwnerKeyBackupStorage::<T>::insert(gua_id, owner_key_backup);

            // 更新数据哈希
            Guas::<T>::mutate(gua_id, |maybe_gua| {
                if let Some(gua) = maybe_gua {
                    gua.sensitive_data_hash = Some(data_hash);
                }
            });

            Self::deposit_event(Event::EncryptedDataUpdated { gua_id, data_hash });

            Ok(())
        }

        /// 删除卦象
        ///
        /// 删除卦象记录及其所有关联数据，并返还存储押金。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是卦象所有者）
        /// - `gua_id`: 卦象 ID
        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn delete_gua(
            origin: OriginFor<T>,
            gua_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取卦象记录并验证所有权
            let gua = Guas::<T>::get(gua_id)
                .ok_or(Error::<T>::GuaNotFound)?;
            ensure!(gua.creator == who, Error::<T>::NotGuaOwner);

            // 2. 从用户索引中移除
            UserGuas::<T>::mutate(&who, |guas| {
                guas.retain(|&id| id != gua_id);
            });

            // 4. 从公开列表中移除（如果是公开的）
            if gua.privacy_mode == pallet_divination_privacy::types::PrivacyMode::Public {
                PublicGuas::<T>::mutate(|list| {
                    list.retain(|&id| id != gua_id);
                });
            }

            // 5. 移除加密数据（如果有）
            EncryptedDataStorage::<T>::remove(gua_id);

            // 6. 移除密钥备份（如果有）
            OwnerKeyBackupStorage::<T>::remove(gua_id);

            // 7. 删除主卦象记录
            Guas::<T>::remove(gua_id);

            // 6. 发送删除事件
            Self::deposit_event(Event::GuaDeleted {
                gua_id,
                owner: who,
            });

            Ok(())
        }
    }

    // ========================================================================
    // 内部方法
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 检查每日起卦限制
        fn check_daily_limit(who: &T::AccountId) -> DispatchResult {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let day = Self::block_to_day(current_block);
            let count = DailyGuaCount::<T>::get((who, day));

            ensure!(count < T::MaxDailyGuas::get(), Error::<T>::DailyLimitExceeded);

            Ok(())
        }

        /// 增加每日计数
        fn increment_daily_count(who: &T::AccountId) {
            let current_block = <frame_system::Pallet<T>>::block_number();
            let day = Self::block_to_day(current_block);
            DailyGuaCount::<T>::mutate((who, day), |count| {
                *count = count.saturating_add(1);
            });
        }

        /// 区块号转天数
        fn block_to_day(block: BlockNumberFor<T>) -> u32 {
            // 假设 6 秒一个区块，14400 块 = 1 天
            let block_u32: u32 = block.try_into().unwrap_or(0);
            block_u32 / 14400
        }

        /// 执行排卦核心逻辑
        fn do_divine(
            who: &T::AccountId,
            yaos: [Yao; 6],
            method: DivinationMethod,
            year_gz: (TianGan, DiZhi),
            month_gz: (TianGan, DiZhi),
            day_gz: (TianGan, DiZhi),
            hour_gz: (TianGan, DiZhi),
        ) -> Result<u64, DispatchError> {
            // 检查用户存储上限
            let user_guas = UserGuas::<T>::get(who);
            ensure!(
                user_guas.len() < T::MaxUserGuas::get() as usize,
                Error::<T>::UserGuaLimitExceeded
            );

            // 获取新 ID
            let gua_id = NextGuaId::<T>::get();
            NextGuaId::<T>::put(gua_id + 1);

            // 计算内外卦
            let (original_inner, original_outer) = yaos_to_trigrams(&yaos);

            // 计算卦宫和世应
            let (gua_xu, gong) = calculate_shi_ying_gong(original_inner, original_outer);

            // 计算六神
            let liu_shen_array = calculate_liu_shen(day_gz.0);

            // 计算旬空
            let xun_kong = calculate_xun_kong(day_gz.0, day_gz.1);

            // 计算六十四卦索引
            let original_name_idx = calculate_gua_index(original_inner, original_outer);

            // 构建本卦六爻信息
            let gong_wx = gong.wu_xing();
            let mut original_yaos = [YaoInfo::default(); 6];
            let mut liu_qin_array = [LiuQin::XiongDi; 6];

            for i in 0..6 {
                let (gan, zhi) = if i < 3 {
                    get_inner_najia(original_inner, i as u8)
                } else {
                    get_outer_najia(original_outer, (i - 3) as u8)
                };

                let yao_wx = zhi.wu_xing();
                let liu_qin = LiuQin::from_wu_xing(gong_wx, yao_wx);
                liu_qin_array[i] = liu_qin;

                let shi_pos = gua_xu.shi_yao_pos() as usize;
                let ying_pos = gua_xu.ying_yao_pos() as usize;

                original_yaos[i] = YaoInfo {
                    yao: yaos[i],
                    tian_gan: gan,
                    di_zhi: zhi,
                    wu_xing: yao_wx,
                    liu_qin,
                    liu_shen: liu_shen_array[i],
                    is_shi: i + 1 == shi_pos,
                    is_ying: i + 1 == ying_pos,
                };
            }

            // 计算变卦
            let (changed_inner, changed_outer, has_bian_gua) = calculate_bian_gua(&yaos);
            let changed_name_idx = calculate_gua_index(changed_inner, changed_outer);

            // 构建变卦六爻信息
            let mut changed_yaos = [YaoInfo::default(); 6];
            if has_bian_gua {
                for i in 0..6 {
                    let (gan, zhi) = if i < 3 {
                        get_inner_najia(changed_inner, i as u8)
                    } else {
                        get_outer_najia(changed_outer, (i - 3) as u8)
                    };

                    let yao_wx = zhi.wu_xing();
                    // 变卦六亲仍按本卦卦宫计算
                    let liu_qin = LiuQin::from_wu_xing(gong_wx, yao_wx);

                    changed_yaos[i] = YaoInfo {
                        yao: if yaos[i].is_moving() {
                            if yaos[i].is_yang() { Yao::ShaoYin } else { Yao::ShaoYang }
                        } else {
                            yaos[i]
                        },
                        tian_gan: gan,
                        di_zhi: zhi,
                        wu_xing: yao_wx,
                        liu_qin,
                        liu_shen: liu_shen_array[i],
                        is_shi: false,
                        is_ying: false,
                    };
                }
            }

            // 计算动爻位图
            let moving_yaos = calculate_moving_bitmap(&yaos);

            // 计算互卦
            let (hu_inner, hu_outer) = calculate_hu_gua(&yaos);
            let hu_name_idx = calculate_gua_index(hu_inner, hu_outer);

            // 计算卦身
            let shi_pos = gua_xu.shi_yao_pos();
            let shi_is_yang = yaos[(shi_pos - 1) as usize].is_yang();
            let gua_shen = calculate_gua_shen(shi_pos, shi_is_yang);

            // 查找伏神
            let fu_shen = find_fu_shen(gong, &liu_qin_array);

            // 创建卦象（Public 模式，所有数据明文存储）
            let gua = LiuYaoGua {
                id: gua_id,
                creator: who.clone(),
                created_at: <frame_system::Pallet<T>>::block_number(),
                // 隐私控制字段
                privacy_mode: pallet_divination_privacy::types::PrivacyMode::Public,
                encrypted_fields: None,
                sensitive_data_hash: None,
                // 起卦信息
                method,
                question_cid: None,
                // 时间信息
                year_gz: Some(year_gz),
                month_gz: Some(month_gz),
                day_gz: Some(day_gz),
                hour_gz: Some(hour_gz),
                // 本卦信息
                original_yaos: Some(original_yaos),
                original_inner: Some(original_inner),
                original_outer: Some(original_outer),
                original_name_idx: Some(original_name_idx),
                gong: Some(gong),
                gua_xu: Some(gua_xu),
                // 变卦信息
                has_bian_gua,
                changed_yaos: Some(changed_yaos),
                changed_inner: Some(changed_inner),
                changed_outer: Some(changed_outer),
                changed_name_idx: Some(changed_name_idx),
                // 互卦信息
                hu_inner: Some(hu_inner),
                hu_outer: Some(hu_outer),
                hu_name_idx: Some(hu_name_idx),
                // 卦身
                gua_shen: Some(gua_shen),
                // 动爻
                moving_yaos: Some(moving_yaos),
                // 旬空
                xun_kong: Some(xun_kong),
                // 伏神
                fu_shen: Some(fu_shen),
            };

            // 存储卦象
            Guas::<T>::insert(gua_id, gua);

            // 更新用户卦象列表
            UserGuas::<T>::try_mutate(who, |list| {
                list.try_push(gua_id).map_err(|_| Error::<T>::UserGuaLimitExceeded)
            })?;

            // 更新用户统计
            UserStatsStorage::<T>::mutate(who, |stats| {
                if stats.total_guas == 0 {
                    stats.first_gua_block = Self::block_to_day(<frame_system::Pallet<T>>::block_number());
                }
                stats.total_guas = stats.total_guas.saturating_add(1);
            });

            Ok(gua_id)
        }

        // ====================================================================
        // Runtime API 辅助方法
        // ====================================================================

        /// 获取核心解卦结果（供 Runtime API 调用）
        ///
        /// # 参数
        /// - `gua_id`: 卦象 ID
        /// - `shi_xiang`: 占问事项类型（0-9）
        ///
        /// # 返回
        /// - `Some(LiuYaoCoreInterpretation)`: 核心解卦
        /// - `None`: 卦象不存在或无法解读（Private 模式）
        pub fn get_core_interpretation(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<crate::interpretation::LiuYaoCoreInterpretation> {
            let gua = Guas::<T>::get(gua_id)?;

            // 检查是否可解读
            if !gua.can_interpret() {
                return None;
            }

            // 转换事项类型
            let shi_xiang_type = match shi_xiang {
                0 => crate::interpretation::ShiXiangType::CaiYun,
                1 => crate::interpretation::ShiXiangType::ShiYe,
                2 => crate::interpretation::ShiXiangType::HunYin,
                3 => crate::interpretation::ShiXiangType::JianKang,
                4 => crate::interpretation::ShiXiangType::KaoShi,
                5 => crate::interpretation::ShiXiangType::GuanSi,
                6 => crate::interpretation::ShiXiangType::ChuXing,
                7 => crate::interpretation::ShiXiangType::XunRen,
                8 => crate::interpretation::ShiXiangType::TianQi,
                _ => crate::interpretation::ShiXiangType::QiTa,
            };

            // 获取当前区块号作为时间戳
            let block_number = <frame_system::Pallet<T>>::block_number();
            let timestamp: u32 = block_number.try_into().unwrap_or(0);

            // 计算并返回核心解卦
            Some(crate::interpretation::calculate_core_interpretation(
                &gua,
                shi_xiang_type,
                timestamp,
            ))
        }

        /// 获取完整解卦结果（供 Runtime API 调用）
        ///
        /// # 参数
        /// - `gua_id`: 卦象 ID
        /// - `shi_xiang`: 占问事项类型（0-9）
        ///
        /// # 返回
        /// - `Some(LiuYaoFullInterpretation)`: 完整解卦
        /// - `None`: 卦象不存在或无法解读（Private 模式）
        pub fn get_full_interpretation(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<crate::interpretation::LiuYaoFullInterpretation> {
            let gua = Guas::<T>::get(gua_id)?;

            // 检查是否可解读
            if !gua.can_interpret() {
                return None;
            }

            // 转换事项类型
            let shi_xiang_type = match shi_xiang {
                0 => crate::interpretation::ShiXiangType::CaiYun,
                1 => crate::interpretation::ShiXiangType::ShiYe,
                2 => crate::interpretation::ShiXiangType::HunYin,
                3 => crate::interpretation::ShiXiangType::JianKang,
                4 => crate::interpretation::ShiXiangType::KaoShi,
                5 => crate::interpretation::ShiXiangType::GuanSi,
                6 => crate::interpretation::ShiXiangType::ChuXing,
                7 => crate::interpretation::ShiXiangType::XunRen,
                8 => crate::interpretation::ShiXiangType::TianQi,
                _ => crate::interpretation::ShiXiangType::QiTa,
            };

            // 获取当前区块号作为时间戳
            let block_number = <frame_system::Pallet<T>>::block_number();
            let timestamp: u32 = block_number.try_into().unwrap_or(0);

            // 计算核心解卦
            let core = crate::interpretation::calculate_core_interpretation(
                &gua,
                shi_xiang_type,
                timestamp,
            );

            // 构建完整解卦
            let mut full = crate::interpretation::LiuYaoFullInterpretation::new(timestamp);
            full.core = core;

            // 填充卦象分析（需要解包 Option）
            let original_name_idx = gua.original_name_idx.unwrap_or(0);
            let changed_name_idx = gua.changed_name_idx.unwrap_or(0);
            let hu_name_idx = gua.hu_name_idx.unwrap_or(0);
            let gong = gua.gong.unwrap_or_default();
            let gua_xu = gua.gua_xu.unwrap_or_default();
            let gua_shen = gua.gua_shen.unwrap_or_default();
            let original_yaos = gua.original_yaos.unwrap_or_default();
            let changed_yaos = gua.changed_yaos.unwrap_or_default();
            let fu_shen = gua.fu_shen.unwrap_or_default();
            let day_gz = gua.day_gz.unwrap_or_default();
            let month_gz = gua.month_gz.unwrap_or_default();

            full.gua_xiang.ben_gua_idx = original_name_idx;
            full.gua_xiang.bian_gua_idx = if gua.has_bian_gua {
                changed_name_idx
            } else {
                255
            };
            full.gua_xiang.hu_gua_idx = hu_name_idx;
            full.gua_xiang.gong = gong.index();
            full.gua_xiang.gua_xu = gua_xu as u8;
            full.gua_xiang.shi_pos = gua_xu.shi_yao_pos() - 1;
            full.gua_xiang.ying_pos = gua_xu.ying_yao_pos() - 1;
            full.gua_xiang.gua_shen = gua_shen.index();

            // 判断六冲六合
            full.gua_xiang.is_liu_chong =
                crate::algorithm::is_liu_chong_by_index(original_name_idx);
            full.gua_xiang.is_liu_he =
                crate::algorithm::is_liu_he(original_name_idx);

            // 填充六亲分析
            for i in 0..6 {
                let qin = original_yaos[i].liu_qin;
                let state = full.liu_qin.get_qin_state_mut(qin);
                state.add_position(i as u8);
            }

            // 检查伏神
            for i in 0..6 {
                if let Some(fu) = &fu_shen[i] {
                    let state = full.liu_qin.get_qin_state_mut(fu.liu_qin);
                    state.has_fu_shen = true;
                    state.fu_shen_pos = fu.position;
                }
            }

            // 填充各爻分析
            let (kong1, kong2) = crate::algorithm::calculate_xun_kong(day_gz.0, day_gz.1);

            for i in 0..6 {
                let yao = if let Some(y) = full.get_yao_mut(i as u8) {
                    y
                } else {
                    continue;
                };

                let yao_info = &original_yaos[i];
                yao.position = i as u8;
                yao.is_kong = crate::interpretation::is_zhi_kong(yao_info.di_zhi, kong1, kong2);
                yao.is_yue_po =
                    crate::interpretation::is_zhi_yue_po(yao_info.di_zhi, month_gz.1);
                yao.is_ri_chong =
                    crate::interpretation::is_zhi_ri_chong(yao_info.di_zhi, day_gz.1);
                yao.is_dong = yao_info.yao.is_moving();

                if yao.is_dong && gua.has_bian_gua {
                    let changed_zhi = changed_yaos[i].di_zhi;
                    let hua = crate::interpretation::calculate_hua_type(
                        yao_info.di_zhi,
                        changed_zhi,
                        kong1,
                        kong2,
                    );
                    yao.set_hua_type(hua);
                }

                yao.wang_shuai = crate::interpretation::calculate_wang_shuai(
                    yao_info.di_zhi,
                    month_gz.1,
                    day_gz.1,
                );
            }

            Some(full)
        }

        /// 获取解卦文本索引列表（供 Runtime API 调用）
        ///
        /// # 参数
        /// - `gua_id`: 卦象 ID
        /// - `shi_xiang`: 占问事项类型（0-9）
        ///
        /// # 返回
        /// - `Some(Vec<JieGuaTextType>)`: 解卦文本索引列表
        /// - `None`: 卦象不存在或无法解读
        pub fn get_interpretation_texts(
            gua_id: u64,
            shi_xiang: u8,
        ) -> Option<sp_std::vec::Vec<crate::interpretation::JieGuaTextType>> {
            use crate::interpretation::JieGuaTextType;

            let core = Self::get_core_interpretation(gua_id, shi_xiang)?;
            let gua = Guas::<T>::get(gua_id)?;

            // 检查是否可解读
            if !gua.can_interpret() {
                return None;
            }

            let original_name_idx = gua.original_name_idx.unwrap_or(0);

            let mut texts = sp_std::vec::Vec::new();

            // 1. 吉凶总断
            let ji_xiong_text = match core.ji_xiong {
                crate::interpretation::JiXiongLevel::DaJi => JieGuaTextType::DaJiZongDuan,
                crate::interpretation::JiXiongLevel::Ji => JieGuaTextType::JiZongDuan,
                crate::interpretation::JiXiongLevel::XiaoJi => JieGuaTextType::XiaoJiZongDuan,
                crate::interpretation::JiXiongLevel::Ping => JieGuaTextType::PingZongDuan,
                crate::interpretation::JiXiongLevel::XiaoXiong => JieGuaTextType::XiaoXiongZongDuan,
                crate::interpretation::JiXiongLevel::Xiong => JieGuaTextType::XiongZongDuan,
                crate::interpretation::JiXiongLevel::DaXiong => JieGuaTextType::DaXiongZongDuan,
            };
            texts.push(ji_xiong_text);

            // 2. 用神状态
            let yong_shen_text = match core.yong_shen_state {
                crate::interpretation::YongShenState::WangXiang => JieGuaTextType::YongShenWangXiang,
                crate::interpretation::YongShenState::XiuQiu => JieGuaTextType::YongShenXiuQiu,
                crate::interpretation::YongShenState::DongHuaJin => JieGuaTextType::YongShenHuaJin,
                crate::interpretation::YongShenState::DongHuaTui => JieGuaTextType::YongShenHuaTui,
                crate::interpretation::YongShenState::DongHuaKong => JieGuaTextType::YongShenKong,
                crate::interpretation::YongShenState::FuCang => JieGuaTextType::YongShenFuCang,
                crate::interpretation::YongShenState::KongWang => JieGuaTextType::YongShenKong,
                crate::interpretation::YongShenState::RuMu => JieGuaTextType::YongShenRuMu,
                crate::interpretation::YongShenState::ShouKe => JieGuaTextType::YongShenShouKe,
                crate::interpretation::YongShenState::DeSheng => JieGuaTextType::YongShenDeSheng,
            };
            texts.push(yong_shen_text);

            // 3. 动爻断语
            match core.dong_yao_count {
                0 => texts.push(JieGuaTextType::WuDongYao),
                1 => texts.push(JieGuaTextType::YiYaoDuFa),
                6 => texts.push(JieGuaTextType::LiuYaoJieDong),
                _ => texts.push(JieGuaTextType::DuoYaoQiDong),
            }

            // 4. 特殊状态
            if crate::algorithm::is_liu_chong_by_index(original_name_idx) {
                texts.push(JieGuaTextType::GuaFengLiuChong);
            }
            if crate::algorithm::is_liu_he(original_name_idx) {
                texts.push(JieGuaTextType::GuaFengLiuHe);
            }

            // 5. 应期断语
            let ying_qi_text = match core.ying_qi {
                crate::interpretation::YingQiType::JinQi => JieGuaTextType::YingQiZaiRi,
                crate::interpretation::YingQiType::DuanQi => JieGuaTextType::YingQiZaiYue,
                crate::interpretation::YingQiType::ZhongQi => JieGuaTextType::YingQiZaiJi,
                crate::interpretation::YingQiType::ChangQi => JieGuaTextType::YingQiZaiNian,
                crate::interpretation::YingQiType::YuanQi => JieGuaTextType::YingQiZaiNian,
                crate::interpretation::YingQiType::BuQueDing => JieGuaTextType::YingQiDaiChong,
            };
            texts.push(ying_qi_text);

            Some(texts)
        }
    }

}
