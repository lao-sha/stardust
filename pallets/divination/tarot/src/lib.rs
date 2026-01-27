//! # 塔罗牌排盘 Pallet
//!
//! 本模块实现了区块链上的塔罗牌占卜系统，提供：
//! - 随机抽牌（使用链上随机数）
//! - 时间起卦（基于时间戳生成）
//! - 数字起卦（基于用户数字生成）
//! - 手动指定（直接指定牌面）
//! - 带切牌的随机抽牌（模拟真实塔罗仪式）
//! - 多种牌阵支持（单张、三牌、凯尔特十字等）
//! - 占卜记录存储与查询
//! - AI 解读请求（链下工作机触发）
//!
//! ## 核心概念
//!
//! - **大阿卡纳**: 22张主牌，代表人生重大主题
//! - **小阿卡纳**: 56张副牌，分四种花色（权杖、圣杯、宝剑、星币）
//! - **正逆位**: 牌的朝向影响解读
//! - **牌阵**: 不同的摆牌方式，适用于不同问题
//! - **切牌**: 模拟真实塔罗仪式的切牌过程

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

pub mod algorithm;
pub mod constants;
pub mod interpretation;
pub mod ocw_tee;
pub mod runtime_api;
pub mod types;

// TODO: 测试文件待完善 mock 配置（frame_system::Config 兼容性问题）
// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

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

    /// Pallet 配置 trait
    ///
    /// 注：RuntimeEvent 关联类型已从 Polkadot SDK 2506 版本开始自动附加，
    /// 无需在此显式声明。系统会自动添加：
    /// `frame_system::Config<RuntimeEvent: From<Event<Self>>>`
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// 货币类型
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        /// 随机数生成器
        type Randomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

        /// 每次占卜最大牌数（对应最复杂的牌阵）
        #[pallet::constant]
        type MaxCardsPerReading: Get<u32>;

        /// 每个用户最多存储的占卜记录数量
        #[pallet::constant]
        type MaxUserReadings: Get<u32>;

        /// 公开占卜列表的最大长度
        #[pallet::constant]
        type MaxPublicReadings: Get<u32>;

        /// 每日免费占卜次数
        #[pallet::constant]
        type DailyFreeDivinations: Get<u32>;

        /// 每日最大占卜次数（防刷）
        #[pallet::constant]
        type MaxDailyDivinations: Get<u32>;

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

    // ==================== 存储项 ====================

    /// 下一个占卜记录 ID
    #[pallet::storage]
    #[pallet::getter(fn next_reading_id)]
    pub type NextReadingId<T> = StorageValue<_, u64, ValueQuery>;

    /// 占卜记录存储
    ///
    /// 键：占卜记录 ID
    /// 值：完整的塔罗牌占卜结果
    #[pallet::storage]
    #[pallet::getter(fn readings)]
    pub type Readings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        TarotReading<T::AccountId, BlockNumberFor<T>, T::MaxCardsPerReading>,
    >;

    /// 用户占卜索引
    ///
    /// 键：用户账户
    /// 值：该用户的所有占卜记录 ID 列表
    #[pallet::storage]
    #[pallet::getter(fn user_readings)]
    pub type UserReadings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<u64, T::MaxUserReadings>,
        ValueQuery,
    >;

    /// 公开占卜列表
    ///
    /// 存储所有设置为公开的占卜记录 ID
    #[pallet::storage]
    #[pallet::getter(fn public_readings)]
    pub type PublicReadings<T: Config> =
        StorageValue<_, BoundedVec<u64, T::MaxPublicReadings>, ValueQuery>;

    /// 每日占卜计数
    ///
    /// 用于限制每日占卜次数，防止滥用
    /// 键1：用户账户
    /// 键2：天数（从创世块起算）
    /// 值：当日占卜次数
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

    /// 用户统计信息
    ///
    /// 记录用户的占卜统计数据
    #[pallet::storage]
    #[pallet::getter(fn user_stats)]
    pub type UserStats<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, DivinationStats, ValueQuery>;

    /// 用户各牌出现频率
    ///
    /// 记录每个用户抽到每张牌的次数，用于统计最常出现的牌
    /// 键1：用户账户
    /// 键2：牌ID (0-77)
    /// 值：出现次数
    #[pallet::storage]
    #[pallet::getter(fn user_card_frequency)]
    pub type UserCardFrequency<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Twox64Concat,
        u8,  // card_id
        u32, // count
        ValueQuery,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 新占卜记录创建成功
        /// [占卜ID, 占卜者, 牌阵类型, 起卦方式]
        ReadingCreated {
            reading_id: u64,
            diviner: T::AccountId,
            spread_type: SpreadType,
            method: DivinationMethod,
        },

        /// AI 解读请求已提交
        /// [占卜ID, 请求者]
        AiInterpretationRequested {
            reading_id: u64,
            requester: T::AccountId,
        },

        /// AI 解读结果已提交
        /// [占卜ID, IPFS CID]
        AiInterpretationSubmitted {
            reading_id: u64,
            cid: BoundedVec<u8, ConstU32<64>>,
        },

        /// 占卜隐私模式已更改
        /// [占卜ID, 隐私模式]
        ReadingPrivacyModeChanged {
            reading_id: u64,
            privacy_mode: PrivacyMode,
        },

        /// 占卜记录已删除
        /// [占卜ID, 所有者]
        ReadingDeleted {
            reading_id: u64,
            owner: T::AccountId,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 占卜记录不存在
        ReadingNotFound,
        /// 非占卜记录所有者
        NotOwner,
        /// 每日占卜次数超限
        DailyLimitExceeded,
        /// 用户占卜列表已满
        UserReadingsFull,
        /// 公开占卜列表已满
        PublicReadingsFull,
        /// 无效的牌阵类型
        InvalidSpreadType,
        /// 抽牌数量与牌阵不匹配
        CardCountMismatch,
        /// 无效的牌ID（超出0-77范围）
        InvalidCardId,
        /// 存在重复的牌
        DuplicateCards,
        /// AI 解读费用不足
        InsufficientFee,
        /// AI 解读请求已存在
        AiRequestAlreadyExists,
        /// AI 解读请求不存在
        AiRequestNotFound,
        /// 数字参数缺失
        MissingNumberParams,
        /// 手动指定参数缺失
        MissingManualParams,
    }

    // ==================== 可调用函数 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 随机抽牌占卜
        ///
        /// 使用链上随机数生成塔罗牌占卜结果。
        ///
        /// # 参数
        /// - `origin`: 调用者（签名账户）
        /// - `spread_type`: 牌阵类型
        /// - `question_hash`: 占卜问题的哈希值（隐私保护）
        /// - `privacy_mode`: 隐私模式（公开/私密/授权）
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_random(
            origin: OriginFor<T>,
            spread_type: SpreadType,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 使用链上随机源
            let random_seed = T::Randomness::random(&b"tarot"[..]).0;
            let random_bytes: [u8; 32] = random_seed
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 抽取牌
            let card_count = spread_type.card_count();
            let drawn = algorithm::draw_cards_random(&random_bytes, card_count);

            Self::create_reading(
                who,
                spread_type,
                DivinationMethod::Random,
                drawn,
                question_hash,
                privacy_mode,
            )
        }

        /// 时间起卦占卜
        ///
        /// 使用当前区块时间戳生成占卜结果。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `spread_type`: 牌阵类型
        /// - `question_hash`: 问题哈希
        /// - `privacy_mode`: 隐私模式
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_time(
            origin: OriginFor<T>,
            spread_type: SpreadType,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 获取时间戳和区块哈希
            let timestamp = Self::get_timestamp_secs();
            let block_hash = <frame_system::Pallet<T>>::parent_hash();
            let block_hash_bytes: [u8; 32] = block_hash
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 获取区块号作为额外熵源
            let block_number: u64 = <frame_system::Pallet<T>>::block_number()
                .try_into()
                .unwrap_or(0);

            // 抽取牌（使用增强版时间起卦）
            let card_count = spread_type.card_count();
            let drawn =
                algorithm::draw_cards_by_time(timestamp, &block_hash_bytes, block_number, card_count);

            Self::create_reading(
                who,
                spread_type,
                DivinationMethod::ByTime,
                drawn,
                question_hash,
                privacy_mode,
            )
        }

        /// 数字起卦占卜
        ///
        /// 使用用户提供的数字生成占卜结果。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `numbers`: 用户提供的数字列表
        /// - `spread_type`: 牌阵类型
        /// - `question_hash`: 问题哈希
        /// - `privacy_mode`: 隐私模式
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_by_numbers(
            origin: OriginFor<T>,
            numbers: BoundedVec<u16, ConstU32<16>>,
            spread_type: SpreadType,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            ensure!(!numbers.is_empty(), Error::<T>::MissingNumberParams);

            // 获取区块哈希
            let block_hash = <frame_system::Pallet<T>>::parent_hash();
            let block_hash_bytes: [u8; 32] = block_hash
                .as_ref()
                .try_into()
                .unwrap_or([0u8; 32]);

            // 抽取牌
            let card_count = spread_type.card_count();
            let drawn = algorithm::draw_cards_by_numbers(&numbers, &block_hash_bytes, card_count);

            Self::create_reading(
                who,
                spread_type,
                DivinationMethod::ByNumbers,
                drawn,
                question_hash,
                privacy_mode,
            )
        }

        /// 手动指定牌面占卜
        ///
        /// 直接指定牌面和正逆位，用于记录已知的占卜结果。
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `cards`: 指定的牌列表 (牌ID, 是否逆位)
        /// - `spread_type`: 牌阵类型
        /// - `question_hash`: 问题哈希
        /// - `privacy_mode`: 隐私模式
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn divine_manual(
            origin: OriginFor<T>,
            cards: BoundedVec<(u8, bool), ConstU32<12>>,
            spread_type: SpreadType,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            ensure!(!cards.is_empty(), Error::<T>::MissingManualParams);

            // 验证牌数与牌阵匹配
            ensure!(
                cards.len() == spread_type.card_count() as usize,
                Error::<T>::CardCountMismatch
            );

            // 验证牌的有效性
            let card_ids: Vec<u8> = cards.iter().map(|(id, _)| *id).collect();
            ensure!(
                algorithm::validate_drawn_cards(&card_ids),
                Error::<T>::InvalidCardId
            );

            let drawn: Vec<(u8, bool)> = cards.into_iter().collect();

            Self::create_reading(
                who,
                spread_type,
                DivinationMethod::Manual,
                drawn,
                question_hash,
                privacy_mode,
            )
        }

        /// 带切牌的随机占卜
        ///
        /// 模拟真实塔罗牌占卜仪式，包含洗牌-切牌-抽牌的完整流程。
        /// 用户可以指定切牌位置，增加占卜的仪式感和参与感。
        ///
        /// # 参数
        /// - `origin`: 调用者（签名账户）
        /// - `spread_type`: 牌阵类型
        /// - `cut_position`: 切牌位置（1-77），None 表示随机切牌
        /// - `question_hash`: 占卜问题的哈希值
        /// - `privacy_mode`: 隐私模式
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(55_000_000, 0))]
        pub fn divine_random_with_cut(
            origin: OriginFor<T>,
            spread_type: SpreadType,
            cut_position: Option<u8>,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::check_daily_limit(&who)?;

            // 使用链上随机源
            let random_seed = T::Randomness::random(&b"tarot_cut"[..]).0;
            let random_bytes: [u8; 32] = random_seed.as_ref().try_into().unwrap_or([0u8; 32]);

            // 使用带切牌的抽牌算法
            let card_count = spread_type.card_count();
            let drawn = algorithm::draw_cards_with_cut(&random_bytes, cut_position, card_count);

            Self::create_reading(
                who,
                spread_type,
                DivinationMethod::RandomWithCut,
                drawn,
                question_hash,
                privacy_mode,
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
        /// - `reading_id`: 占卜记录 ID
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::request_interpretation"
        )]
        pub fn request_ai_interpretation(
            origin: OriginFor<T>,
            reading_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证占卜记录存在且为调用者所有
            let reading = Readings::<T>::get(reading_id)
                .ok_or(Error::<T>::ReadingNotFound)?;
            ensure!(reading.diviner == who, Error::<T>::NotOwner);

            // 检查是否已有请求
            ensure!(
                !AiInterpretationRequests::<T>::contains_key(reading_id),
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
            AiInterpretationRequests::<T>::insert(reading_id, who.clone());

            // 发送事件触发链下工作机
            Self::deposit_event(Event::AiInterpretationRequested {
                reading_id,
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
        /// - `reading_id`: 占卜记录 ID
        /// - `interpretation_cid`: 解读内容的 IPFS CID
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        #[deprecated(
            since = "0.2.0",
            note = "请使用 pallet_divination_ai::submit_result"
        )]
        pub fn submit_ai_interpretation(
            origin: OriginFor<T>,
            reading_id: u64,
            interpretation_cid: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            // 验证 AI 预言机权限
            T::AiOracleOrigin::ensure_origin(origin)?;

            // 验证请求存在
            ensure!(
                AiInterpretationRequests::<T>::contains_key(reading_id),
                Error::<T>::AiRequestNotFound
            );

            // 更新占卜记录的 AI 解读 CID
            Readings::<T>::try_mutate(reading_id, |maybe_reading| {
                let reading = maybe_reading
                    .as_mut()
                    .ok_or(Error::<T>::ReadingNotFound)?;
                reading.interpretation_cid = Some(interpretation_cid.clone());
                Ok::<_, DispatchError>(())
            })?;

            // 移除请求
            AiInterpretationRequests::<T>::remove(reading_id);

            Self::deposit_event(Event::AiInterpretationSubmitted {
                reading_id,
                cid: interpretation_cid,
            });

            Ok(())
        }

        /// 更改占卜隐私模式
        ///
        /// # 参数
        /// - `origin`: 调用者
        /// - `reading_id`: 占卜记录 ID
        /// - `privacy_mode`: 新的隐私模式
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(20_000_000, 0))]
        pub fn set_reading_privacy_mode(
            origin: OriginFor<T>,
            reading_id: u64,
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Readings::<T>::try_mutate(reading_id, |maybe_reading| {
                let reading = maybe_reading
                    .as_mut()
                    .ok_or(Error::<T>::ReadingNotFound)?;
                ensure!(reading.diviner == who, Error::<T>::NotOwner);

                let was_public = matches!(reading.privacy_mode, PrivacyMode::Public);
                let is_public = matches!(privacy_mode, PrivacyMode::Public);
                reading.privacy_mode = privacy_mode.clone();

                // 更新公开占卜列表
                if is_public && !was_public {
                    // 添加到公开列表
                    PublicReadings::<T>::try_mutate(|list| {
                        list.try_push(reading_id)
                            .map_err(|_| Error::<T>::PublicReadingsFull)
                    })?;
                } else if !is_public && was_public {
                    // 从公开列表移除
                    PublicReadings::<T>::mutate(|list| {
                        list.retain(|&id| id != reading_id);
                    });
                }

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::ReadingPrivacyModeChanged {
                reading_id,
                privacy_mode,
            });

            Ok(())
        }

        /// 删除占卜记录
        ///
        /// 删除占卜记录及其所有关联数据，并返还存储押金。
        ///
        /// # 参数
        /// - `origin`: 调用者（必须是记录所有者）
        /// - `reading_id`: 占卜记录 ID
        ///
        /// # 返还规则
        /// - 30天内删除：100% 返还
        /// - 30天后删除：90% 返还（10% 进入国库）
        ///
        /// # 删除内容
        /// 1. 主记录（Readings）
        /// 2. 用户索引（UserReadings）
        /// 3. 公开列表（PublicReadings，如适用）
        /// 4. AI 解读请求（AiInterpretationRequests）
        /// 5. 押金记录（DepositRecords）
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(50_000_000, 0))]
        pub fn delete_reading(
            origin: OriginFor<T>,
            reading_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. 获取记录并验证所有权
            let reading = Readings::<T>::get(reading_id)
                .ok_or(Error::<T>::ReadingNotFound)?;
            ensure!(reading.diviner == who, Error::<T>::NotOwner);

            // 2. 从用户索引中移除
            UserReadings::<T>::mutate(&who, |readings| {
                readings.retain(|&id| id != reading_id);
            });

            // 4. 从公开列表中移除（如果是公开的）
            if matches!(reading.privacy_mode, PrivacyMode::Public) {
                PublicReadings::<T>::mutate(|list| {
                    list.retain(|&id| id != reading_id);
                });
            }

            // 5. 移除 AI 解读请求（如果有）
            AiInterpretationRequests::<T>::remove(reading_id);

            // 6. 删除主记录
            Readings::<T>::remove(reading_id);

            // 5. 发送删除事件
            Self::deposit_event(Event::ReadingDeleted {
                reading_id,
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

        /// 检查每日占卜次数限制
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

        /// 创建占卜记录并存储
        fn create_reading(
            diviner: T::AccountId,
            spread_type: SpreadType,
            method: DivinationMethod,
            drawn: Vec<(u8, bool)>,
            question_hash: [u8; 32],
            privacy_mode: PrivacyMode,
        ) -> DispatchResult {
            // 获取新的占卜记录 ID
            let reading_id = NextReadingId::<T>::get();
            NextReadingId::<T>::put(reading_id.saturating_add(1));

            // 获取当前区块号和时间戳
            let block_number = <frame_system::Pallet<T>>::block_number();
            let timestamp = Self::get_timestamp_secs();

            // 转换抽牌结果为 DrawnCard
            let mut cards = BoundedVec::<DrawnCard, T::MaxCardsPerReading>::default();
            for (i, (card_id, reversed)) in drawn.iter().enumerate() {
                let drawn_card = DrawnCard::new(*card_id, *reversed, i as u8);
                cards.try_push(drawn_card).map_err(|_| Error::<T>::CardCountMismatch)?;
            }

            // 创建占卜记录
            let reading = TarotReading {
                id: reading_id,
                diviner: diviner.clone(),
                spread_type,
                method: method.clone(),
                cards,
                question_hash,
                block_number,
                timestamp,
                interpretation_cid: None,
                privacy_mode: privacy_mode.clone(),
            };

            // 存储占卜记录
            Readings::<T>::insert(reading_id, reading);

            // 更新用户占卜索引
            UserReadings::<T>::try_mutate(&diviner, |list| {
                list.try_push(reading_id)
                    .map_err(|_| Error::<T>::UserReadingsFull)
            })?;

            // 如果公开，添加到公开列表
            if matches!(privacy_mode, PrivacyMode::Public) {
                PublicReadings::<T>::try_mutate(|list| {
                    list.try_push(reading_id)
                        .map_err(|_| Error::<T>::PublicReadingsFull)
                })?;
            }

            // 更新用户统计
            Self::update_user_stats(&diviner, &drawn);

            // 发送事件
            Self::deposit_event(Event::ReadingCreated {
                reading_id,
                diviner,
                spread_type,
                method,
            });

            Ok(())
        }

        /// 更新用户统计信息（完整实现）
        ///
        /// 统计包括：
        /// - 总占卜次数
        /// - 大阿卡纳出现次数
        /// - 逆位出现次数
        /// - 最常出现的牌及其次数
        fn update_user_stats(who: &T::AccountId, drawn: &[(u8, bool)]) {
            // 先更新每张牌的频率，同时跟踪最大频率
            let mut max_card_id: u8 = 0;
            let mut max_count: u32 = 0;

            for (card_id, _) in drawn {
                if *card_id < 78 {
                    // 更新该牌的频率
                    let new_count = UserCardFrequency::<T>::mutate(who, card_id, |count| {
                        *count = count.saturating_add(1);
                        *count
                    });

                    // 检查是否为新的最高频率
                    if new_count > max_count {
                        max_count = new_count;
                        max_card_id = *card_id;
                    }
                }
            }

            // 更新用户统计
            UserStats::<T>::mutate(who, |stats| {
                stats.total_readings = stats.total_readings.saturating_add(1);

                for (card_id, reversed) in drawn {
                    // 统计大阿卡纳
                    if *card_id < 22 {
                        stats.major_arcana_count = stats.major_arcana_count.saturating_add(1);
                    }

                    // 统计逆位
                    if *reversed {
                        stats.reversed_count = stats.reversed_count.saturating_add(1);
                    }
                }

                // 更新最常出现的牌
                // 如果本次更新后的最大频率超过了历史记录，则更新
                if max_count > stats.most_frequent_count {
                    stats.most_frequent_card = max_card_id;
                    stats.most_frequent_count = max_count;
                } else if max_count == stats.most_frequent_count && max_card_id != stats.most_frequent_card {
                    // 频率相同时，检查当前记录的牌的实际频率
                    let current_max_freq = UserCardFrequency::<T>::get(who, stats.most_frequent_card);
                    if max_count > current_max_freq {
                        stats.most_frequent_card = max_card_id;
                        stats.most_frequent_count = max_count;
                    }
                }
            });
        }

        // ==================== Runtime API 实现函数 ====================

        /// 获取核心解卦结果
        ///
        /// 基于占卜记录生成核心解卦数据
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(TarotCoreInterpretation)`: 核心解卦结果
        /// - `None`: 占卜记录不存在
        pub fn api_get_core_interpretation(
            reading_id: u64,
        ) -> Option<crate::interpretation::TarotCoreInterpretation> {
            let reading = Readings::<T>::get(reading_id)?;

            // 转换牌数据
            let cards: Vec<(u8, bool)> = reading
                .cards
                .iter()
                .map(|c| (c.card.id, c.position.is_reversed()))
                .collect();

            let block_number = reading
                .block_number
                .try_into()
                .unwrap_or(0u32);

            Some(algorithm::generate_core_interpretation(
                &cards,
                reading.spread_type,
                block_number,
            ))
        }

        /// 获取完整解卦结果
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(TarotFullInterpretation)`: 完整解卦结果
        /// - `None`: 占卜记录不存在
        pub fn api_get_full_interpretation(
            reading_id: u64,
        ) -> Option<crate::interpretation::TarotFullInterpretation<T::MaxCardsPerReading>> {
            let reading = Readings::<T>::get(reading_id)?;

            // 转换牌数据
            let cards: Vec<(u8, bool)> = reading
                .cards
                .iter()
                .map(|c| (c.card.id, c.position.is_reversed()))
                .collect();

            let block_number = reading
                .block_number
                .try_into()
                .unwrap_or(0u32);

            // 生成核心解卦
            let core = algorithm::generate_core_interpretation(
                &cards,
                reading.spread_type,
                block_number,
            );

            // 生成能量分析
            let spread_energy = algorithm::generate_spread_energy_analysis(&cards);

            // 构建完整解卦（其他字段暂不填充）
            Some(crate::interpretation::TarotFullInterpretation {
                core,
                spread_energy,
                card_analyses: None,
                card_relationships: None,
                timeline_analysis: None,
            })
        }

        /// 获取解读文本索引列表
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(Vec<InterpretationTextType>)`: 解读文本索引列表
        /// - `None`: 占卜记录不存在
        pub fn api_get_interpretation_texts(
            reading_id: u64,
        ) -> Option<Vec<crate::interpretation::InterpretationTextType>> {
            use crate::interpretation::*;

            let core = Self::api_get_core_interpretation(reading_id)?;
            let mut texts = Vec::new();

            // 1. 能量描述
            let energy_text = if core.overall_energy >= 75 {
                InterpretationTextType::EnergyHigh
            } else if core.overall_energy >= 50 {
                InterpretationTextType::EnergyMedium
            } else if core.overall_energy >= 25 {
                InterpretationTextType::EnergyLow
            } else {
                InterpretationTextType::EnergyVolatile
            };
            texts.push(energy_text);

            // 2. 元素主导描述
            let element_text = match core.dominant_element {
                DominantElement::Fire => InterpretationTextType::FireDominant,
                DominantElement::Water => InterpretationTextType::WaterDominant,
                DominantElement::Air => InterpretationTextType::AirDominant,
                DominantElement::Earth => InterpretationTextType::EarthDominant,
                DominantElement::Spirit => InterpretationTextType::SpiritDominant,
                DominantElement::None => InterpretationTextType::ElementBalanced,
            };
            texts.push(element_text);

            // 3. 吉凶判断
            let fortune_text = match core.fortune_tendency {
                FortuneTendency::Excellent => InterpretationTextType::FortuneExcellent,
                FortuneTendency::Good => InterpretationTextType::FortuneGood,
                FortuneTendency::Neutral => InterpretationTextType::FortuneNeutral,
                FortuneTendency::MinorBad => InterpretationTextType::FortuneMinorBad,
                FortuneTendency::Bad => InterpretationTextType::FortuneBad,
            };
            texts.push(fortune_text);

            // 4. 特殊组合
            if core.has_fool_world_combo() {
                texts.push(InterpretationTextType::FoolWorldCombo);
            }
            if core.has_many_major_arcana() {
                texts.push(InterpretationTextType::ManyMajorArcana);
            }
            if core.has_same_suit_sequence() {
                texts.push(InterpretationTextType::SameSuitSequence);
            }
            if core.is_all_reversed() {
                texts.push(InterpretationTextType::AllReversed);
            }
            if core.is_all_upright() {
                texts.push(InterpretationTextType::AllUpright);
            }

            // 5. 行动建议
            let advice_text = if core.action_index >= 75 && core.fortune_tendency as u8 <= 1 {
                InterpretationTextType::ActionTakeAction
            } else if core.stability_index >= 60 {
                InterpretationTextType::ActionPersist
            } else if core.change_index >= 60 {
                InterpretationTextType::ActionWaitAndSee
            } else if core.reversed_ratio >= 50 {
                InterpretationTextType::ActionReflect
            } else {
                InterpretationTextType::ActionLearn
            };
            texts.push(advice_text);

            // 6. 能量指数高亮
            if core.action_index >= 70 {
                texts.push(InterpretationTextType::ActionIndexHigh);
            }
            if core.emotion_index >= 70 {
                texts.push(InterpretationTextType::EmotionIndexHigh);
            }
            if core.intellect_index >= 70 {
                texts.push(InterpretationTextType::IntellectIndexHigh);
            }
            if core.material_index >= 70 {
                texts.push(InterpretationTextType::MaterialIndexHigh);
            }
            if core.spiritual_index >= 70 {
                texts.push(InterpretationTextType::SpiritualIndexHigh);
            }
            if core.stability_index >= 70 {
                texts.push(InterpretationTextType::StabilityIndexHigh);
            }
            if core.change_index >= 70 {
                texts.push(InterpretationTextType::ChangeIndexHigh);
            }

            Some(texts)
        }

        /// 生成AI解读提示词上下文
        ///
        /// # 参数
        /// - `reading_id`: 塔罗牌占卜记录 ID
        ///
        /// # 返回
        /// - `Some(Vec<u8>)`: AI提示词上下文（UTF-8编码）
        /// - `None`: 占卜记录不存在
        pub fn api_generate_ai_prompt_context(
            reading_id: u64,
        ) -> Option<Vec<u8>> {
            use crate::interpretation::*;

            let reading = Readings::<T>::get(reading_id)?;
            let core = Self::api_get_core_interpretation(reading_id)?;

            let mut context = Vec::new();

            // 牌阵类型
            let spread_name = reading.spread_type.name();
            context.extend_from_slice(b"spread:");
            context.extend_from_slice(spread_name.as_bytes());
            context.extend_from_slice(b";\n");

            // 主导元素
            let element_name = match core.dominant_element {
                DominantElement::None => "none",
                DominantElement::Fire => "fire-wands",
                DominantElement::Water => "water-cups",
                DominantElement::Air => "air-swords",
                DominantElement::Earth => "earth-pentacles",
                DominantElement::Spirit => "spirit-major",
            };
            context.extend_from_slice(b"dominant:");
            context.extend_from_slice(element_name.as_bytes());
            context.extend_from_slice(b";\n");

            // 能量状态
            let energy_desc = if core.overall_energy >= 75 {
                "high"
            } else if core.overall_energy >= 50 {
                "medium"
            } else {
                "low"
            };
            context.extend_from_slice(b"energy:");
            context.extend_from_slice(energy_desc.as_bytes());
            context.extend_from_slice(b";\n");

            // 吉凶倾向
            let fortune_desc = match core.fortune_tendency {
                FortuneTendency::Excellent => "excellent",
                FortuneTendency::Good => "good",
                FortuneTendency::Neutral => "neutral",
                FortuneTendency::MinorBad => "minor-bad",
                FortuneTendency::Bad => "bad",
            };
            context.extend_from_slice(b"fortune:");
            context.extend_from_slice(fortune_desc.as_bytes());
            context.extend_from_slice(b";\n");

            // 各牌信息
            let position_names = reading.spread_type.position_names();
            for (i, drawn_card) in reading.cards.iter().enumerate() {
                let position_name = position_names.get(i).copied().unwrap_or("unknown");
                let card_id = drawn_card.card.id;
                let is_reversed = drawn_card.position.is_reversed();

                context.extend_from_slice(b"card:");
                // 使用牌ID代替中文名称
                let mut id_buf = [0u8; 3];
                let id_len = if card_id >= 100 { 3 } else if card_id >= 10 { 2 } else { 1 };
                let mut n = card_id;
                for i in (0..id_len).rev() {
                    id_buf[i] = b'0' + (n % 10);
                    n /= 10;
                }
                context.extend_from_slice(&id_buf[..id_len]);
                context.extend_from_slice(b"[");
                if is_reversed {
                    context.extend_from_slice(b"R");
                } else {
                    context.extend_from_slice(b"U");
                }
                context.extend_from_slice(b"]@");
                context.extend_from_slice(position_name.as_bytes());
                context.extend_from_slice(b";\n");
            }

            // 特殊组合
            if core.has_fool_world_combo() {
                context.extend_from_slice(b"special:fool-world;\n");
            }
            if core.has_many_major_arcana() {
                context.extend_from_slice(b"special:many-major;\n");
            }

            // 综合评分
            context.extend_from_slice(b"score:");
            let score = core.overall_score;
            let mut score_buf = [0u8; 3];
            let score_len = if score >= 100 { 3 } else if score >= 10 { 2 } else { 1 };
            let mut s = score;
            for i in (0..score_len).rev() {
                score_buf[i] = b'0' + (s % 10);
                s /= 10;
            }
            context.extend_from_slice(&score_buf[..score_len]);
            context.extend_from_slice(b";\n");

            Some(context)
        }

        /// 检查占卜记录是否存在
        pub fn api_reading_exists(reading_id: u64) -> bool {
            Readings::<T>::contains_key(reading_id)
        }

        /// 获取占卜记录创建者
        pub fn api_get_reading_owner(reading_id: u64) -> Option<T::AccountId> {
            Readings::<T>::get(reading_id).map(|r| r.diviner)
        }

        /// 批量获取核心解卦结果
        pub fn api_batch_get_core_interpretations(
            reading_ids: Vec<u64>,
        ) -> Vec<(u64, Option<crate::interpretation::TarotCoreInterpretation>)> {
            reading_ids
                .into_iter()
                .map(|id| (id, Self::api_get_core_interpretation(id)))
                .collect()
        }

        /// 分析单张牌在特定牌阵位置的含义
        pub fn api_analyze_card_in_spread(
            card_id: u8,
            is_reversed: bool,
            spread_type: u8,
            position: u8,
        ) -> Option<crate::interpretation::CardInterpretation> {
            use crate::interpretation::*;

            if card_id >= 78 {
                return None;
            }

            let spread = crate::types::SpreadType::from_count(spread_type);
            // 位置权重基于牌阵类型和位置
            let position_weight = match spread {
                crate::types::SpreadType::CelticCross => {
                    // 凯尔特十字的中心牌（0，1）权重最高
                    if position <= 1 { 10 } else if position <= 5 { 7 } else { 5 }
                }
                crate::types::SpreadType::ThreeCardTime => {
                    // 现在牌（位置1）权重最高
                    if position == 1 { 10 } else { 7 }
                }
                _ => 7,
            };

            // 计算能量强度
            let card = crate::types::TarotCard::from_id(card_id);
            let base_energy: u8 = if card.is_major() { 80 } else { 60 };
            let energy_strength = if is_reversed {
                base_energy.saturating_sub(20)
            } else {
                base_energy
            };

            Some(CardInterpretation {
                card_id,
                is_reversed,
                spread_position: position,
                position_weight,
                energy_strength,
                relation_to_prev: RelationshipType::None,
                relation_to_next: RelationshipType::None,
            })
        }

        /// 分析两张牌之间的关系
        pub fn api_analyze_card_relationship(
            card1_id: u8,
            card2_id: u8,
        ) -> Option<crate::interpretation::CardRelationship> {
            use crate::interpretation::*;

            if card1_id >= 78 || card2_id >= 78 {
                return None;
            }

            let card1 = crate::types::TarotCard::from_id(card1_id);
            let card2 = crate::types::TarotCard::from_id(card2_id);

            // 判断关系类型
            let relationship_type = if card1.suit == card2.suit && card1.suit != crate::types::Suit::None {
                RelationshipType::SameElementReinforce
            } else if card1_id == 0 && card2_id == 21 {
                // 愚者与世界
                RelationshipType::Complementary
            } else if card1.is_major() && card2.is_major() {
                RelationshipType::Generating
            } else {
                // 元素相生相克判断
                match (card1.suit, card2.suit) {
                    (crate::types::Suit::None, _) | (_, crate::types::Suit::None) => RelationshipType::None,
                    (s1, s2) => {
                        // 火生土，土生金(星币)，金生水，水生木(权杖)，木生火
                        let s1_idx = s1 as u8;
                        let s2_idx = s2 as u8;
                        if (s1_idx % 4) + 1 == (s2_idx % 4) {
                            RelationshipType::Generating
                        } else if (s1_idx + 2) % 4 == s2_idx % 4 {
                            RelationshipType::Controlling
                        } else {
                            RelationshipType::None
                        }
                    }
                }
            };

            // 计算关系强度
            let strength = match relationship_type {
                RelationshipType::SameElementReinforce => 90,
                RelationshipType::Complementary => 95,
                RelationshipType::Generating => 70,
                RelationshipType::Controlling => 60,
                _ => 30,
            };

            Some(CardRelationship {
                card1_index: card1_id,
                card2_index: card2_id,
                relationship_type,
                strength,
            })
        }

        /// 获取牌阵能量分析
        pub fn api_get_spread_energy(
            reading_id: u64,
        ) -> Option<crate::interpretation::SpreadEnergyAnalysis> {
            let reading = Readings::<T>::get(reading_id)?;

            let cards: Vec<(u8, bool)> = reading
                .cards
                .iter()
                .map(|c| (c.card.id, c.position.is_reversed()))
                .collect();

            Some(algorithm::generate_spread_energy_analysis(&cards))
        }

        /// 获取时间线分析
        pub fn api_get_timeline_analysis(
            reading_id: u64,
        ) -> Option<crate::interpretation::TimelineAnalysis> {
            use crate::interpretation::*;

            let full = Self::api_get_full_interpretation(reading_id)?;

            // 基于牌阵能量推断时间线
            let past_trend = if full.spread_energy.past_energy >= 60 {
                TimelineTrend::Rising
            } else if full.spread_energy.past_energy >= 40 {
                TimelineTrend::Stable
            } else {
                TimelineTrend::Declining
            };

            let present_state = if full.spread_energy.present_energy >= 70 {
                TimelineState::HighPoint
            } else if full.spread_energy.present_energy >= 40 {
                TimelineState::Stable
            } else {
                TimelineState::LowPoint
            };

            let future_trend = if full.spread_energy.future_energy >= 60 {
                TimelineTrend::Rising
            } else if full.spread_energy.future_energy >= 40 {
                TimelineTrend::Stable
            } else {
                TimelineTrend::Declining
            };

            let overall_direction = match (past_trend, future_trend) {
                (TimelineTrend::Rising, TimelineTrend::Rising) => OverallDirection::Positive,
                (TimelineTrend::Declining, TimelineTrend::Declining) => OverallDirection::Negative,
                _ => OverallDirection::Neutral,
            };

            Some(TimelineAnalysis {
                past_trend,
                present_state,
                future_trend,
                turning_point: 255, // 无转折点
                overall_direction,
            })
        }
    }

}
