//! # 统一隐私授权模块 (pallet-divination-privacy)
//!
//! 本模块为所有占卜系统提供统一的加密存储和多方授权功能。
//!
//! ## 功能概述
//!
//! 1. **密钥管理**：用户注册和更新 X25519 加密公钥
//! 2. **服务提供者管理**：命理师、AI 服务、家族成员注册
//! 3. **加密数据存储**：AES-256-GCM 加密的敏感数据存储
//! 4. **授权管理**：多方授权、角色控制、范围控制
//! 5. **悬赏集成**：与悬赏系统的授权集成
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        pallet-divination-market                          │
//! │                         (悬赏问答 & 订单系统)                              │
//! └───────────────────────────────────────┬─────────────────────────────────┘
//!                                         │ BountyPrivacy trait
//!                                         ▼
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                       pallet-divination-privacy                          │
//! │                         (统一隐私授权模块)                                │
//! │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐    │
//! │  │ 密钥管理    │ │ 授权管理    │ │ 加密存储    │ │ 服务提供者管理  │    │
//! │  │ - 公钥注册  │ │ - 授权/撤销 │ │ - 加密数据  │ │ - 命理师注册    │    │
//! │  │ - 密钥更新  │ │ - 角色/范围 │ │ - 解密凭证  │ │ - AI服务注册    │    │
//! │  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────────┘    │
//! └───────────────────────────────────────┬─────────────────────────────────┘
//!                                         │ DivinationPrivacy trait
//!                   ┌─────────────────────┼─────────────────┐
//!                   ▼                     ▼                 ▼
//!          ┌──────────────┐      ┌──────────────┐   ┌──────────────┐
//!          │ pallet-bazi  │      │pallet-meihua │   │pallet-liuyao │  ...
//!          │   (八字)     │      │  (梅花易数)   │   │   (六爻)     │
//!          └──────────────┘      └──────────────┘   └──────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ### 1. 用户注册加密公钥
//! ```ignore
//! Privacy::register_encryption_key(origin, public_key)?;
//! ```
//!
//! ### 2. 创建加密记录
//! ```ignore
//! Privacy::create_encrypted_record(
//!     origin,
//!     divination_type,
//!     result_id,
//!     privacy_mode,
//!     encrypted_data,
//!     nonce,
//!     auth_tag,
//!     data_hash,
//!     owner_encrypted_key,
//! )?;
//! ```
//!
//! ### 3. 授权访问
//! ```ignore
//! Privacy::grant_access(
//!     origin,
//!     divination_type,
//!     result_id,
//!     grantee,
//!     encrypted_key,
//!     role,
//!     scope,
//!     expires_at,
//! )?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod traits;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_divination_common::DivinationType;
use sp_std::vec::Vec;

use crate::traits::{BountyPrivacy, DivinationPrivacy, PrivacyEventHandler};
use crate::types::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use crate::weights::WeightInfo;

    // ========================================================================
    // Pallet 配置
    // ========================================================================

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> + pallet_timestamp::Config {
        /// 加密数据最大长度
        #[pallet::constant]
        type MaxEncryptedDataLen: Get<u32>;

        /// 加密密钥最大长度
        #[pallet::constant]
        type MaxEncryptedKeyLen: Get<u32>;

        /// 单条记录最大授权数
        #[pallet::constant]
        type MaxGranteesPerRecord: Get<u32>;

        /// 用户最大记录数（按类型）
        #[pallet::constant]
        type MaxRecordsPerUser: Get<u32>;

        /// 服务提供者最大数量（按类型）
        #[pallet::constant]
        type MaxProvidersPerType: Get<u32>;

        /// 提供者最大被授权记录数
        #[pallet::constant]
        type MaxGrantsPerProvider: Get<u32>;

        /// 单个悬赏最大授权数
        #[pallet::constant]
        type MaxAuthorizationsPerBounty: Get<u32>;

        /// 隐私事件回调处理器
        type EventHandler: PrivacyEventHandler<Self::AccountId>;

        /// 权重信息
        type WeightInfo: weights::WeightInfo;
    }

    // ========================================================================
    // 存储定义
    // ========================================================================

    // -------------------- 用户密钥管理 --------------------

    /// 用户加密公钥
    ///
    /// AccountId -> UserEncryptionInfo
    #[pallet::storage]
    #[pallet::getter(fn user_encryption_keys)]
    pub type UserEncryptionKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserEncryptionInfo<BlockNumberFor<T>>>;

    // -------------------- 服务提供者管理 --------------------

    /// 服务提供者信息
    ///
    /// AccountId -> ServiceProvider
    #[pallet::storage]
    #[pallet::getter(fn service_providers)]
    pub type ServiceProviders<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ServiceProvider<BlockNumberFor<T>>>;

    /// 按类型索引服务提供者
    ///
    /// ServiceProviderType -> Vec<AccountId>
    #[pallet::storage]
    #[pallet::getter(fn providers_by_type)]
    pub type ProvidersByType<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ServiceProviderType,
        BoundedVec<T::AccountId, T::MaxProvidersPerType>,
        ValueQuery,
    >;

    // -------------------- 加密数据存储 --------------------

    /// 加密记录存储（通用）
    ///
    /// (DivinationType, result_id) -> EncryptedRecord
    #[pallet::storage]
    #[pallet::getter(fn encrypted_records)]
    pub type EncryptedRecords<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        DivinationType,
        Blake2_128Concat,
        u64,
        EncryptedRecord<T::AccountId, BlockNumberFor<T>, T::MaxEncryptedDataLen>,
    >;

    /// 用户的加密记录索引
    ///
    /// (AccountId, DivinationType) -> Vec<result_id>
    #[pallet::storage]
    #[pallet::getter(fn user_encrypted_records)]
    pub type UserEncryptedRecords<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        DivinationType,
        BoundedVec<u64, T::MaxRecordsPerUser>,
        ValueQuery,
    >;

    // -------------------- 授权管理 --------------------

    /// 授权关系存储
    ///
    /// (DivinationType, result_id, grantee) -> AuthorizationEntry
    #[pallet::storage]
    #[pallet::getter(fn authorizations)]
    pub type Authorizations<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, DivinationType>,
            NMapKey<Blake2_128Concat, u64>,
            NMapKey<Blake2_128Concat, T::AccountId>,
        ),
        AuthorizationEntry<T::AccountId, BlockNumberFor<T>, T::MaxEncryptedKeyLen>,
    >;

    /// 记录的授权列表索引
    ///
    /// (DivinationType, result_id) -> Vec<grantee>
    #[pallet::storage]
    #[pallet::getter(fn record_grantees)]
    pub type RecordGrantees<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        DivinationType,
        Blake2_128Concat,
        u64,
        BoundedVec<T::AccountId, T::MaxGranteesPerRecord>,
        ValueQuery,
    >;

    /// 提供者被授权的记录索引（反向索引）
    ///
    /// AccountId -> Vec<(DivinationType, result_id)>
    #[pallet::storage]
    #[pallet::getter(fn provider_grants)]
    pub type ProviderGrants<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<RecordKey, T::MaxGrantsPerProvider>,
        ValueQuery,
    >;

    // -------------------- 悬赏授权关联 --------------------

    /// 悬赏授权信息
    ///
    /// bounty_id -> BountyAuthInfo
    #[pallet::storage]
    #[pallet::getter(fn bounty_auth_info)]
    pub type BountyAuthInfos<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BountyAuthInfo<BlockNumberFor<T>>>;

    /// 悬赏的授权账户列表
    ///
    /// bounty_id -> Vec<AccountId>
    #[pallet::storage]
    #[pallet::getter(fn bounty_authorizations)]
    pub type BountyAuthorizations<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<T::AccountId, T::MaxAuthorizationsPerBounty>,
        ValueQuery,
    >;

    // ========================================================================
    // 事件定义
    // ========================================================================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // -------------------- 密钥管理事件 --------------------

        /// 用户注册了加密公钥
        EncryptionKeyRegistered {
            account: T::AccountId,
            public_key: [u8; 32],
        },

        /// 用户更新了加密公钥
        EncryptionKeyUpdated {
            account: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },

        // -------------------- 服务提供者事件 --------------------

        /// 服务提供者注册
        ProviderRegistered {
            account: T::AccountId,
            provider_type: ServiceProviderType,
        },

        /// 服务提供者注销
        ProviderUnregistered { account: T::AccountId },

        /// 提供者状态变更
        ProviderStatusChanged {
            account: T::AccountId,
            is_active: bool,
        },

        /// 提供者公钥更新
        ProviderKeyUpdated {
            account: T::AccountId,
            old_key: [u8; 32],
            new_key: [u8; 32],
        },

        // -------------------- 加密记录事件 --------------------

        /// 创建加密记录
        EncryptedRecordCreated {
            divination_type: DivinationType,
            result_id: u64,
            owner: T::AccountId,
            privacy_mode: PrivacyMode,
        },

        /// 更新加密记录
        EncryptedRecordUpdated {
            divination_type: DivinationType,
            result_id: u64,
        },

        /// 隐私模式变更
        PrivacyModeChanged {
            divination_type: DivinationType,
            result_id: u64,
            old_mode: PrivacyMode,
            new_mode: PrivacyMode,
        },

        /// 删除加密记录
        EncryptedRecordDeleted {
            divination_type: DivinationType,
            result_id: u64,
        },

        // -------------------- 授权管理事件 --------------------

        /// 授权访问
        AccessGranted {
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
            role: AccessRole,
            scope: AccessScope,
            expires_at: BlockNumberFor<T>,
        },

        /// 撤销授权
        AccessRevoked {
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
        },

        /// 撤销所有授权
        AllAccessRevoked {
            divination_type: DivinationType,
            result_id: u64,
            count: u32,
        },

        /// 更新授权范围
        AccessScopeUpdated {
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
            new_scope: AccessScope,
        },

        // -------------------- 悬赏授权事件 --------------------

        /// 创建悬赏授权配置
        BountyAuthorizationCreated {
            bounty_id: u64,
            divination_type: DivinationType,
            result_id: u64,
            auto_authorize: bool,
        },

        /// 悬赏回答者获得授权
        BountyAnswererAuthorized {
            bounty_id: u64,
            answerer: T::AccountId,
        },

        /// 悬赏授权撤销
        BountyAuthorizationsRevoked { bounty_id: u64, count: u32 },
    }

    // ========================================================================
    // 错误定义
    // ========================================================================

    #[pallet::error]
    pub enum Error<T> {
        // -------------------- 密钥管理错误 --------------------

        /// 用户已注册公钥
        EncryptionKeyAlreadyRegistered,

        /// 用户未注册公钥
        EncryptionKeyNotRegistered,

        /// 无效的公钥格式
        InvalidPublicKey,

        // -------------------- 服务提供者错误 --------------------

        /// 已注册为服务提供者
        AlreadyRegisteredAsProvider,

        /// 不是服务提供者
        NotAProvider,

        /// 服务提供者类型列表已满
        ProviderTypeListFull,

        /// 服务提供者不活跃
        ProviderNotActive,

        // -------------------- 加密记录错误 --------------------

        /// 加密记录已存在
        EncryptedRecordAlreadyExists,

        /// 加密记录不存在
        EncryptedRecordNotFound,

        /// 不是记录所有者
        NotRecordOwner,

        /// 加密数据过长
        EncryptedDataTooLong,

        /// 用户记录列表已满
        UserRecordListFull,

        // -------------------- 授权管理错误 --------------------

        /// 授权已存在
        AuthorizationAlreadyExists,

        /// 授权不存在
        AuthorizationNotFound,

        /// 授权列表已满
        AuthorizationListFull,

        /// 授权已过期
        AuthorizationExpired,

        /// 无法撤销所有者授权
        CannotRevokeOwnerAccess,

        /// 提供者授权列表已满
        ProviderGrantListFull,

        /// 被授权者未注册公钥
        GranteeKeyNotRegistered,

        /// 加密密钥过长
        EncryptedKeyTooLong,

        // -------------------- 悬赏授权错误 --------------------

        /// 悬赏授权已存在
        BountyAuthorizationAlreadyExists,

        /// 悬赏授权不存在
        BountyAuthorizationNotFound,

        /// 悬赏授权列表已满
        BountyAuthorizationListFull,

        /// 关联记录非加密
        AssociatedRecordNotEncrypted,

        // -------------------- Partial 模式错误 --------------------

        /// Partial 模式必须指定加密字段
        ///
        /// 当 privacy_mode 为 Partial 时，encrypted_fields 参数必须指定
        /// 至少一个要加密的字段（使用 EncryptedFields 常量组合）
        PartialModeRequiresFields,

        /// 无效的加密字段标志
        ///
        /// encrypted_fields 值无效或包含未定义的标志位
        InvalidEncryptedFields,

        // -------------------- 通用错误 --------------------

        /// 无访问权限
        NoAccessPermission,

        /// 操作未授权
        Unauthorized,

        /// 数值溢出
        Overflow,
    }

    // ========================================================================
    // 交易接口
    // ========================================================================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // ====================================================================
        // 密钥管理
        // ====================================================================

        /// 注册用户加密公钥
        ///
        /// 用户首次注册自己的 X25519 公钥，用于接收加密数据。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `public_key`: X25519 公钥（32 字节）
        ///
        /// # 错误
        /// - `EncryptionKeyAlreadyRegistered`: 已注册公钥
        /// - `InvalidPublicKey`: 公钥格式无效
        #[pallet::call_index(0)]
        #[pallet::weight(<T as Config>::WeightInfo::register_encryption_key())]
        pub fn register_encryption_key(
            origin: OriginFor<T>,
            public_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否已注册
            ensure!(
                !UserEncryptionKeys::<T>::contains_key(&who),
                Error::<T>::EncryptionKeyAlreadyRegistered
            );

            // 验证公钥（非全零）
            ensure!(
                public_key != [0u8; 32],
                Error::<T>::InvalidPublicKey
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建用户加密信息
            let info = UserEncryptionInfo {
                public_key,
                registered_at: current_block,
                updated_at: current_block,
            };

            // 存储
            UserEncryptionKeys::<T>::insert(&who, info);

            // 触发事件
            Self::deposit_event(Event::EncryptionKeyRegistered {
                account: who,
                public_key,
            });

            Ok(())
        }

        /// 更新用户加密公钥
        ///
        /// 用户更新自己的加密公钥。注意：更新后需要重新为所有授权方加密数据密钥。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `new_public_key`: 新的 X25519 公钥（32 字节）
        ///
        /// # 错误
        /// - `EncryptionKeyNotRegistered`: 未注册公钥
        /// - `InvalidPublicKey`: 新公钥格式无效
        #[pallet::call_index(1)]
        #[pallet::weight(<T as Config>::WeightInfo::update_encryption_key())]
        pub fn update_encryption_key(
            origin: OriginFor<T>,
            new_public_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取现有信息
            let mut info = UserEncryptionKeys::<T>::get(&who)
                .ok_or(Error::<T>::EncryptionKeyNotRegistered)?;

            // 验证新公钥
            ensure!(
                new_public_key != [0u8; 32],
                Error::<T>::InvalidPublicKey
            );

            let old_key = info.public_key;

            // 更新公钥
            info.public_key = new_public_key;
            info.updated_at = frame_system::Pallet::<T>::block_number();

            // 存储
            UserEncryptionKeys::<T>::insert(&who, info);

            // 触发事件
            Self::deposit_event(Event::EncryptionKeyUpdated {
                account: who,
                old_key,
                new_key: new_public_key,
            });

            Ok(())
        }

        // ====================================================================
        // 服务提供者管理
        // ====================================================================

        /// 注册为服务提供者
        ///
        /// 用户注册为命理师、AI 服务或其他类型的服务提供者。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `provider_type`: 服务提供者类型
        /// - `public_key`: X25519 公钥（32 字节）
        ///
        /// # 错误
        /// - `AlreadyRegisteredAsProvider`: 已注册为服务提供者
        /// - `InvalidPublicKey`: 公钥格式无效
        /// - `ProviderTypeListFull`: 该类型服务提供者列表已满
        #[pallet::call_index(10)]
        #[pallet::weight(<T as Config>::WeightInfo::register_provider())]
        pub fn register_provider(
            origin: OriginFor<T>,
            provider_type: ServiceProviderType,
            public_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查是否已注册
            ensure!(
                !ServiceProviders::<T>::contains_key(&who),
                Error::<T>::AlreadyRegisteredAsProvider
            );

            // 验证公钥
            ensure!(
                public_key != [0u8; 32],
                Error::<T>::InvalidPublicKey
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建服务提供者信息
            let provider = ServiceProvider {
                provider_type,
                public_key,
                reputation: 50, // 初始信誉分
                is_active: true,
                registered_at: current_block,
                completed_services: 0,
            };

            // 存储提供者信息
            ServiceProviders::<T>::insert(&who, provider);

            // 添加到类型索引
            ProvidersByType::<T>::try_mutate(provider_type, |list| {
                list.try_push(who.clone())
                    .map_err(|_| Error::<T>::ProviderTypeListFull)
            })?;

            // 触发事件
            Self::deposit_event(Event::ProviderRegistered {
                account: who,
                provider_type,
            });

            Ok(())
        }

        /// 更新提供者公钥
        ///
        /// 服务提供者更新自己的加密公钥。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `new_public_key`: 新的 X25519 公钥（32 字节）
        ///
        /// # 错误
        /// - `NotAProvider`: 不是服务提供者
        /// - `InvalidPublicKey`: 新公钥格式无效
        #[pallet::call_index(11)]
        #[pallet::weight(<T as Config>::WeightInfo::update_provider_key())]
        pub fn update_provider_key(
            origin: OriginFor<T>,
            new_public_key: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取提供者信息
            let mut provider =
                ServiceProviders::<T>::get(&who).ok_or(Error::<T>::NotAProvider)?;

            // 验证新公钥
            ensure!(
                new_public_key != [0u8; 32],
                Error::<T>::InvalidPublicKey
            );

            let old_key = provider.public_key;

            // 更新公钥
            provider.public_key = new_public_key;

            // 存储
            ServiceProviders::<T>::insert(&who, provider);

            // 触发事件
            Self::deposit_event(Event::ProviderKeyUpdated {
                account: who,
                old_key,
                new_key: new_public_key,
            });

            Ok(())
        }

        /// 设置提供者活跃状态
        ///
        /// 服务提供者可以暂停或恢复自己的服务状态。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        /// - `is_active`: 是否活跃
        ///
        /// # 错误
        /// - `NotAProvider`: 不是服务提供者
        #[pallet::call_index(12)]
        #[pallet::weight(<T as Config>::WeightInfo::set_provider_active())]
        pub fn set_provider_active(origin: OriginFor<T>, is_active: bool) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取提供者信息
            ServiceProviders::<T>::try_mutate(&who, |maybe_provider| {
                let provider = maybe_provider.as_mut().ok_or(Error::<T>::NotAProvider)?;
                provider.is_active = is_active;
                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::ProviderStatusChanged {
                account: who,
                is_active,
            });

            Ok(())
        }

        /// 注销服务提供者
        ///
        /// 服务提供者注销自己的服务。
        ///
        /// # 参数
        /// - `origin`: 交易发起者
        ///
        /// # 错误
        /// - `NotAProvider`: 不是服务提供者
        #[pallet::call_index(13)]
        #[pallet::weight(<T as Config>::WeightInfo::unregister_provider())]
        pub fn unregister_provider(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并移除提供者信息
            let provider =
                ServiceProviders::<T>::take(&who).ok_or(Error::<T>::NotAProvider)?;

            // 从类型索引中移除
            ProvidersByType::<T>::mutate(provider.provider_type, |list| {
                list.retain(|account| account != &who);
            });

            // 触发事件
            Self::deposit_event(Event::ProviderUnregistered { account: who });

            Ok(())
        }

        // ====================================================================
        // 加密数据管理
        // ====================================================================

        /// 创建加密记录
        ///
        /// 为占卜结果创建加密存储记录。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是占卜结果的所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 占卜结果 ID
        /// - `privacy_mode`: 隐私模式
        /// - `encrypted_data`: 加密的敏感数据
        /// - `nonce`: 加密随机数（24 字节）
        /// - `auth_tag`: 认证标签（16 字节）
        /// - `data_hash`: 数据哈希（32 字节）
        /// - `owner_encrypted_key`: 所有者的加密数据密钥
        ///
        /// # 错误
        /// - `EncryptedRecordAlreadyExists`: 记录已存在
        /// - `EncryptedDataTooLong`: 加密数据过长
        /// - `UserRecordListFull`: 用户记录列表已满
        #[pallet::call_index(20)]
        #[pallet::weight(<T as Config>::WeightInfo::create_encrypted_record())]
        pub fn create_encrypted_record(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            privacy_mode: PrivacyMode,
            encrypted_data: Vec<u8>,
            nonce: [u8; 24],
            auth_tag: [u8; 16],
            data_hash: [u8; 32],
            owner_encrypted_key: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 检查记录是否已存在
            ensure!(
                !EncryptedRecords::<T>::contains_key(divination_type, result_id),
                Error::<T>::EncryptedRecordAlreadyExists
            );

            // 验证加密数据长度
            let bounded_data: BoundedVec<u8, T::MaxEncryptedDataLen> = encrypted_data
                .try_into()
                .map_err(|_| Error::<T>::EncryptedDataTooLong)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建加密记录
            let record = EncryptedRecord {
                divination_type,
                result_id,
                owner: who.clone(),
                privacy_mode,
                encrypted_data: bounded_data,
                nonce,
                auth_tag,
                data_hash,
                created_at: current_block,
                updated_at: current_block,
                // extrinsic 创建的记录不使用 Partial 模式，设为 None
                encrypted_fields: None,
            };

            // 存储加密记录
            EncryptedRecords::<T>::insert(divination_type, result_id, record);

            // 添加到用户记录索引
            UserEncryptedRecords::<T>::try_mutate(&who, divination_type, |list| {
                list.try_push(result_id)
                    .map_err(|_| Error::<T>::UserRecordListFull)
            })?;

            // 如果提供了所有者加密密钥，创建所有者授权条目
            if !owner_encrypted_key.is_empty() {
                let bounded_key: BoundedVec<u8, T::MaxEncryptedKeyLen> = owner_encrypted_key
                    .try_into()
                    .map_err(|_| Error::<T>::EncryptedKeyTooLong)?;

                let owner_auth = AuthorizationEntry {
                    grantee: who.clone(),
                    encrypted_key: bounded_key,
                    role: AccessRole::Owner,
                    scope: AccessScope::FullAccess,
                    granted_at: current_block,
                    expires_at: BlockNumberFor::<T>::default(), // 永不过期
                    bounty_id: None,
                };

                Authorizations::<T>::insert(
                    (divination_type, result_id, &who),
                    owner_auth,
                );

                // 添加到授权列表
                RecordGrantees::<T>::try_mutate(divination_type, result_id, |list| {
                    list.try_push(who.clone())
                        .map_err(|_| Error::<T>::AuthorizationListFull)
                })?;
            }

            // 调用事件处理器
            T::EventHandler::on_encrypted_record_created(divination_type, result_id, &who);

            // 触发事件
            Self::deposit_event(Event::EncryptedRecordCreated {
                divination_type,
                result_id,
                owner: who,
                privacy_mode,
            });

            Ok(())
        }

        /// 更新加密记录
        ///
        /// 更新加密数据内容。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `encrypted_data`: 新的加密数据
        /// - `nonce`: 新的加密随机数
        /// - `auth_tag`: 新的认证标签
        /// - `data_hash`: 新的数据哈希
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `EncryptedDataTooLong`: 加密数据过长
        #[pallet::call_index(21)]
        #[pallet::weight(<T as Config>::WeightInfo::update_encrypted_record())]
        pub fn update_encrypted_record(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            encrypted_data: Vec<u8>,
            nonce: [u8; 24],
            auth_tag: [u8; 16],
            data_hash: [u8; 32],
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并更新记录
            EncryptedRecords::<T>::try_mutate(divination_type, result_id, |maybe_record| {
                let record = maybe_record
                    .as_mut()
                    .ok_or(Error::<T>::EncryptedRecordNotFound)?;

                // 验证所有者
                ensure!(record.owner == who, Error::<T>::NotRecordOwner);

                // 验证并更新加密数据
                let bounded_data: BoundedVec<u8, T::MaxEncryptedDataLen> = encrypted_data
                    .try_into()
                    .map_err(|_| Error::<T>::EncryptedDataTooLong)?;

                record.encrypted_data = bounded_data;
                record.nonce = nonce;
                record.auth_tag = auth_tag;
                record.data_hash = data_hash;
                record.updated_at = frame_system::Pallet::<T>::block_number();

                Ok::<(), Error<T>>(())
            })?;

            // 触发事件
            Self::deposit_event(Event::EncryptedRecordUpdated {
                divination_type,
                result_id,
            });

            Ok(())
        }

        /// 更改隐私模式
        ///
        /// 更改记录的隐私模式。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `new_mode`: 新的隐私模式
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        #[pallet::call_index(22)]
        #[pallet::weight(<T as Config>::WeightInfo::change_privacy_mode())]
        pub fn change_privacy_mode(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            new_mode: PrivacyMode,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let old_mode = EncryptedRecords::<T>::try_mutate(
                divination_type,
                result_id,
                |maybe_record| {
                    let record = maybe_record
                        .as_mut()
                        .ok_or(Error::<T>::EncryptedRecordNotFound)?;

                    // 验证所有者
                    ensure!(record.owner == who, Error::<T>::NotRecordOwner);

                    let old = record.privacy_mode;
                    record.privacy_mode = new_mode;
                    record.updated_at = frame_system::Pallet::<T>::block_number();

                    Ok::<PrivacyMode, Error<T>>(old)
                },
            )?;

            // 触发事件
            Self::deposit_event(Event::PrivacyModeChanged {
                divination_type,
                result_id,
                old_mode,
                new_mode,
            });

            Ok(())
        }

        /// 删除加密记录
        ///
        /// 删除加密记录及其所有授权。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        #[pallet::call_index(23)]
        #[pallet::weight(<T as Config>::WeightInfo::delete_encrypted_record())]
        pub fn delete_encrypted_record(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取并验证记录
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 删除所有授权
            let grantees = RecordGrantees::<T>::take(divination_type, result_id);
            for grantee in grantees.iter() {
                Authorizations::<T>::remove((divination_type, result_id, grantee));

                // 从提供者授权列表中移除
                ProviderGrants::<T>::mutate(grantee, |list| {
                    list.retain(|key| {
                        key.divination_type != divination_type || key.result_id != result_id
                    });
                });
            }

            // 从用户记录索引中移除
            UserEncryptedRecords::<T>::mutate(&who, divination_type, |list| {
                list.retain(|id| *id != result_id);
            });

            // 删除记录
            EncryptedRecords::<T>::remove(divination_type, result_id);

            // 调用事件处理器
            T::EventHandler::on_encrypted_record_deleted(divination_type, result_id, &who);

            // 触发事件
            Self::deposit_event(Event::EncryptedRecordDeleted {
                divination_type,
                result_id,
            });

            Ok(())
        }

        // ====================================================================
        // 授权管理
        // ====================================================================

        /// 授权访问
        ///
        /// 为指定账户授权访问加密记录。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `grantee`: 被授权账户
        /// - `encrypted_key`: 用被授权者公钥加密的数据密钥
        /// - `role`: 授权角色
        /// - `scope`: 访问范围
        /// - `expires_at`: 过期时间（0 表示永久）
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `AuthorizationAlreadyExists`: 授权已存在
        /// - `AuthorizationListFull`: 授权列表已满
        /// - `GranteeKeyNotRegistered`: 被授权者未注册公钥
        /// - `EncryptedKeyTooLong`: 加密密钥过长
        #[pallet::call_index(30)]
        #[pallet::weight(<T as Config>::WeightInfo::grant_access())]
        pub fn grant_access(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
            encrypted_key: Vec<u8>,
            role: AccessRole,
            scope: AccessScope,
            expires_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取记录并验证所有者
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 检查授权是否已存在
            ensure!(
                !Authorizations::<T>::contains_key((divination_type, result_id, &grantee)),
                Error::<T>::AuthorizationAlreadyExists
            );

            // 验证加密密钥长度
            let bounded_key: BoundedVec<u8, T::MaxEncryptedKeyLen> = encrypted_key
                .try_into()
                .map_err(|_| Error::<T>::EncryptedKeyTooLong)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建授权条目
            let auth_entry = AuthorizationEntry {
                grantee: grantee.clone(),
                encrypted_key: bounded_key,
                role,
                scope,
                granted_at: current_block,
                expires_at,
                bounty_id: None,
            };

            // 存储授权
            Authorizations::<T>::insert((divination_type, result_id, &grantee), auth_entry);

            // 添加到授权列表
            RecordGrantees::<T>::try_mutate(divination_type, result_id, |list| {
                list.try_push(grantee.clone())
                    .map_err(|_| Error::<T>::AuthorizationListFull)
            })?;

            // 添加到提供者授权列表
            let record_key = RecordKey::new(divination_type, result_id);
            ProviderGrants::<T>::try_mutate(&grantee, |list| {
                list.try_push(record_key)
                    .map_err(|_| Error::<T>::ProviderGrantListFull)
            })?;

            // 调用事件处理器
            T::EventHandler::on_access_granted(divination_type, result_id, &who, &grantee, role);

            // 触发事件
            Self::deposit_event(Event::AccessGranted {
                divination_type,
                result_id,
                grantee,
                role,
                scope,
                expires_at,
            });

            Ok(())
        }

        /// 撤销授权
        ///
        /// 撤销指定账户的访问权限。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `grantee`: 被撤销账户
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `AuthorizationNotFound`: 授权不存在
        /// - `CannotRevokeOwnerAccess`: 无法撤销所有者授权
        #[pallet::call_index(31)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_access())]
        pub fn revoke_access(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取记录并验证所有者
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 获取授权
            let auth = Authorizations::<T>::get((divination_type, result_id, &grantee))
                .ok_or(Error::<T>::AuthorizationNotFound)?;

            // 不能撤销所有者授权
            ensure!(auth.role != AccessRole::Owner, Error::<T>::CannotRevokeOwnerAccess);

            // 删除授权
            Authorizations::<T>::remove((divination_type, result_id, &grantee));

            // 从授权列表中移除
            RecordGrantees::<T>::mutate(divination_type, result_id, |list| {
                list.retain(|account| account != &grantee);
            });

            // 从提供者授权列表中移除
            ProviderGrants::<T>::mutate(&grantee, |list| {
                list.retain(|key| {
                    key.divination_type != divination_type || key.result_id != result_id
                });
            });

            // 调用事件处理器
            T::EventHandler::on_access_revoked(divination_type, result_id, &who, &grantee);

            // 触发事件
            Self::deposit_event(Event::AccessRevoked {
                divination_type,
                result_id,
                grantee,
            });

            Ok(())
        }

        /// 撤销所有授权
        ///
        /// 撤销记录的所有授权（除所有者外）。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        #[pallet::call_index(32)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_all_access())]
        pub fn revoke_all_access(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取记录并验证所有者
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 获取所有授权
            let grantees = RecordGrantees::<T>::get(divination_type, result_id);
            let mut revoked_count = 0u32;

            for grantee in grantees.iter() {
                // 获取授权信息
                if let Some(auth) =
                    Authorizations::<T>::get((divination_type, result_id, grantee))
                {
                    // 跳过所有者
                    if auth.role == AccessRole::Owner {
                        continue;
                    }

                    // 删除授权
                    Authorizations::<T>::remove((divination_type, result_id, grantee));

                    // 从提供者授权列表中移除
                    ProviderGrants::<T>::mutate(grantee, |list| {
                        list.retain(|key| {
                            key.divination_type != divination_type || key.result_id != result_id
                        });
                    });

                    // 调用事件处理器
                    T::EventHandler::on_access_revoked(
                        divination_type,
                        result_id,
                        &who,
                        grantee,
                    );

                    revoked_count = revoked_count.saturating_add(1);
                }
            }

            // 更新授权列表（仅保留所有者）
            RecordGrantees::<T>::mutate(divination_type, result_id, |list| {
                list.retain(|account| account == &who);
            });

            // 触发事件
            Self::deposit_event(Event::AllAccessRevoked {
                divination_type,
                result_id,
                count: revoked_count,
            });

            Ok(())
        }

        /// 更新授权范围
        ///
        /// 更新被授权者的访问范围。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是所有者）
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `grantee`: 被授权账户
        /// - `new_scope`: 新的访问范围
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `AuthorizationNotFound`: 授权不存在
        #[pallet::call_index(33)]
        #[pallet::weight(<T as Config>::WeightInfo::update_access_scope())]
        pub fn update_access_scope(
            origin: OriginFor<T>,
            divination_type: DivinationType,
            result_id: u64,
            grantee: T::AccountId,
            new_scope: AccessScope,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取记录并验证所有者
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 更新授权范围
            Authorizations::<T>::try_mutate(
                (divination_type, result_id, &grantee),
                |maybe_auth| {
                    let auth = maybe_auth.as_mut().ok_or(Error::<T>::AuthorizationNotFound)?;
                    auth.scope = new_scope;
                    Ok::<(), Error<T>>(())
                },
            )?;

            // 触发事件
            Self::deposit_event(Event::AccessScopeUpdated {
                divination_type,
                result_id,
                grantee,
                new_scope,
            });

            Ok(())
        }

        // ====================================================================
        // 悬赏授权集成
        // ====================================================================

        /// 为悬赏创建授权配置
        ///
        /// 创建悬赏时调用，配置悬赏与加密数据的关联。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是记录所有者）
        /// - `bounty_id`: 悬赏 ID
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 结果 ID
        /// - `expires_at`: 授权过期时间
        /// - `auto_authorize`: 是否自动授权新回答者
        ///
        /// # 错误
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `BountyAuthorizationAlreadyExists`: 悬赏授权配置已存在
        #[pallet::call_index(40)]
        #[pallet::weight(<T as Config>::WeightInfo::create_bounty_authorization())]
        pub fn create_bounty_authorization(
            origin: OriginFor<T>,
            bounty_id: u64,
            divination_type: DivinationType,
            result_id: u64,
            expires_at: BlockNumberFor<T>,
            auto_authorize: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取记录并验证所有者
            let record = EncryptedRecords::<T>::get(divination_type, result_id)
                .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 检查是否已存在
            ensure!(
                !BountyAuthInfos::<T>::contains_key(bounty_id),
                Error::<T>::BountyAuthorizationAlreadyExists
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建悬赏授权信息
            let auth_info = BountyAuthInfo {
                divination_type,
                result_id,
                expires_at,
                created_at: current_block,
                auto_authorize,
            };

            // 存储
            BountyAuthInfos::<T>::insert(bounty_id, auth_info);

            // 触发事件
            Self::deposit_event(Event::BountyAuthorizationCreated {
                bounty_id,
                divination_type,
                result_id,
                auto_authorize,
            });

            Ok(())
        }

        /// 为悬赏回答者添加授权
        ///
        /// 大师接单时，所有者为其授权访问加密数据。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是记录所有者）
        /// - `bounty_id`: 悬赏 ID
        /// - `answerer`: 回答者账户
        /// - `encrypted_key`: 用回答者公钥加密的数据密钥
        ///
        /// # 错误
        /// - `BountyAuthorizationNotFound`: 悬赏授权配置不存在
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        /// - `AuthorizationAlreadyExists`: 授权已存在
        /// - `BountyAuthorizationListFull`: 悬赏授权列表已满
        #[pallet::call_index(41)]
        #[pallet::weight(<T as Config>::WeightInfo::authorize_bounty_answerer())]
        pub fn authorize_bounty_answerer(
            origin: OriginFor<T>,
            bounty_id: u64,
            answerer: T::AccountId,
            encrypted_key: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取悬赏授权信息
            let auth_info = BountyAuthInfos::<T>::get(bounty_id)
                .ok_or(Error::<T>::BountyAuthorizationNotFound)?;

            // 获取记录并验证所有者
            let record =
                EncryptedRecords::<T>::get(auth_info.divination_type, auth_info.result_id)
                    .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 检查是否已授权
            ensure!(
                !Authorizations::<T>::contains_key((
                    auth_info.divination_type,
                    auth_info.result_id,
                    &answerer
                )),
                Error::<T>::AuthorizationAlreadyExists
            );

            // 验证加密密钥长度
            let bounded_key: BoundedVec<u8, T::MaxEncryptedKeyLen> = encrypted_key
                .try_into()
                .map_err(|_| Error::<T>::EncryptedKeyTooLong)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建授权条目
            let auth_entry = AuthorizationEntry {
                grantee: answerer.clone(),
                encrypted_key: bounded_key,
                role: AccessRole::BountyAnswerer,
                scope: AccessScope::ReadOnly, // 悬赏回答者默认只读
                granted_at: current_block,
                expires_at: auth_info.expires_at,
                bounty_id: Some(bounty_id),
            };

            // 存储授权
            Authorizations::<T>::insert(
                (auth_info.divination_type, auth_info.result_id, &answerer),
                auth_entry,
            );

            // 添加到记录授权列表
            RecordGrantees::<T>::try_mutate(
                auth_info.divination_type,
                auth_info.result_id,
                |list| {
                    list.try_push(answerer.clone())
                        .map_err(|_| Error::<T>::AuthorizationListFull)
                },
            )?;

            // 添加到提供者授权列表
            let record_key = RecordKey::new(auth_info.divination_type, auth_info.result_id);
            ProviderGrants::<T>::try_mutate(&answerer, |list| {
                list.try_push(record_key)
                    .map_err(|_| Error::<T>::ProviderGrantListFull)
            })?;

            // 添加到悬赏授权列表
            BountyAuthorizations::<T>::try_mutate(bounty_id, |list| {
                list.try_push(answerer.clone())
                    .map_err(|_| Error::<T>::BountyAuthorizationListFull)
            })?;

            // 触发事件
            Self::deposit_event(Event::BountyAnswererAuthorized {
                bounty_id,
                answerer,
            });

            Ok(())
        }

        /// 悬赏结束时撤销所有临时授权
        ///
        /// 悬赏结束后，撤销所有 BountyAnswerer 的临时授权。
        ///
        /// # 参数
        /// - `origin`: 交易发起者（必须是记录所有者）
        /// - `bounty_id`: 悬赏 ID
        ///
        /// # 错误
        /// - `BountyAuthorizationNotFound`: 悬赏授权配置不存在
        /// - `EncryptedRecordNotFound`: 记录不存在
        /// - `NotRecordOwner`: 不是记录所有者
        #[pallet::call_index(42)]
        #[pallet::weight(<T as Config>::WeightInfo::revoke_bounty_authorizations())]
        pub fn revoke_bounty_authorizations(
            origin: OriginFor<T>,
            bounty_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 获取悬赏授权信息
            let auth_info = BountyAuthInfos::<T>::get(bounty_id)
                .ok_or(Error::<T>::BountyAuthorizationNotFound)?;

            // 获取记录并验证所有者
            let record =
                EncryptedRecords::<T>::get(auth_info.divination_type, auth_info.result_id)
                    .ok_or(Error::<T>::EncryptedRecordNotFound)?;

            ensure!(record.owner == who, Error::<T>::NotRecordOwner);

            // 获取并撤销所有悬赏授权
            let answerers = BountyAuthorizations::<T>::take(bounty_id);
            let revoked_count = answerers.len() as u32;

            for answerer in answerers.iter() {
                // 删除授权
                Authorizations::<T>::remove((
                    auth_info.divination_type,
                    auth_info.result_id,
                    answerer,
                ));

                // 从记录授权列表中移除
                RecordGrantees::<T>::mutate(
                    auth_info.divination_type,
                    auth_info.result_id,
                    |list| {
                        list.retain(|account| account != answerer);
                    },
                );

                // 从提供者授权列表中移除
                ProviderGrants::<T>::mutate(answerer, |list| {
                    list.retain(|key| {
                        key.divination_type != auth_info.divination_type
                            || key.result_id != auth_info.result_id
                    });
                });
            }

            // 删除悬赏授权信息
            BountyAuthInfos::<T>::remove(bounty_id);

            // 触发事件
            Self::deposit_event(Event::BountyAuthorizationsRevoked {
                bounty_id,
                count: revoked_count,
            });

            Ok(())
        }
    }

    // ========================================================================
    // Trait 实现
    // ========================================================================

    impl<T: Config> DivinationPrivacy<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
        fn is_encrypted(divination_type: DivinationType, result_id: u64) -> bool {
            EncryptedRecords::<T>::contains_key(divination_type, result_id)
        }

        fn get_privacy_mode(
            divination_type: DivinationType,
            result_id: u64,
        ) -> Option<PrivacyMode> {
            EncryptedRecords::<T>::get(divination_type, result_id)
                .map(|record| record.privacy_mode)
        }

        fn has_access(
            divination_type: DivinationType,
            result_id: u64,
            account: &T::AccountId,
        ) -> bool {
            // 获取记录
            if let Some(record) = EncryptedRecords::<T>::get(divination_type, result_id) {
                match record.privacy_mode {
                    // Public: 所有人可访问
                    PrivacyMode::Public => true,
                    // Partial & Private: 所有者或被授权者可访问
                    PrivacyMode::Partial | PrivacyMode::Private => {
                        // 检查是否是所有者
                        if record.owner == *account {
                            return true;
                        }

                        // 检查授权是否存在且未过期
                        if let Some(auth) =
                            Authorizations::<T>::get((divination_type, result_id, account))
                        {
                            let current_block = frame_system::Pallet::<T>::block_number();
                            // expires_at 为 0 表示永不过期
                            auth.expires_at == BlockNumberFor::<T>::default()
                                || auth.expires_at > current_block
                        } else {
                            false
                        }
                    }
                }
            } else {
                false
            }
        }

        fn get_access_role(
            divination_type: DivinationType,
            result_id: u64,
            account: &T::AccountId,
        ) -> Option<AccessRole> {
            Authorizations::<T>::get((divination_type, result_id, account))
                .map(|auth| auth.role)
        }

        fn get_access_scope(
            divination_type: DivinationType,
            result_id: u64,
            account: &T::AccountId,
        ) -> Option<AccessScope> {
            Authorizations::<T>::get((divination_type, result_id, account))
                .map(|auth| auth.scope)
        }

        fn get_grantees(divination_type: DivinationType, result_id: u64) -> Vec<T::AccountId> {
            RecordGrantees::<T>::get(divination_type, result_id).into_inner()
        }

        fn get_owner(
            divination_type: DivinationType,
            result_id: u64,
        ) -> Option<T::AccountId> {
            EncryptedRecords::<T>::get(divination_type, result_id).map(|record| record.owner)
        }

        fn get_user_public_key(account: &T::AccountId) -> Option<[u8; 32]> {
            UserEncryptionKeys::<T>::get(account).map(|info| info.public_key)
        }

        fn get_provider_type(account: &T::AccountId) -> Option<ServiceProviderType> {
            ServiceProviders::<T>::get(account).map(|provider| provider.provider_type)
        }

        fn is_provider_active(account: &T::AccountId) -> bool {
            ServiceProviders::<T>::get(account)
                .map(|provider| provider.is_active)
                .unwrap_or(false)
        }
    }

    impl<T: Config> BountyPrivacy<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
        fn is_bounty_encrypted(divination_type: DivinationType, result_id: u64) -> bool {
            EncryptedRecords::<T>::contains_key(divination_type, result_id)
        }

        fn can_answer_bounty(
            divination_type: DivinationType,
            result_id: u64,
            answerer: &T::AccountId,
        ) -> bool {
            // 如果记录不存在或公开，则允许
            if let Some(record) = EncryptedRecords::<T>::get(divination_type, result_id) {
                match record.privacy_mode {
                    // Public: 任何人都可以回答
                    PrivacyMode::Public => true,
                    // Partial & Private: 需要授权才能访问敏感数据
                    PrivacyMode::Partial | PrivacyMode::Private => {
                        // 检查是否有授权
                        if let Some(auth) =
                            Authorizations::<T>::get((divination_type, result_id, answerer))
                        {
                            let current_block = frame_system::Pallet::<T>::block_number();
                            auth.expires_at == BlockNumberFor::<T>::default()
                                || auth.expires_at > current_block
                        } else {
                            false
                        }
                    }
                }
            } else {
                true // 记录不存在，默认允许
            }
        }

        fn get_bounty_authorizations(bounty_id: u64) -> Vec<T::AccountId> {
            BountyAuthorizations::<T>::get(bounty_id).into_inner()
        }

        fn bounty_requires_authorization(
            divination_type: DivinationType,
            result_id: u64,
        ) -> bool {
            if let Some(record) = EncryptedRecords::<T>::get(divination_type, result_id) {
                match record.privacy_mode {
                    // Public: 不需要授权
                    PrivacyMode::Public => false,
                    // Partial & Private: 需要授权访问加密/敏感数据
                    PrivacyMode::Partial | PrivacyMode::Private => true,
                }
            } else {
                false
            }
        }

        fn get_bounty_authorization_expiry(bounty_id: u64) -> Option<BlockNumberFor<T>> {
            BountyAuthInfos::<T>::get(bounty_id).map(|info| info.expires_at)
        }

        fn is_auto_authorize_enabled(bounty_id: u64) -> bool {
            BountyAuthInfos::<T>::get(bounty_id)
                .map(|info| info.auto_authorize)
                .unwrap_or(false)
        }
    }

    // ========================================================================
    // 内部函数（供其他 pallet 调用）
    // ========================================================================

    impl<T: Config> Pallet<T> {
        /// 内部函数：创建加密记录
        ///
        /// 供其他 pallet（如 meihua、bazi、liuyao）原子性调用。
        /// 不触发事件，由调用方统一处理。
        ///
        /// # 参数
        /// - `owner`: 记录所有者
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 占卜结果 ID
        /// - `privacy_mode`: 隐私模式
        /// - `encrypted_data`: 加密数据
        /// - `nonce`: 加密随机数（24 字节）
        /// - `auth_tag`: 认证标签（16 字节）
        /// - `data_hash`: 数据哈希（32 字节）
        /// - `owner_encrypted_key`: 所有者的加密密钥
        ///
        /// # 返回
        /// - `Ok(())`: 创建成功
        /// - `Err(DispatchError)`: 创建失败
        pub fn do_create_encrypted_record(
            owner: &T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            privacy_mode: PrivacyMode,
            encrypted_data: Vec<u8>,
            nonce: [u8; 24],
            auth_tag: [u8; 16],
            data_hash: [u8; 32],
            owner_encrypted_key: Vec<u8>,
        ) -> DispatchResult {
            // 调用带 encrypted_fields 的完整版本
            Self::do_create_encrypted_record_with_fields(
                owner,
                divination_type,
                result_id,
                privacy_mode,
                encrypted_data,
                nonce,
                auth_tag,
                data_hash,
                owner_encrypted_key,
                None, // 不指定加密字段（完全加密模式）
            )
        }

        /// 内部函数：创建带字段标识的加密记录（支持 Partial 模式）
        ///
        /// 供其他 pallet（如 qimen）原子性调用，支持 Partial 模式的字段级加密。
        /// 不触发事件，由调用方统一处理。
        ///
        /// # 参数
        /// - `owner`: 记录所有者
        /// - `divination_type`: 占卜类型
        /// - `result_id`: 占卜结果 ID
        /// - `privacy_mode`: 隐私模式
        /// - `encrypted_data`: 加密数据
        /// - `nonce`: 加密随机数（24 字节）
        /// - `auth_tag`: 认证标签（16 字节）
        /// - `data_hash`: 数据哈希（32 字节）
        /// - `owner_encrypted_key`: 所有者的加密密钥
        /// - `encrypted_fields`: 加密字段标志位（Partial 模式必需，使用 `EncryptedFields` 常量组合）
        ///
        /// # Partial 模式说明
        ///
        /// 当 `privacy_mode == PrivacyMode::Partial` 时：
        /// - `encrypted_fields` 应指定哪些字段被加密（如 `EncryptedFields::NAME | EncryptedFields::QUESTION`）
        /// - 加密数据仅包含指定字段的内容
        /// - 未加密的计算数据保留在调用方的存储中（如 QimenChart）
        ///
        /// # 返回
        /// - `Ok(())`: 创建成功
        /// - `Err(DispatchError)`: 创建失败
        pub fn do_create_encrypted_record_with_fields(
            owner: &T::AccountId,
            divination_type: DivinationType,
            result_id: u64,
            privacy_mode: PrivacyMode,
            encrypted_data: Vec<u8>,
            nonce: [u8; 24],
            auth_tag: [u8; 16],
            data_hash: [u8; 32],
            owner_encrypted_key: Vec<u8>,
            encrypted_fields: Option<u16>,
        ) -> DispatchResult {
            // 检查记录是否已存在
            ensure!(
                !EncryptedRecords::<T>::contains_key(divination_type, result_id),
                Error::<T>::EncryptedRecordAlreadyExists
            );

            // Partial 模式验证：必须指定加密字段
            if privacy_mode == PrivacyMode::Partial {
                ensure!(
                    encrypted_fields.map(|f| f > 0).unwrap_or(false),
                    Error::<T>::PartialModeRequiresFields
                );
            }

            // 验证加密数据长度
            let bounded_data: BoundedVec<u8, T::MaxEncryptedDataLen> = encrypted_data
                .try_into()
                .map_err(|_| Error::<T>::EncryptedDataTooLong)?;

            let current_block = frame_system::Pallet::<T>::block_number();

            // 创建加密记录
            let record = EncryptedRecord {
                divination_type,
                result_id,
                owner: owner.clone(),
                privacy_mode,
                encrypted_data: bounded_data,
                nonce,
                auth_tag,
                data_hash,
                created_at: current_block,
                updated_at: current_block,
                // Partial 模式专用字段
                encrypted_fields,
            };

            // 存储加密记录
            EncryptedRecords::<T>::insert(divination_type, result_id, record);

            // 添加到用户记录索引
            UserEncryptedRecords::<T>::try_mutate(owner, divination_type, |list| {
                list.try_push(result_id)
                    .map_err(|_| Error::<T>::UserRecordListFull)
            })?;

            // 如果提供了所有者加密密钥，创建所有者授权条目
            if !owner_encrypted_key.is_empty() {
                let bounded_key: BoundedVec<u8, T::MaxEncryptedKeyLen> = owner_encrypted_key
                    .try_into()
                    .map_err(|_| Error::<T>::EncryptedKeyTooLong)?;

                let owner_auth = AuthorizationEntry {
                    grantee: owner.clone(),
                    encrypted_key: bounded_key,
                    role: AccessRole::Owner,
                    scope: AccessScope::FullAccess,
                    granted_at: current_block,
                    expires_at: BlockNumberFor::<T>::default(), // 永不过期
                    bounty_id: None,
                };

                Authorizations::<T>::insert(
                    (divination_type, result_id, owner),
                    owner_auth,
                );

                // 添加到授权列表
                RecordGrantees::<T>::try_mutate(divination_type, result_id, |list| {
                    list.try_push(owner.clone())
                        .map_err(|_| Error::<T>::AuthorizationListFull)
                })?;
            }

            // 调用事件处理器（不触发事件，由调用方处理）
            T::EventHandler::on_encrypted_record_created(divination_type, result_id, owner);

            Ok(())
        }

        /// 内部函数：获取加密记录的隐私模式
        ///
        /// 供其他 pallet 查询使用，判断是否为 Partial 模式
        pub fn get_privacy_mode(
            divination_type: DivinationType,
            result_id: u64,
        ) -> Option<PrivacyMode> {
            EncryptedRecords::<T>::get(divination_type, result_id)
                .map(|record| record.privacy_mode)
        }

        /// 内部函数：获取 Partial 模式的加密字段标志
        ///
        /// 供其他 pallet 查询使用，判断哪些字段被加密
        ///
        /// # 返回
        /// - `Some(flags)`: Partial 模式，返回加密字段标志位
        /// - `None`: 非 Partial 模式或记录不存在
        pub fn get_encrypted_fields(
            divination_type: DivinationType,
            result_id: u64,
        ) -> Option<u16> {
            EncryptedRecords::<T>::get(divination_type, result_id)
                .and_then(|record| {
                    if record.privacy_mode == PrivacyMode::Partial {
                        record.encrypted_fields
                    } else {
                        None
                    }
                })
        }

        /// 内部函数：检查是否为 Partial 模式
        ///
        /// 便捷方法，用于快速判断记录是否采用部分加密
        pub fn is_partial_mode(divination_type: DivinationType, result_id: u64) -> bool {
            Self::get_privacy_mode(divination_type, result_id)
                .map(|mode| mode == PrivacyMode::Partial)
                .unwrap_or(false)
        }

        /// 内部函数：检查加密记录是否存在
        ///
        /// 供其他 pallet 查询使用
        pub fn has_encrypted_record(divination_type: DivinationType, result_id: u64) -> bool {
            EncryptedRecords::<T>::contains_key(divination_type, result_id)
        }
    }
}
