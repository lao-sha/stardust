//! # 聊天权限系统 Pallet
//!
//! 实现基于场景的聊天权限控制系统，支持同一聊天会话应用于多个业务场景。
//!
//! ## 概述
//!
//! 本模块提供以下功能：
//! - 用户隐私设置管理（权限级别、黑白名单）
//! - 好友关系管理
//! - 场景授权管理（多场景共存）
//! - 聊天权限检查
//!
//! ## 核心概念
//!
//! - **聊天会话**: 两个用户之间的通信通道，唯一
//! - **场景授权**: 为什么这两个用户可以聊天的原因，可以有多个
//! - **权限判定**: 黑名单 → 好友 → 场景授权 → 隐私设置
//!
//! ## 使用示例
//!
//! ```ignore
//! // 业务 pallet 授予场景授权
//! T::ChatPermission::grant_bidirectional_scene_authorization(
//!     *b"otc_ordr",
//!     &buyer,
//!     &seller,
//!     SceneType::Order,
//!     SceneId::Numeric(order_id),
//!     Some(30 * 24 * 60 * 10), // 30天
//!     "订单#123".as_bytes().to_vec(),
//! )?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod weights;
pub use weights::WeightInfo;

mod traits;
mod types;
pub mod runtime_api;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use traits::*;
pub use types::*;
pub use runtime_api::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::SaturatedConversion;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Pallet 配置 trait
    #[pallet::config]
    pub trait Config: frame_system::Config<RuntimeEvent: From<Event<Self>>> {
        /// 黑名单最大数量
        #[pallet::constant]
        type MaxBlockListSize: Get<u32>;

        /// 白名单最大数量
        #[pallet::constant]
        type MaxWhitelistSize: Get<u32>;

        /// 单对用户最大场景授权数量
        /// 考虑场景：多个订单 + 多个纪念馆 + 群聊等
        #[pallet::constant]
        type MaxScenesPerPair: Get<u32>;
    }

    // ==================== 存储 ====================

    /// 用户隐私设置存储
    ///
    /// 存储每个用户的聊天权限配置，包括权限级别、黑白名单等。
    #[pallet::storage]
    #[pallet::getter(fn privacy_settings)]
    pub type PrivacySettingsOf<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        PrivacySettings<T>,
        ValueQuery,
    >;

    /// 好友关系存储
    ///
    /// 双向存储好友关系，值为建立好友关系的区块号。
    /// 查询时需要检查双向是否都存在。
    #[pallet::storage]
    #[pallet::getter(fn friendships)]
    pub type Friendships<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        BlockNumberFor<T>,
        OptionQuery,
    >;

    /// 场景授权存储
    ///
    /// Key: (user1, user2) 按字典序排列，保证双向查询一致性
    /// Value: 场景授权列表
    #[pallet::storage]
    #[pallet::getter(fn scene_authorizations)]
    pub type SceneAuthorizations<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<SceneAuthorization<BlockNumberFor<T>>, T::MaxScenesPerPair>,
        ValueQuery,
    >;

    // ==================== 事件 ====================

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 隐私设置已更新
        PrivacySettingsUpdated {
            who: T::AccountId,
        },

        /// 用户已被屏蔽
        UserBlocked {
            blocker: T::AccountId,
            blocked: T::AccountId,
        },

        /// 用户已被解除屏蔽
        UserUnblocked {
            unblocker: T::AccountId,
            unblocked: T::AccountId,
        },

        /// 好友关系已建立
        FriendshipCreated {
            user1: T::AccountId,
            user2: T::AccountId,
        },

        /// 好友关系已解除
        FriendshipRemoved {
            user1: T::AccountId,
            user2: T::AccountId,
        },

        /// 场景授权已授予
        SceneAuthorizationGranted {
            source: [u8; 8],
            user1: T::AccountId,
            user2: T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
        },

        /// 场景授权已撤销
        SceneAuthorizationRevoked {
            source: [u8; 8],
            user1: T::AccountId,
            user2: T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
        },

        /// 场景授权已延期
        SceneAuthorizationExtended {
            user1: T::AccountId,
            user2: T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
            new_expires_at: Option<BlockNumberFor<T>>,
        },

        /// 用户添加到白名单
        UserAddedToWhitelist {
            owner: T::AccountId,
            user: T::AccountId,
        },

        /// 用户从白名单移除
        UserRemovedFromWhitelist {
            owner: T::AccountId,
            user: T::AccountId,
        },
    }

    // ==================== 错误 ====================

    #[pallet::error]
    pub enum Error<T> {
        /// 黑名单已满
        BlockListFull,

        /// 白名单已满
        WhitelistFull,

        /// 用户已在黑名单中
        AlreadyBlocked,

        /// 用户不在黑名单中
        NotInBlockList,

        /// 不能添加自己
        CannotAddSelf,

        /// 好友关系已存在
        FriendshipAlreadyExists,

        /// 好友关系不存在
        FriendshipNotFound,

        /// 场景授权数量已达上限
        TooManyScenes,

        /// 场景授权不存在
        SceneAuthorizationNotFound,

        /// 场景授权已存在
        SceneAuthorizationAlreadyExists,

        /// 用户已在白名单中
        AlreadyInWhitelist,

        /// 用户不在白名单中
        NotInWhitelist,

        /// 元数据过长
        MetadataTooLong,
    }

    // ==================== 用户调用 ====================

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 设置聊天权限级别
        ///
        /// 用户可以设置自己的聊天权限策略：
        /// - Open: 任何人可发起聊天
        /// - FriendsOnly: 仅好友可发起（默认）
        /// - Whitelist: 仅白名单用户可发起
        /// - Closed: 不接受任何消息
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_permission_level(
            origin: OriginFor<T>,
            level: ChatPermissionLevel,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivacySettingsOf::<T>::mutate(&who, |settings| {
                settings.permission_level = level;
                settings.updated_at = frame_system::Pallet::<T>::block_number();
            });

            Self::deposit_event(Event::PrivacySettingsUpdated { who });
            Ok(())
        }

        /// 设置拒绝的场景类型
        ///
        /// 用户可以选择拒绝某些类型的场景授权聊天。
        /// 例如，用户可以拒绝所有 MarketMaker 场景的聊天请求。
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn set_rejected_scene_types(
            origin: OriginFor<T>,
            scene_types: BoundedVec<SceneType, ConstU32<10>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivacySettingsOf::<T>::mutate(&who, |settings| {
                settings.rejected_scene_types = scene_types;
                settings.updated_at = frame_system::Pallet::<T>::block_number();
            });

            Self::deposit_event(Event::PrivacySettingsUpdated { who });
            Ok(())
        }

        /// 添加用户到黑名单
        ///
        /// 被屏蔽的用户将无法向屏蔽者发送消息，
        /// 即使存在有效的场景授权或好友关系。
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn block_user(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who != user, Error::<T>::CannotAddSelf);

            PrivacySettingsOf::<T>::try_mutate(&who, |settings| {
                ensure!(
                    !settings.block_list.contains(&user),
                    Error::<T>::AlreadyBlocked
                );
                settings
                    .block_list
                    .try_push(user.clone())
                    .map_err(|_| Error::<T>::BlockListFull)?;
                settings.updated_at = frame_system::Pallet::<T>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::UserBlocked {
                blocker: who,
                blocked: user,
            });
            Ok(())
        }

        /// 从黑名单移除用户
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn unblock_user(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivacySettingsOf::<T>::try_mutate(&who, |settings| {
                let pos = settings
                    .block_list
                    .iter()
                    .position(|x| x == &user)
                    .ok_or(Error::<T>::NotInBlockList)?;
                settings.block_list.remove(pos);
                settings.updated_at = frame_system::Pallet::<T>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::UserUnblocked {
                unblocker: who,
                unblocked: user,
            });
            Ok(())
        }

        /// 添加好友
        ///
        /// 建立双向好友关系。好友之间可以无视权限级别自由聊天。
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn add_friend(origin: OriginFor<T>, friend: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who != friend, Error::<T>::CannotAddSelf);
            ensure!(
                Friendships::<T>::get(&who, &friend).is_none(),
                Error::<T>::FriendshipAlreadyExists
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            // 双向存储好友关系
            Friendships::<T>::insert(&who, &friend, current_block);
            Friendships::<T>::insert(&friend, &who, current_block);

            Self::deposit_event(Event::FriendshipCreated {
                user1: who,
                user2: friend,
            });
            Ok(())
        }

        /// 删除好友
        ///
        /// 解除双向好友关系。
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(15_000, 0))]
        pub fn remove_friend(origin: OriginFor<T>, friend: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Friendships::<T>::get(&who, &friend).is_some(),
                Error::<T>::FriendshipNotFound
            );

            // 双向移除好友关系
            Friendships::<T>::remove(&who, &friend);
            Friendships::<T>::remove(&friend, &who);

            Self::deposit_event(Event::FriendshipRemoved {
                user1: who,
                user2: friend,
            });
            Ok(())
        }

        /// 添加用户到白名单
        ///
        /// 在 Whitelist 模式下，只有白名单中的用户才能发起聊天。
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn add_to_whitelist(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who != user, Error::<T>::CannotAddSelf);

            PrivacySettingsOf::<T>::try_mutate(&who, |settings| {
                ensure!(
                    !settings.whitelist.contains(&user),
                    Error::<T>::AlreadyInWhitelist
                );
                settings
                    .whitelist
                    .try_push(user.clone())
                    .map_err(|_| Error::<T>::WhitelistFull)?;
                settings.updated_at = frame_system::Pallet::<T>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::UserAddedToWhitelist { owner: who, user });
            Ok(())
        }

        /// 从白名单移除用户
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn remove_from_whitelist(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            PrivacySettingsOf::<T>::try_mutate(&who, |settings| {
                let pos = settings
                    .whitelist
                    .iter()
                    .position(|x| x == &user)
                    .ok_or(Error::<T>::NotInWhitelist)?;
                settings.whitelist.remove(pos);
                settings.updated_at = frame_system::Pallet::<T>::block_number();
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::UserRemovedFromWhitelist { owner: who, user });
            Ok(())
        }
    }

    // ==================== 内部方法 ====================

    impl<T: Config> Pallet<T> {
        /// 获取排序后的用户对（保证存储一致性）
        ///
        /// 将两个用户按字典序排列，确保无论传入顺序如何，
        /// 都能查询到同一条存储记录。
        pub fn sorted_pair(
            user1: &T::AccountId,
            user2: &T::AccountId,
        ) -> (T::AccountId, T::AccountId) {
            if user1 < user2 {
                (user1.clone(), user2.clone())
            } else {
                (user2.clone(), user1.clone())
            }
        }

        /// 检查聊天权限
        ///
        /// 按以下优先级检查权限：
        /// 1. 黑名单检查（最高优先级拒绝）
        /// 2. 好友关系检查
        /// 3. 场景授权检查
        /// 4. 隐私设置检查
        pub fn check_permission(
            sender: &T::AccountId,
            receiver: &T::AccountId,
        ) -> PermissionResult {
            let current_block = frame_system::Pallet::<T>::block_number();

            // 1. 检查是否被屏蔽
            let receiver_settings = PrivacySettingsOf::<T>::get(receiver);
            if receiver_settings.block_list.contains(sender) {
                return PermissionResult::DeniedBlocked;
            }

            // 2. 检查好友关系
            if Friendships::<T>::get(sender, receiver).is_some() {
                return PermissionResult::AllowedByFriendship;
            }

            // 3. 检查场景授权
            let (user1, user2) = Self::sorted_pair(sender, receiver);
            let authorizations = SceneAuthorizations::<T>::get(&user1, &user2);

            let valid_scenes: Vec<SceneType> = authorizations
                .iter()
                .filter(|auth| {
                    // 检查是否过期
                    if let Some(expires_at) = auth.expires_at {
                        if current_block > expires_at {
                            return false;
                        }
                    }
                    // 检查是否被接收方拒绝
                    !receiver_settings
                        .rejected_scene_types
                        .contains(&auth.scene_type)
                })
                .map(|auth| auth.scene_type.clone())
                .collect();

            if !valid_scenes.is_empty() {
                return PermissionResult::AllowedByScene(valid_scenes);
            }

            // 4. 根据隐私设置判断
            match receiver_settings.permission_level {
                ChatPermissionLevel::Open => PermissionResult::Allowed,
                ChatPermissionLevel::FriendsOnly => PermissionResult::DeniedRequiresFriend,
                ChatPermissionLevel::Whitelist => {
                    if receiver_settings.whitelist.contains(sender) {
                        PermissionResult::Allowed
                    } else {
                        PermissionResult::DeniedNotInWhitelist
                    }
                }
                ChatPermissionLevel::Closed => PermissionResult::DeniedClosed,
            }
        }

        /// 获取两用户间所有有效的场景授权
        ///
        /// 返回包含过期状态的场景授权信息列表，用于前端展示。
        pub fn get_active_scenes(
            user1: &T::AccountId,
            user2: &T::AccountId,
        ) -> Vec<SceneAuthorizationInfo> {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (u1, u2) = Self::sorted_pair(user1, user2);
            let authorizations = SceneAuthorizations::<T>::get(&u1, &u2);

            authorizations
                .iter()
                .map(|auth| {
                    let is_expired = auth
                        .expires_at
                        .map(|e| current_block > e)
                        .unwrap_or(false);
                    SceneAuthorizationInfo {
                        scene_type: auth.scene_type.clone(),
                        scene_id: auth.scene_id.clone(),
                        is_expired,
                        expires_at: auth.expires_at.map(|b| b.saturated_into::<u64>()),
                        metadata: auth.metadata.to_vec(),
                    }
                })
                .collect()
        }

        /// 清理过期的场景授权
        ///
        /// 移除两个用户之间所有已过期的场景授权。
        /// 可以在适当时机调用以释放存储空间。
        pub fn cleanup_expired_scenes(user1: &T::AccountId, user2: &T::AccountId) {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (u1, u2) = Self::sorted_pair(user1, user2);

            SceneAuthorizations::<T>::mutate(&u1, &u2, |auths| {
                auths.retain(|auth| auth.expires_at.map(|e| current_block <= e).unwrap_or(true));
            });
        }

        /// 获取用户隐私设置摘要
        ///
        /// 返回简化的隐私设置信息，用于前端展示。
        pub fn get_privacy_summary(user: &T::AccountId) -> PrivacySettingsSummary {
            let settings = PrivacySettingsOf::<T>::get(user);
            PrivacySettingsSummary {
                permission_level: settings.permission_level,
                block_list_count: settings.block_list.len() as u32,
                whitelist_count: settings.whitelist.len() as u32,
                rejected_scene_types: settings.rejected_scene_types.to_vec(),
            }
        }
    }

    // ==================== 实现 SceneAuthorizationManager Trait ====================

    impl<T: Config> SceneAuthorizationManager<T::AccountId, BlockNumberFor<T>> for Pallet<T> {
        /// 授予场景授权（单向）
        fn grant_scene_authorization(
            source: [u8; 8],
            from: &T::AccountId,
            to: &T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
            duration: Option<BlockNumberFor<T>>,
            metadata: Vec<u8>,
        ) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let expires_at = duration.map(|d| current_block + d);
            let (user1, user2) = Self::sorted_pair(from, to);

            let bounded_metadata: BoundedVec<u8, ConstU32<128>> =
                metadata.try_into().map_err(|_| Error::<T>::MetadataTooLong)?;

            let authorization = SceneAuthorization {
                scene_type: scene_type.clone(),
                scene_id: scene_id.clone(),
                source_pallet: source,
                granted_at: current_block,
                expires_at,
                metadata: bounded_metadata,
            };

            SceneAuthorizations::<T>::try_mutate(&user1, &user2, |auths| {
                // 检查是否已存在相同场景
                let existing_pos = auths
                    .iter()
                    .position(|a| a.scene_type == scene_type && a.scene_id == scene_id);

                if let Some(pos) = existing_pos {
                    // 更新现有授权
                    auths[pos] = authorization.clone();
                } else {
                    // 添加新授权
                    auths
                        .try_push(authorization)
                        .map_err(|_| Error::<T>::TooManyScenes)?;
                }
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::SceneAuthorizationGranted {
                source,
                user1,
                user2,
                scene_type,
                scene_id,
            });

            Ok(())
        }

        /// 授予双向场景授权
        fn grant_bidirectional_scene_authorization(
            source: [u8; 8],
            user1: &T::AccountId,
            user2: &T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
            duration: Option<BlockNumberFor<T>>,
            metadata: Vec<u8>,
        ) -> DispatchResult {
            // 由于存储已经是双向的（使用排序后的 key），只需调用一次
            Self::grant_scene_authorization(
                source, user1, user2, scene_type, scene_id, duration, metadata,
            )
        }

        /// 撤销特定场景授权
        fn revoke_scene_authorization(
            source: [u8; 8],
            from: &T::AccountId,
            to: &T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
        ) -> DispatchResult {
            let (user1, user2) = Self::sorted_pair(from, to);

            SceneAuthorizations::<T>::try_mutate(&user1, &user2, |auths| {
                let pos = auths
                    .iter()
                    .position(|a| {
                        a.source_pallet == source
                            && a.scene_type == scene_type
                            && a.scene_id == scene_id
                    })
                    .ok_or(Error::<T>::SceneAuthorizationNotFound)?;

                auths.remove(pos);
                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::SceneAuthorizationRevoked {
                source,
                user1,
                user2,
                scene_type,
                scene_id,
            });

            Ok(())
        }

        /// 撤销某来源的所有场景授权
        fn revoke_all_by_source(
            source: [u8; 8],
            user1: &T::AccountId,
            user2: &T::AccountId,
        ) -> DispatchResult {
            let (u1, u2) = Self::sorted_pair(user1, user2);

            SceneAuthorizations::<T>::mutate(&u1, &u2, |auths| {
                auths.retain(|a| a.source_pallet != source);
            });

            Ok(())
        }

        /// 延长场景授权有效期
        fn extend_scene_authorization(
            source: [u8; 8],
            from: &T::AccountId,
            to: &T::AccountId,
            scene_type: SceneType,
            scene_id: SceneId,
            additional_duration: BlockNumberFor<T>,
        ) -> DispatchResult {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (user1, user2) = Self::sorted_pair(from, to);

            let mut new_expires_at = None;

            SceneAuthorizations::<T>::try_mutate(&user1, &user2, |auths| {
                let auth = auths
                    .iter_mut()
                    .find(|a| {
                        a.source_pallet == source
                            && a.scene_type == scene_type
                            && a.scene_id == scene_id
                    })
                    .ok_or(Error::<T>::SceneAuthorizationNotFound)?;

                // 从当前时间或原过期时间延长
                let base = auth.expires_at.unwrap_or(current_block);
                let new_time = base.max(current_block) + additional_duration;
                auth.expires_at = Some(new_time);
                new_expires_at = Some(new_time);

                Ok::<_, DispatchError>(())
            })?;

            Self::deposit_event(Event::SceneAuthorizationExtended {
                user1,
                user2,
                scene_type,
                scene_id,
                new_expires_at,
            });

            Ok(())
        }

        /// 检查是否有任何有效的场景授权
        fn has_any_valid_scene_authorization(from: &T::AccountId, to: &T::AccountId) -> bool {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (user1, user2) = Self::sorted_pair(from, to);
            let authorizations = SceneAuthorizations::<T>::get(&user1, &user2);

            authorizations.iter().any(|auth| {
                auth.expires_at
                    .map(|e| current_block <= e)
                    .unwrap_or(true)
            })
        }

        /// 获取所有有效的场景授权
        fn get_valid_scene_authorizations(
            user1: &T::AccountId,
            user2: &T::AccountId,
        ) -> Vec<SceneAuthorization<BlockNumberFor<T>>> {
            let current_block = frame_system::Pallet::<T>::block_number();
            let (u1, u2) = Self::sorted_pair(user1, user2);
            let authorizations = SceneAuthorizations::<T>::get(&u1, &u2);

            authorizations
                .into_iter()
                .filter(|auth| auth.expires_at.map(|e| current_block <= e).unwrap_or(true))
                .collect()
        }
    }

    // ==================== 实现 ChatPermissionChecker Trait ====================

    impl<T: Config> ChatPermissionChecker<T::AccountId> for Pallet<T> {
        fn can_send_message(sender: &T::AccountId, receiver: &T::AccountId) -> bool {
            Self::check_permission(sender, receiver).is_allowed()
        }
    }

    // ==================== 实现 FriendshipChecker Trait ====================

    impl<T: Config> FriendshipChecker<T::AccountId> for Pallet<T> {
        fn is_friend(user1: &T::AccountId, user2: &T::AccountId) -> bool {
            Friendships::<T>::get(user1, user2).is_some()
        }
    }
}
