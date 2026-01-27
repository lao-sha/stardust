//! # 婚恋模块 - 用户资料管理
//!
//! 本模块提供用户资料的创建、更新和查询功能。
//!
//! ## 功能概述
//!
//! - **资料创建**：创建用户婚恋资料
//! - **资料更新**：更新个人信息、择偶条件
//! - **资料查询**：查询用户资料
//! - **隐私设置**：控制资料可见性
//! - **八字绑定**：绑定八字命盘用于合婚
//!
//! ## 使用流程
//!
//! 1. 用户创建婚恋资料
//! 2. 设置择偶条件
//! 3. 绑定八字命盘（可选）
//! 4. 设置隐私模式

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

// TODO: 测试文件待完善 mock 配置
// #[cfg(test)]
// mod tests;

use frame_support::pallet_prelude::*;
use frame_support::traits::fungible::{Inspect, Mutate, MutateHold};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Saturating, Zero};
use pallet_matchmaking_common::{
    Gender, ProfilePrivacyMode, EducationLevel, BirthDate, BirthTime,
    PropertyStatus, VehicleStatus, MaritalStatus, Lifestyle, PersonalityTrait,
    ProfileStatus, CompatibilityPreferences, FieldPrivacySettings,
    BaziPersonalityTrait, PersonalitySource,
};
use pallet_trading_common::PricingProvider;
use pallet_affiliate::types::AffiliateDistributor;
use pallet_storage_service::{IpfsPinner, types::{SubjectType, PinTier}};

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 运行时事件类型
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 昵称最大长度
        #[pallet::constant]
        type MaxNicknameLen: Get<u32>;

        /// 位置最大长度
        #[pallet::constant]
        type MaxLocationLen: Get<u32>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 简介最大长度
        #[pallet::constant]
        type MaxBioLen: Get<u32>;

        /// 描述最大长度
        #[pallet::constant]
        type MaxDescLen: Get<u32>;

        /// 职业最大长度
        #[pallet::constant]
        type MaxOccupationLen: Get<u32>;

        /// 性格特征最大数量
        #[pallet::constant]
        type MaxTraits: Get<u32>;

        /// 兴趣爱好最大数量
        #[pallet::constant]
        type MaxHobbies: Get<u32>;

        /// 单个兴趣最大长度
        #[pallet::constant]
        type MaxHobbyLen: Get<u32>;

        /// 权重信息
        type WeightInfo: WeightInfo;

        // ========== 保证金相关配置 ==========
        
        /// Fungible 接口：用于锁定和释放保证金
        type Fungible: Inspect<Self::AccountId, Balance = BalanceOf<Self>>
            + Mutate<Self::AccountId>
            + MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>;
        
        /// RuntimeHoldReason：保证金锁定原因标识
        type RuntimeHoldReason: From<HoldReason>;
        
        /// 保证金兆底金额（DUST数量，pricing不可用时使用）
        #[pallet::constant]
        type ProfileDeposit: Get<BalanceOf<Self>>;
        
        /// 保证金USD价值（精度10^6，50_000_000 = 50 USDT）
        #[pallet::constant]
        type ProfileDepositUsd: Get<u64>;
        
        /// 月费兆底金额（DUST数量，pricing不可用时使用）
        #[pallet::constant]
        type MonthlyFee: Get<BalanceOf<Self>>;
        
        /// 月费USD价值（精度10^6，2_000_000 = 2 USDT）
        #[pallet::constant]
        type MonthlyFeeUsd: Get<u64>;
        
        /// 定价接口（用于换算保证金和月费）
        type Pricing: pallet_trading_common::PricingProvider<BalanceOf<Self>>;
        
        /// 国库账户（月费和罚没资金转入）
        type TreasuryAccount: Get<Self::AccountId>;
        
        /// 销毁账户
        type BurnAccount: Get<Self::AccountId>;
        
        /// 存储账户
        type StorageAccount: Get<Self::AccountId>;
        
        /// 联盟计酬分配器（15层推荐链分配）
        type AffiliateDistributor: pallet_affiliate::types::AffiliateDistributor<
            Self::AccountId,
            BalanceOf<Self>,
            BlockNumberFor<Self>,
        >;
        
        /// IPFS Pin 接口（用于固定用户照片和资料）
        type IpfsPinner: IpfsPinner<Self::AccountId, BalanceOf<Self>>;
        
        /// 治理权限来源（用于处理违规）
        type GovernanceOrigin: frame_support::traits::EnsureOrigin<Self::RuntimeOrigin>;
        
        /// 每区块秒数（用于计算暂停天数）
        #[pallet::constant]
        type BlocksPerDay: Get<BlockNumberFor<Self>>;
        
        /// 余额类型
        type Balance: codec::FullCodec
            + codec::MaxEncodedLen
            + Copy
            + MaybeSerializeDeserialize
            + core::fmt::Debug
            + Default
            + scale_info::TypeInfo
            + Saturating
            + Zero
            + PartialOrd
            + Ord
            + TryFrom<u128>
            + TryInto<u128>;
    }

    /// 余额类型别名
    pub type BalanceOf<T> = <T as Config>::Balance;

    /// 保证金锁定原因枚举
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// 婚恋资料保证金
        ProfileDeposit,
    }

    // ========================================================================
    // 类型定义
    // ========================================================================

    /// 出生地信息
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct BirthLocation<T: Config> {
        /// 城市名称
        pub city: BoundedVec<u8, T::MaxLocationLen>,
        /// 经度（可选）
        pub longitude: Option<i32>,
        /// 纬度（可选）
        pub latitude: Option<i32>,
    }

    /// 用户征婚资料（完整版）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct UserProfile<T: Config> {
        // ========== 基本信息 ==========
        /// 昵称
        pub nickname: BoundedVec<u8, T::MaxNicknameLen>,
        /// 性别
        pub gender: Gender,
        /// 年龄（可选）
        pub age: Option<u8>,
        /// 出生日期（可选）
        pub birth_date: Option<BirthDate>,
        /// 出生时间（可选）
        pub birth_time: Option<BirthTime>,
        /// 出生地（可选）
        pub birth_location: Option<BirthLocation<T>>,
        /// 当前所在地（可选）
        pub current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
        /// 头像 CID（可选）
        pub avatar_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
        /// 生活照片列表（最多 9 张）
        pub photo_cids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, ConstU32<9>>,
        
        // ========== 个人条件 ==========
        /// 身高（cm）
        pub height: Option<u16>,
        /// 体重（kg）
        pub weight: Option<u16>,
        /// 学历
        pub education: Option<EducationLevel>,
        /// 职业
        pub occupation: Option<BoundedVec<u8, T::MaxOccupationLen>>,
        /// 收入范围（月收入）
        pub income_range: Option<(u32, u32)>,
        /// 房产情况
        pub property_status: Option<PropertyStatus>,
        /// 车辆情况
        pub vehicle_status: Option<VehicleStatus>,
        /// 婚姻状况
        pub marital_status: Option<MaritalStatus>,
        /// 是否有孩子
        pub has_children: Option<bool>,
        /// 是否想要孩子
        pub wants_children: Option<bool>,
        
        // ========== 性格与兴趣 ==========
        /// 性格特点
        pub personality_traits: BoundedVec<PersonalityTrait, T::MaxTraits>,
        /// 兴趣爱好
        pub hobbies: BoundedVec<BoundedVec<u8, T::MaxHobbyLen>, T::MaxHobbies>,
        /// 生活方式
        pub lifestyle: Option<Lifestyle>,
        
        // ========== 玄学信息 ==========
        /// 八字命盘 ID
        pub bazi_chart_id: Option<u64>,
        /// 合婚偏好
        pub compatibility_preferences: Option<CompatibilityPreferences>,
        
        // ========== 择偶条件 ==========
        /// 择偶条件
        pub partner_preferences: Option<PartnerPreferences<T>>,
        
        // ========== 自我介绍 ==========
        /// 个人简介
        pub bio: Option<BoundedVec<u8, T::MaxBioLen>>,
        /// 理想对象描述
        pub ideal_partner_desc: Option<BoundedVec<u8, T::MaxDescLen>>,
        
        // ========== 隐私与权限 ==========
        /// 隐私模式
        pub privacy_mode: ProfilePrivacyMode,
        /// 字段级隐私设置
        pub field_privacy: FieldPrivacySettings,
        
        // ========== 状态与元数据 ==========
        /// 资料完整度（0-100）
        pub completeness: u8,
        /// 资料状态
        pub status: ProfileStatus,
        /// 是否已验证
        pub verified: bool,
        /// 创建时间
        pub created_at: BlockNumberFor<T>,
        /// 最后更新时间
        pub updated_at: BlockNumberFor<T>,
        /// 最后活跃时间
        pub last_active_at: BlockNumberFor<T>,
    }

    /// 择偶条件（详细版）
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug, Default)]
    #[scale_info(skip_type_params(T))]
    pub struct PartnerPreferences<T: Config> {
        // 基础条件
        /// 年龄范围
        pub age_range: Option<(u8, u8)>,
        /// 身高范围（cm）
        pub height_range: Option<(u16, u16)>,
        /// 地域偏好
        pub location_preference: Option<BoundedVec<u8, T::MaxLocationLen>>,
        /// 学历要求
        pub education_requirement: Option<EducationLevel>,
        /// 收入范围
        pub income_range: Option<(u32, u32)>,
        /// 房产要求
        pub property_requirement: Option<PropertyStatus>,
        /// 车辆要求
        pub vehicle_requirement: Option<VehicleStatus>,
        /// 婚姻状况要求
        pub marital_status_requirement: Option<MaritalStatus>,
        /// 是否接受有孩子
        pub accept_children: Option<bool>,
        
        // 性格与兴趣
        /// 期望的性格特征
        pub desired_personality_traits: BoundedVec<PersonalityTrait, T::MaxTraits>,
        /// 期望的生活方式
        pub desired_lifestyle: Option<Lifestyle>,
        
        // 玄学条件
        /// 最低八字合婚评分要求
        pub min_bazi_compatibility: Option<u8>,
    }

    /// 性格分析数据
    /// 
    /// 结合用户自填和八字解盘的综合性格分析
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct PersonalityAnalysisData<T: Config> {
        // ========== 用户自填性格 ==========
        /// 用户自选的性格标签（最多 5 个）
        pub user_traits: BoundedVec<PersonalityTrait, ConstU32<5>>,
        /// 用户自我描述（可选）
        pub self_description: Option<BoundedVec<u8, T::MaxDescLen>>,
        
        // ========== 八字解盘性格 ==========
        /// 主要性格特点（来自八字解盘，最多 3 个）
        pub bazi_main_traits: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
        /// 性格优点（来自八字解盘，最多 3 个）
        pub bazi_strengths: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
        /// 性格缺点（来自八字解盘，最多 2 个）
        pub bazi_weaknesses: BoundedVec<BaziPersonalityTrait, ConstU32<2>>,
        
        // ========== 元数据 ==========
        /// 性格分析来源
        pub source: PersonalitySource,
        /// 八字命盘 ID（如果有八字分析）
        pub bazi_chart_id: Option<u64>,
        /// 分析更新时间（区块号）
        pub updated_at: BlockNumberFor<T>,
    }

    // ========================================================================
    // 存储
    // ========================================================================

    /// 用户资料存储
    #[pallet::storage]
    #[pallet::getter(fn profiles)]
    pub type Profiles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        UserProfile<T>,
    >;

    /// 用户总数
    #[pallet::storage]
    #[pallet::getter(fn profile_count)]
    pub type ProfileCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 性别索引（用于推荐）
    #[pallet::storage]
    #[pallet::getter(fn gender_index)]
    pub type GenderIndex<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        Gender,
        Blake2_128Concat,
        T::AccountId,
        (),
    >;

    /// 用户保证金记录
    #[pallet::storage]
    #[pallet::getter(fn deposits)]
    pub type Deposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BalanceOf<T>,
    >;

    /// 用户会员到期时间（区块号）
    #[pallet::storage]
    #[pallet::getter(fn membership_expiry)]
    pub type MembershipExpiry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
    >;

    /// 用户性格分析
    /// 
    /// 存储用户自填性格和八字解盘性格的综合分析
    #[pallet::storage]
    #[pallet::getter(fn personality_analysis)]
    pub type PersonalityAnalyses<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PersonalityAnalysisData<T>,
    >;

    /// 封禁用户列表
    #[pallet::storage]
    #[pallet::getter(fn banned_users)]
    pub type BannedUsers<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,  // 封禁时间
    >;

    /// 暂停用户列表（暂停结束区块号）
    #[pallet::storage]
    #[pallet::getter(fn suspended_until)]
    pub type SuspendedUntil<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,  // 暂停结束时间
    >;

    // ========================================================================
    // 事件
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 资料已创建
        ProfileCreated {
            who: T::AccountId,
            nickname: BoundedVec<u8, T::MaxNicknameLen>,
            gender: Gender,
        },
        /// 资料已更新
        ProfileUpdated {
            who: T::AccountId,
        },
        /// 择偶条件已更新
        PreferencesUpdated {
            who: T::AccountId,
        },
        /// 八字已绑定
        BaziLinked {
            who: T::AccountId,
            bazi_id: u64,
        },
        /// 隐私模式已更新
        PrivacyModeUpdated {
            who: T::AccountId,
            mode: ProfilePrivacyMode,
        },
        /// 资料已删除
        ProfileDeleted {
            who: T::AccountId,
        },
        /// 保证金已锁定
        DepositLocked {
            who: T::AccountId,
            amount: u128,
        },
        /// 保证金已释放
        DepositReleased {
            who: T::AccountId,
            amount: u128,
        },
        /// 保证金已罚没
        DepositSlashed {
            who: T::AccountId,
            amount: u128,
            reason: SlashReason,
        },
        /// 月费已支付
        MonthlyFeePaid {
            who: T::AccountId,
            amount: u128,
            months: u32,
            expiry_block: u64,
        },
        /// 会员已过期
        MembershipExpired {
            who: T::AccountId,
        },
        /// 用户性格已更新（自填）
        UserPersonalityUpdated {
            who: T::AccountId,
            traits_count: u8,
        },
        /// 八字性格已同步
        BaziPersonalitySynced {
            who: T::AccountId,
            bazi_chart_id: u64,
        },
        /// 照片已上传并固定到 IPFS
        PhotoUploaded {
            who: T::AccountId,
            cid: BoundedVec<u8, T::MaxCidLen>,
            pin_tier: u8,
        },
        /// 头像已更新并固定到 IPFS
        AvatarUpdated {
            who: T::AccountId,
            cid: BoundedVec<u8, T::MaxCidLen>,
        },
        /// 照片已取消固定（删除资料时）
        PhotoUnpinned {
            who: T::AccountId,
            cid: BoundedVec<u8, T::MaxCidLen>,
        },
        /// 用户已被封禁
        UserBanned {
            who: T::AccountId,
            reason: SlashReason,
        },
        /// 用户已被暂停
        UserSuspended {
            who: T::AccountId,
            until_block: BlockNumberFor<T>,
            reason: SlashReason,
        },
        /// 保证金已补充
        DepositToppedUp {
            who: T::AccountId,
            amount: u128,
            new_total: u128,
        },
        /// 保证金不足警告
        DepositInsufficient {
            who: T::AccountId,
            current: u128,
            required: u128,
        },
    }

    /// 罚没原因
    #[derive(Clone, Copy, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub enum SlashReason {
        /// 投诉成立 - 色情内容
        Pornography,
        /// 投诉成立 - 诈骗
        Fraud,
        /// 投诉成立 - 骚扰
        Harassment,
        /// 投诉成立 - 虚假资料
        FakeProfile,
        /// 投诉成立 - 其他
        Other,
    }

    /// 违规类型（用于确定扣除比例）
    #[derive(Clone, Copy, Encode, Decode, codec::DecodeWithMemTracking, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
    pub enum ViolationType {
        /// 轻微违规（5%）- 资料信息不完整、轻微误导
        Minor,
        /// 一般违规（10%）- 虚假信息、不当言论
        Moderate,
        /// 严重违规（20%）- 骚扰他人、恶意行为
        Severe,
        /// 特别严重（50%）- 欺诈、色情内容
        Critical,
        /// 永久封禁（100%）- 严重违法、多次违规
        PermanentBan,
    }

    impl ViolationType {
        /// 获取扣除比例（基点，10000 = 100%）
        pub fn slash_bps(&self) -> u16 {
            match self {
                ViolationType::Minor => 500,       // 5%
                ViolationType::Moderate => 1000,   // 10%
                ViolationType::Severe => 2000,     // 20%
                ViolationType::Critical => 5000,   // 50%
                ViolationType::PermanentBan => 10000, // 100%
            }
        }

        /// 是否需要封禁
        pub fn should_ban(&self) -> bool {
            matches!(self, ViolationType::PermanentBan)
        }

        /// 获取暂停天数（0 表示不暂停）
        pub fn suspension_days(&self) -> u32 {
            match self {
                ViolationType::Minor => 0,
                ViolationType::Moderate => 0,
                ViolationType::Severe => 7,
                ViolationType::Critical => 30,
                ViolationType::PermanentBan => 0, // 永久封禁
            }
        }
    }

    // ========================================================================
    // 错误
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        /// 资料已存在
        ProfileAlreadyExists,
        /// 资料不存在
        ProfileNotFound,
        /// 昵称过长
        NicknameTooLong,
        /// 昵称为空
        NicknameEmpty,
        /// 位置过长
        LocationTooLong,
        /// 简介过长
        BioTooLong,
        /// 无效的年龄范围
        InvalidAgeRange,
        /// 八字不存在
        BaziNotFound,
        /// 不是八字所有者
        NotBaziOwner,
        /// 余额不足支付保证金
        InsufficientBalance,
        /// 保证金不存在
        DepositNotFound,
        /// 罚没金额超过保证金
        SlashAmountExceedsDeposit,
        /// 会员已过期
        MembershipExpired,
        /// 无效的月数
        InvalidMonths,
        /// 性格标签过多
        TooManyPersonalityTraits,
        /// 八字性格分析不存在
        BaziPersonalityNotFound,
        /// 照片列表已满
        PhotoListFull,
        /// IPFS Pin 失败
        IpfsPinFailed,
        /// 用户已被封禁
        UserBanned,
        /// 用户已被暂停
        UserSuspended,
        /// 无权限操作
        NotAuthorized,
    }

    // ========================================================================
    // Extrinsics
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建用户资料
        /// 
        /// 需要支付 50 USDT 等值的 DUST 作为保证金，保证金将被锁定。
        /// 如果用户被投诉成功，保证金将被部分或全部罚没。
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_profile())]
        pub fn create_profile(
            origin: OriginFor<T>,
            nickname: BoundedVec<u8, T::MaxNicknameLen>,
            gender: Gender,
            age: Option<u8>,
            birth_date: Option<BirthDate>,
            current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
            bio: Option<BoundedVec<u8, T::MaxBioLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查资料是否已存在
            ensure!(!Profiles::<T>::contains_key(&who), Error::<T>::ProfileAlreadyExists);

            // 验证昵称
            ensure!(!nickname.is_empty(), Error::<T>::NicknameEmpty);

            // 计算并锁定保证金（50 USDT 等值的 DUST）
            let deposit_amount = Self::calculate_deposit_amount();
            
            // 检查用户余额是否足够
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= deposit_amount, Error::<T>::InsufficientBalance);
            
            // 锁定保证金
            T::Fungible::hold(&HoldReason::ProfileDeposit.into(), &who, deposit_amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
            
            // 记录保证金
            Deposits::<T>::insert(&who, deposit_amount);

            let current_block = frame_system::Pallet::<T>::block_number();

            let profile = UserProfile {
                // 基本信息
                nickname: nickname.clone(),
                gender,
                age,
                birth_date,
                birth_time: None,
                birth_location: None,
                current_location,
                avatar_cid: None,
                photo_cids: BoundedVec::default(),
                // 个人条件
                height: None,
                weight: None,
                education: None,
                occupation: None,
                income_range: None,
                property_status: None,
                vehicle_status: None,
                marital_status: None,
                has_children: None,
                wants_children: None,
                // 性格与兴趣
                personality_traits: BoundedVec::default(),
                hobbies: BoundedVec::default(),
                lifestyle: None,
                // 玄学信息
                bazi_chart_id: None,
                compatibility_preferences: None,
                // 择偶条件
                partner_preferences: None,
                // 自我介绍
                bio,
                ideal_partner_desc: None,
                // 隐私与权限
                privacy_mode: ProfilePrivacyMode::Public,
                field_privacy: FieldPrivacySettings::default(),
                // 状态与元数据
                completeness: 10, // 初始完整度
                status: ProfileStatus::Active,
                verified: false,
                created_at: current_block,
                updated_at: current_block,
                last_active_at: current_block,
            };

            // 存储资料
            Profiles::<T>::insert(&who, profile);
            ProfileCount::<T>::mutate(|c| *c = c.saturating_add(1));

            // 更新性别索引
            GenderIndex::<T>::insert(gender, &who, ());

            Self::deposit_event(Event::ProfileCreated {
                who: who.clone(),
                nickname,
                gender,
            });

            Self::deposit_event(Event::DepositLocked {
                who,
                amount: deposit_amount.try_into().unwrap_or(0u128),
            });

            Ok(())
        }

        /// 更新用户资料
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn update_profile(
            origin: OriginFor<T>,
            nickname: Option<BoundedVec<u8, T::MaxNicknameLen>>,
            current_location: Option<BoundedVec<u8, T::MaxLocationLen>>,
            avatar_cid: Option<BoundedVec<u8, T::MaxCidLen>>,
            bio: Option<BoundedVec<u8, T::MaxBioLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Profiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                if let Some(new_nickname) = nickname {
                    ensure!(!new_nickname.is_empty(), Error::<T>::NicknameEmpty);
                    profile.nickname = new_nickname;
                }

                if current_location.is_some() {
                    profile.current_location = current_location;
                }

                if avatar_cid.is_some() {
                    profile.avatar_cid = avatar_cid;
                }

                if bio.is_some() {
                    profile.bio = bio;
                }

                profile.updated_at = frame_system::Pallet::<T>::block_number();

                Self::deposit_event(Event::ProfileUpdated { who: who.clone() });

                Ok(())
            })
        }

        /// 更新择偶条件
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::update_preferences())]
        pub fn update_preferences(
            origin: OriginFor<T>,
            age_range: Option<(u8, u8)>,
            height_range: Option<(u16, u16)>,
            location_preference: Option<BoundedVec<u8, T::MaxLocationLen>>,
            education_requirement: Option<EducationLevel>,
            income_range: Option<(u32, u32)>,
            min_bazi_compatibility: Option<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证年龄范围
            if let Some((min, max)) = age_range {
                ensure!(min <= max, Error::<T>::InvalidAgeRange);
                ensure!(min >= 18 && max <= 100, Error::<T>::InvalidAgeRange);
            }

            Profiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                profile.partner_preferences = Some(PartnerPreferences {
                    age_range,
                    height_range,
                    location_preference,
                    education_requirement,
                    income_range,
                    property_requirement: None,
                    vehicle_requirement: None,
                    marital_status_requirement: None,
                    accept_children: None,
                    desired_personality_traits: BoundedVec::default(),
                    desired_lifestyle: None,
                    min_bazi_compatibility,
                });

                profile.updated_at = frame_system::Pallet::<T>::block_number();

                Self::deposit_event(Event::PreferencesUpdated { who: who.clone() });

                Ok(())
            })
        }

        /// 绑定八字命盘
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::link_bazi())]
        pub fn link_bazi(
            origin: OriginFor<T>,
            bazi_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Profiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                // TODO: 验证八字所有权（需要 BaziProvider trait）

                profile.bazi_chart_id = Some(bazi_id);
                profile.updated_at = frame_system::Pallet::<T>::block_number();

                Self::deposit_event(Event::BaziLinked {
                    who: who.clone(),
                    bazi_id,
                });

                Ok(())
            })
        }

        /// 更新隐私模式
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::update_privacy_mode())]
        pub fn update_privacy_mode(
            origin: OriginFor<T>,
            mode: ProfilePrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Profiles::<T>::try_mutate(&who, |maybe_profile| {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                profile.privacy_mode = mode;
                profile.updated_at = frame_system::Pallet::<T>::block_number();

                Self::deposit_event(Event::PrivacyModeUpdated {
                    who: who.clone(),
                    mode,
                });

                Ok(())
            })
        }

        /// 删除资料
        /// 
        /// 删除资料时将：
        /// 1. 取消固定所有照片（unpin from IPFS）
        /// 2. 释放剩余的保证金
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::delete_profile())]
        pub fn delete_profile(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let profile = Profiles::<T>::get(&who).ok_or(Error::<T>::ProfileNotFound)?;

            // ========== 1. Unpin 所有照片 ==========
            // Unpin 头像
            if let Some(avatar_cid) = &profile.avatar_cid {
                let _ = T::IpfsPinner::unpin_cid(who.clone(), avatar_cid.to_vec());
                Self::deposit_event(Event::PhotoUnpinned {
                    who: who.clone(),
                    cid: avatar_cid.clone(),
                });
            }
            
            // Unpin 所有照片
            for photo_cid in profile.photo_cids.iter() {
                let _ = T::IpfsPinner::unpin_cid(who.clone(), photo_cid.to_vec());
                Self::deposit_event(Event::PhotoUnpinned {
                    who: who.clone(),
                    cid: photo_cid.clone(),
                });
            }

            // ========== 2. 释放保证金 ==========
            if let Some(deposit_amount) = Deposits::<T>::take(&who) {
                if !deposit_amount.is_zero() {
                    // 释放锁定的保证金
                    let _ = T::Fungible::release(
                        &HoldReason::ProfileDeposit.into(),
                        &who,
                        deposit_amount,
                        frame_support::traits::tokens::Precision::BestEffort,
                    );
                    
                    Self::deposit_event(Event::DepositReleased {
                        who: who.clone(),
                        amount: deposit_amount.try_into().unwrap_or(0u128),
                    });
                }
            }

            // 移除性别索引
            GenderIndex::<T>::remove(profile.gender, &who);

            // 删除资料
            Profiles::<T>::remove(&who);
            ProfileCount::<T>::mutate(|c| *c = c.saturating_sub(1));

            Self::deposit_event(Event::ProfileDeleted { who });

            Ok(())
        }

        /// 支付月费
        /// 
        /// 支付 2 USDT 等值的 DUST 作为每月会员费。
        /// 可以一次性支付多个月。
        /// 
        /// 费用分配（15层推荐链）：
        /// - 销毁：5%
        /// - 国库：2%
        /// - 存储：3%
        /// - 推荐链分配：90%（15层分成）
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::create_profile())]
        pub fn pay_monthly_fee(
            origin: OriginFor<T>,
            months: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证资料存在
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);
            
            // 验证月数有效（1-12个月）
            ensure!(months >= 1 && months <= 12, Error::<T>::InvalidMonths);

            // 计算月费金额（2 USDT * months）
            let single_month_fee = Self::calculate_monthly_fee_amount();
            let months_u128 = months as u128;
            let single_fee_u128: u128 = single_month_fee.try_into().unwrap_or(0u128);
            let total_fee_u128 = single_fee_u128.saturating_mul(months_u128);
            let total_fee: BalanceOf<T> = total_fee_u128.try_into().unwrap_or(single_month_fee);
            
            // 检查用户余额是否足够
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= total_fee, Error::<T>::InsufficientBalance);
            
            // ========== 系统费用扣除（10%）==========
            // 销毁：5%
            let burn_amount = Self::calculate_percent(total_fee, 5);
            // 国库：2%
            let treasury_amount = Self::calculate_percent(total_fee, 2);
            // 存储：3%
            let storage_amount = Self::calculate_percent(total_fee, 3);
            // 可分配：90%
            let distributable = total_fee
                .saturating_sub(burn_amount)
                .saturating_sub(treasury_amount)
                .saturating_sub(storage_amount);
            
            // 销毁
            if !burn_amount.is_zero() {
                let burn_account = T::BurnAccount::get();
                T::Fungible::transfer(
                    &who,
                    &burn_account,
                    burn_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }
            
            // 国库
            if !treasury_amount.is_zero() {
                let treasury = T::TreasuryAccount::get();
                T::Fungible::transfer(
                    &who,
                    &treasury,
                    treasury_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }
            
            // 存储
            if !storage_amount.is_zero() {
                let storage_account = T::StorageAccount::get();
                T::Fungible::transfer(
                    &who,
                    &storage_account,
                    storage_amount,
                    frame_support::traits::tokens::Preservation::Preserve,
                )?;
            }
            
            // ========== 15层推荐链分配（90%）==========
            // 使用联盟计酬分配器进行15层分配
            let _ = T::AffiliateDistributor::distribute_rewards(
                &who,
                distributable,
                None, // 无特定目标
            );
            
            // 计算新的到期时间
            let current_block = frame_system::Pallet::<T>::block_number();
            // 假设每月约 432000 个区块（6秒/块，30天）
            let blocks_per_month: BlockNumberFor<T> = 432000u32.into();
            let extension_blocks = blocks_per_month.saturating_mul(months.into());
            
            let new_expiry = if let Some(current_expiry) = MembershipExpiry::<T>::get(&who) {
                // 如果当前会员未过期，从当前到期时间延长
                if current_expiry > current_block {
                    current_expiry.saturating_add(extension_blocks)
                } else {
                    // 已过期，从当前时间开始计算
                    current_block.saturating_add(extension_blocks)
                }
            } else {
                // 首次购买，从当前时间开始
                current_block.saturating_add(extension_blocks)
            };
            
            // 更新会员到期时间
            MembershipExpiry::<T>::insert(&who, new_expiry);
            
            // 更新最后活跃时间
            Profiles::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
                if let Some(profile) = maybe_profile.as_mut() {
                    profile.last_active_at = current_block;
                }
                Ok(())
            })?;

            Self::deposit_event(Event::MonthlyFeePaid {
                who,
                amount: total_fee.try_into().unwrap_or(0u128),
                months,
                expiry_block: new_expiry.try_into().unwrap_or(0u64),
            });

            Ok(())
        }

        /// 更新用户自填性格
        /// 
        /// 用户可以选择最多 5 个性格标签来描述自己
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn update_user_personality(
            origin: OriginFor<T>,
            traits: BoundedVec<PersonalityTrait, ConstU32<5>>,
            self_description: Option<BoundedVec<u8, T::MaxDescLen>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证资料存在
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);
            
            // 验证性格标签数量
            ensure!(traits.len() <= 5, Error::<T>::TooManyPersonalityTraits);

            let current_block = frame_system::Pallet::<T>::block_number();
            let traits_count = traits.len() as u8;

            // 更新或创建性格分析数据
            PersonalityAnalyses::<T>::mutate(&who, |maybe_analysis| {
                if let Some(analysis) = maybe_analysis.as_mut() {
                    // 更新现有数据
                    analysis.user_traits = traits;
                    analysis.self_description = self_description;
                    analysis.updated_at = current_block;
                    // 更新来源
                    if !analysis.bazi_main_traits.is_empty() {
                        analysis.source = PersonalitySource::Combined;
                    } else {
                        analysis.source = PersonalitySource::UserFilled;
                    }
                } else {
                    // 创建新数据
                    *maybe_analysis = Some(PersonalityAnalysisData {
                        user_traits: traits,
                        self_description,
                        bazi_main_traits: BoundedVec::default(),
                        bazi_strengths: BoundedVec::default(),
                        bazi_weaknesses: BoundedVec::default(),
                        source: PersonalitySource::UserFilled,
                        bazi_chart_id: None,
                        updated_at: current_block,
                    });
                }
            });

            // 更新资料的最后活跃时间
            Profiles::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
                if let Some(profile) = maybe_profile.as_mut() {
                    profile.last_active_at = current_block;
                }
                Ok(())
            })?;

            Self::deposit_event(Event::UserPersonalityUpdated {
                who,
                traits_count,
            });

            Ok(())
        }

        /// 同步八字性格分析
        /// 
        /// 从用户绑定的八字命盘同步性格分析数据
        /// 需要用户已绑定八字命盘
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn sync_bazi_personality(
            origin: OriginFor<T>,
            bazi_main_traits: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
            bazi_strengths: BoundedVec<BaziPersonalityTrait, ConstU32<3>>,
            bazi_weaknesses: BoundedVec<BaziPersonalityTrait, ConstU32<2>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证资料存在
            let profile = Profiles::<T>::get(&who).ok_or(Error::<T>::ProfileNotFound)?;
            
            // 验证八字命盘已绑定
            let bazi_chart_id = profile.bazi_chart_id.ok_or(Error::<T>::BaziNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 更新或创建性格分析数据
            PersonalityAnalyses::<T>::mutate(&who, |maybe_analysis| {
                if let Some(analysis) = maybe_analysis.as_mut() {
                    // 更新现有数据
                    analysis.bazi_main_traits = bazi_main_traits;
                    analysis.bazi_strengths = bazi_strengths;
                    analysis.bazi_weaknesses = bazi_weaknesses;
                    analysis.bazi_chart_id = Some(bazi_chart_id);
                    analysis.updated_at = current_block;
                    // 更新来源
                    if !analysis.user_traits.is_empty() {
                        analysis.source = PersonalitySource::Combined;
                    } else {
                        analysis.source = PersonalitySource::BaziAnalysis;
                    }
                } else {
                    // 创建新数据
                    *maybe_analysis = Some(PersonalityAnalysisData {
                        user_traits: BoundedVec::default(),
                        self_description: None,
                        bazi_main_traits,
                        bazi_strengths,
                        bazi_weaknesses,
                        source: PersonalitySource::BaziAnalysis,
                        bazi_chart_id: Some(bazi_chart_id),
                        updated_at: current_block,
                    });
                }
            });

            // 更新资料的最后活跃时间
            Profiles::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
                if let Some(profile) = maybe_profile.as_mut() {
                    profile.last_active_at = current_block;
                }
                Ok(())
            })?;

            Self::deposit_event(Event::BaziPersonalitySynced {
                who,
                bazi_chart_id,
            });

            Ok(())
        }

        /// 上传照片并固定到 IPFS
        /// 
        /// 用户上传照片 CID，系统自动调用 IPFS 模块进行 Pin 操作。
        /// 照片使用 Standard 层级存储（3副本，24小时巡检）。
        /// 
        /// # 参数
        /// - `cid`: 照片的 IPFS CID
        /// - `is_avatar`: 是否设置为头像
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn upload_photo(
            origin: OriginFor<T>,
            cid: BoundedVec<u8, T::MaxCidLen>,
            is_avatar: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证资料存在
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);

            // 生成唯一的 subject_id（使用账户哈希 + 时间戳）
            let current_block = frame_system::Pallet::<T>::block_number();
            let subject_id = Self::generate_subject_id(&who, current_block);

            // 调用 IPFS Pin（使用 Matchmaking SubjectType，Standard 层级）
            T::IpfsPinner::pin_cid_for_subject(
                who.clone(),
                SubjectType::Matchmaking,
                subject_id,
                cid.to_vec(),
                Some(PinTier::Standard),
            ).map_err(|_| Error::<T>::IpfsPinFailed)?;

            // 更新资料
            Profiles::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                if is_avatar {
                    // 设置为头像
                    profile.avatar_cid = Some(cid.clone());
                    Self::deposit_event(Event::AvatarUpdated {
                        who: who.clone(),
                        cid: cid.clone(),
                    });
                } else {
                    // 添加到照片列表
                    profile.photo_cids.try_push(cid.clone())
                        .map_err(|_| Error::<T>::PhotoListFull)?;
                }

                profile.updated_at = current_block;
                profile.last_active_at = current_block;

                Ok(())
            })?;

            Self::deposit_event(Event::PhotoUploaded {
                who,
                cid,
                pin_tier: 1, // Standard = 1
            });

            Ok(())
        }

        /// 批量上传照片并固定到 IPFS
        /// 
        /// 一次性上传多张照片，最多 9 张。
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::update_profile())]
        pub fn upload_photos_batch(
            origin: OriginFor<T>,
            cids: BoundedVec<BoundedVec<u8, T::MaxCidLen>, ConstU32<9>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 验证资料存在
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);

            let current_block = frame_system::Pallet::<T>::block_number();

            for (index, cid) in cids.iter().enumerate() {
                // 生成唯一的 subject_id
                let subject_id = Self::generate_subject_id(&who, current_block)
                    .saturating_add(index as u64);

                // 调用 IPFS Pin
                T::IpfsPinner::pin_cid_for_subject(
                    who.clone(),
                    SubjectType::Matchmaking,
                    subject_id,
                    cid.to_vec(),
                    Some(PinTier::Standard),
                ).map_err(|_| Error::<T>::IpfsPinFailed)?;

                Self::deposit_event(Event::PhotoUploaded {
                    who: who.clone(),
                    cid: cid.clone(),
                    pin_tier: 1,
                });
            }

            // 更新照片列表
            Profiles::<T>::try_mutate(&who, |maybe_profile| -> DispatchResult {
                let profile = maybe_profile.as_mut().ok_or(Error::<T>::ProfileNotFound)?;

                for cid in cids.iter() {
                    profile.photo_cids.try_push(cid.clone())
                        .map_err(|_| Error::<T>::PhotoListFull)?;
                }

                profile.updated_at = current_block;
                profile.last_active_at = current_block;

                Ok(())
            })?;

            Ok(())
        }

        // ==================== 治理调用入口 ====================

        /// 处理用户违规（治理权限）
        /// 
        /// 根据违规类型扣除保证金并执行相应处罚
        /// 
        /// # 参数
        /// - `user`: 违规用户
        /// - `violation_type`: 违规类型
        /// - `reason`: 罚没原因
        #[pallet::call_index(20)]
        #[pallet::weight(Weight::from_parts(60_000_000, 0))]
        pub fn handle_violation(
            origin: OriginFor<T>,
            user: T::AccountId,
            violation_type: ViolationType,
            reason: SlashReason,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            // 检查用户资料存在
            ensure!(Profiles::<T>::contains_key(&user), Error::<T>::ProfileNotFound);

            // 检查是否已被封禁
            ensure!(!BannedUsers::<T>::contains_key(&user), Error::<T>::UserBanned);

            let current_block = frame_system::Pallet::<T>::block_number();

            // 扣除保证金
            let slash_bps = violation_type.slash_bps();
            let slashed_amount = Self::slash_deposit(&user, slash_bps, reason)?;

            // 处理封禁
            if violation_type.should_ban() {
                BannedUsers::<T>::insert(&user, current_block);
                
                // 更新资料状态
                Profiles::<T>::mutate(&user, |maybe_profile| {
                    if let Some(profile) = maybe_profile {
                        profile.status = ProfileStatus::Banned;
                    }
                });

                Self::deposit_event(Event::UserBanned {
                    who: user.clone(),
                    reason,
                });
            } else {
                // 处理暂停
                let suspension_days = violation_type.suspension_days();
                if suspension_days > 0 {
                    let blocks_per_day = T::BlocksPerDay::get();
                    let suspension_blocks = blocks_per_day.saturating_mul(suspension_days.into());
                    let until_block = current_block.saturating_add(suspension_blocks);
                    
                    SuspendedUntil::<T>::insert(&user, until_block);
                    
                    // 更新资料状态
                    Profiles::<T>::mutate(&user, |maybe_profile| {
                        if let Some(profile) = maybe_profile {
                            profile.status = ProfileStatus::Suspended;
                        }
                    });

                    Self::deposit_event(Event::UserSuspended {
                        who: user.clone(),
                        until_block,
                        reason,
                    });
                }
            }

            // 检查保证金是否不足
            let deposit = Deposits::<T>::get(&user).unwrap_or_default();
            let required = Self::calculate_deposit_amount();
            let half_required = Self::calculate_percent(required, 50);
            
            if deposit < half_required {
                Self::deposit_event(Event::DepositInsufficient {
                    who: user,
                    current: deposit.try_into().unwrap_or(0u128),
                    required: required.try_into().unwrap_or(0u128),
                });
            }

            Ok(())
        }

        /// 补充保证金
        /// 
        /// 当保证金因违规被扣除后，用户可以补充保证金
        #[pallet::call_index(21)]
        #[pallet::weight(Weight::from_parts(30_000_000, 0))]
        pub fn top_up_deposit(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查资料存在
            ensure!(Profiles::<T>::contains_key(&who), Error::<T>::ProfileNotFound);

            // 不能是已封禁状态
            ensure!(!BannedUsers::<T>::contains_key(&who), Error::<T>::UserBanned);

            // 检查余额
            let balance = T::Fungible::balance(&who);
            ensure!(balance >= amount, Error::<T>::InsufficientBalance);

            // 锁定保证金
            T::Fungible::hold(&HoldReason::ProfileDeposit.into(), &who, amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // 更新保证金记录
            let current_deposit = Deposits::<T>::get(&who).unwrap_or_default();
            let new_total = current_deposit.saturating_add(amount);
            Deposits::<T>::insert(&who, new_total);

            // 如果保证金恢复到足够水平且处于暂停状态，可以恢复
            let required = Self::calculate_deposit_amount();
            if new_total >= required {
                // 检查暂停是否已过期
                if let Some(until_block) = SuspendedUntil::<T>::get(&who) {
                    let current_block = frame_system::Pallet::<T>::block_number();
                    if current_block >= until_block {
                        // 暂停已过期，恢复状态
                        SuspendedUntil::<T>::remove(&who);
                        Profiles::<T>::mutate(&who, |maybe_profile| {
                            if let Some(profile) = maybe_profile {
                                if profile.status == ProfileStatus::Suspended {
                                    profile.status = ProfileStatus::Active;
                                }
                            }
                        });
                    }
                }
            }

            Self::deposit_event(Event::DepositToppedUp {
                who,
                amount: amount.try_into().unwrap_or(0u128),
                new_total: new_total.try_into().unwrap_or(0u128),
            });

            Ok(())
        }

        /// 解除暂停（治理权限或用户自行解除）
        /// 
        /// 暂停期满后用户可以自行解除，或由治理提前解除
        #[pallet::call_index(22)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn lift_suspension(
            origin: OriginFor<T>,
            user: Option<T::AccountId>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin.clone()).ok();
            let is_governance = T::GovernanceOrigin::ensure_origin(origin).is_ok();

            let target = if is_governance {
                user.ok_or(Error::<T>::ProfileNotFound)?
            } else {
                caller.ok_or(Error::<T>::NotAuthorized)?
            };

            // 检查是否处于暂停状态
            let until_block = SuspendedUntil::<T>::get(&target)
                .ok_or(Error::<T>::ProfileNotFound)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 非治理调用需要检查暂停是否已过期
            if !is_governance {
                ensure!(current_block >= until_block, Error::<T>::UserSuspended);
            }

            // 检查保证金是否足够
            let deposit = Deposits::<T>::get(&target).unwrap_or_default();
            let required = Self::calculate_deposit_amount();
            let half_required = Self::calculate_percent(required, 50);
            ensure!(deposit >= half_required, Error::<T>::InsufficientBalance);

            // 解除暂停
            SuspendedUntil::<T>::remove(&target);
            Profiles::<T>::mutate(&target, |maybe_profile| {
                if let Some(profile) = maybe_profile {
                    if profile.status == ProfileStatus::Suspended {
                        profile.status = ProfileStatus::Active;
                    }
                }
            });

            Ok(())
        }
    }
}

// ============================================================================
// 辅助实现
// ============================================================================

impl<T: Config> Pallet<T> {
    /// 生成唯一的 subject_id（用于 IPFS Pin）
    /// 
    /// 使用账户编码和区块号生成唯一 ID
    fn generate_subject_id(account: &T::AccountId, block: BlockNumberFor<T>) -> u64 {
        use codec::Encode;
        let encoded = (account, block).encode();
        let hash = sp_io::hashing::blake2_256(&encoded);
        u64::from_le_bytes([hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]])
    }

    /// 检查资料是否存在
    pub fn profile_exists(account: &T::AccountId) -> bool {
        Profiles::<T>::contains_key(account)
    }

    /// 获取用户八字 ID
    pub fn get_bazi_id(account: &T::AccountId) -> Option<u64> {
        Profiles::<T>::get(account).and_then(|p| p.bazi_chart_id)
    }

    /// 检查用户是否已验证
    pub fn is_verified(account: &T::AccountId) -> bool {
        Profiles::<T>::get(account).map(|p| p.verified).unwrap_or(false)
    }

    /// 计算月费金额（2 USDT 等值的 DUST）
    /// 
    /// 优先使用实时汇率计算，如果汇率不可用则使用兆底金额
    pub fn calculate_monthly_fee_amount() -> BalanceOf<T> {
        // 尝试使用实时汇率计算
        if let Some(rate) = T::Pricing::get_dust_to_usd_rate() {
            let usd_amount = T::MonthlyFeeUsd::get(); // 2_000_000 = 2 USD
            if !rate.is_zero() {
                let usd_u128 = usd_amount as u128;
                let rate_u128: u128 = rate.try_into().unwrap_or(1_000_000u128);
                let dust_precision: u128 = 1_000_000_000_000_000_000u128; // 10^18
                let dust_amount_u128 = usd_u128.saturating_mul(dust_precision) / rate_u128;
                if let Ok(amount) = dust_amount_u128.try_into() {
                    return amount;
                }
            }
        }
        // 兆底金额
        T::MonthlyFee::get()
    }

    /// 检查用户会员是否有效
    pub fn is_membership_active(account: &T::AccountId) -> bool {
        if let Some(expiry) = MembershipExpiry::<T>::get(account) {
            let current_block = frame_system::Pallet::<T>::block_number();
            return expiry > current_block;
        }
        false
    }

    /// 获取用户会员到期时间
    pub fn get_membership_expiry(account: &T::AccountId) -> Option<BlockNumberFor<T>> {
        MembershipExpiry::<T>::get(account)
    }

    /// 计算百分比
    fn calculate_percent(total: BalanceOf<T>, percent: u8) -> BalanceOf<T> {
        if percent == 0 || percent > 100 {
            return BalanceOf::<T>::default();
        }
        let total_u128: u128 = total.try_into().unwrap_or(0u128);
        let result_u128 = total_u128.saturating_mul(percent as u128) / 100u128;
        result_u128.try_into().unwrap_or(BalanceOf::<T>::default())
    }

    /// 计算保证金金额（50 USDT 等值的 DUST）
    /// 
    /// 优先使用实时汇率计算，如果汇率不可用则使用兆底金额
    pub fn calculate_deposit_amount() -> BalanceOf<T> {
        // 尝试使用实时汇率计算
        if let Some(rate) = T::Pricing::get_dust_to_usd_rate() {
            // rate 精度为 10^6，即 1_000_000 = 1 USD
            // ProfileDepositUsd 精度也为 10^6，即 50_000_000 = 50 USD
            let usd_amount = T::ProfileDepositUsd::get(); // 50_000_000
            // dust_amount = usd_amount / rate * 10^18 (DUST 精度)
            // 简化：dust_amount = usd_amount * 10^18 / rate
            if !rate.is_zero() {
                // 为避免溢出，使用 u128 计算
                let usd_u128 = usd_amount as u128;
                let rate_u128: u128 = rate.try_into().unwrap_or(1_000_000u128);
                let dust_precision: u128 = 1_000_000_000_000_000_000u128; // 10^18
                let dust_amount_u128 = usd_u128.saturating_mul(dust_precision) / rate_u128;
                if let Ok(amount) = dust_amount_u128.try_into() {
                    return amount;
                }
            }
        }
        // 兆底金额
        T::ProfileDeposit::get()
    }

    /// 罚没用户保证金（由仲裁模块调用）
    /// 
    /// # 参数
    /// - `account`: 被罚没用户
    /// - `slash_bps`: 罚没比例（基点，10000 = 100%）
    /// - `reason`: 罚没原因
    /// 
    /// # 返回
    /// - 实际罚没金额
    pub fn slash_deposit(
        account: &T::AccountId,
        slash_bps: u16,
        reason: SlashReason,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let deposit = Deposits::<T>::get(account).ok_or(Error::<T>::DepositNotFound)?;
        
        // 计算罚没金额
        let slash_bps_u128 = slash_bps as u128;
        let deposit_u128: u128 = deposit.try_into().unwrap_or(0u128);
        let slash_amount_u128 = deposit_u128.saturating_mul(slash_bps_u128) / 10000u128;
        let slash_amount: BalanceOf<T> = slash_amount_u128.try_into().unwrap_or(deposit);
        
        // 确保罚没金额不超过保证金
        let actual_slash = slash_amount.min(deposit);
        
        if !actual_slash.is_zero() {
            // 从锁定中罚没并转入国库
            let treasury = T::TreasuryAccount::get();
            
            // 先释放锁定
            T::Fungible::release(
                &HoldReason::ProfileDeposit.into(),
                account,
                actual_slash,
                frame_support::traits::tokens::Precision::BestEffort,
            )?;
            
            // 转入国库
            T::Fungible::transfer(
                account,
                &treasury,
                actual_slash,
                frame_support::traits::tokens::Preservation::Expendable,
            )?;
            
            // 更新保证金记录
            let remaining = deposit.saturating_sub(actual_slash);
            if remaining.is_zero() {
                Deposits::<T>::remove(account);
            } else {
                Deposits::<T>::insert(account, remaining);
            }
            
            Self::deposit_event(Event::DepositSlashed {
                who: account.clone(),
                amount: actual_slash.try_into().unwrap_or(0u128),
                reason,
            });
        }
        
        Ok(actual_slash)
    }

    /// 获取用户当前保证金余额
    pub fn get_deposit(account: &T::AccountId) -> Option<BalanceOf<T>> {
        Deposits::<T>::get(account)
    }
}

// WeightInfo trait 和实现已移至 weights.rs
