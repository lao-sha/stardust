//! # 直播间模块 (pallet-livestream)
//!
//! 为 Stardust 平台提供去中心化直播功能，支持主播开播、观众互动、礼物打赏等核心功能。
//!
//! ## 设计原则
//!
//! - **链上链下分离**: 资金相关操作上链，高频操作（聊天、弹幕）链下处理
//! - **最小化存储**: 观众列表、礼物记录等移至链下或用事件替代
//! - **签名验证**: 主播推流通过私钥签名验证，不存储 stream_key
//!
//! ## 功能模块
//!
//! - 直播间管理: 创建、开播、暂停、结束
//! - 礼物系统: 打赏、分成、提现
//! - 付费直播: 门票购买、验证
//! - 连麦功能: 开始/结束连麦记录
//! - 管理功能: 黑名单、封禁

#![cfg_attr(not(feature = "std"), no_std)]

pub mod runtime_api;
pub mod types;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, BuildGenesisConfig},
    PalletId,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};

pub use pallet::*;
pub use types::*;
pub use weights::WeightInfo;

/// 余额类型别名
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    /// 存储版本
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// 事件类型
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// 货币类型
        type Currency: ReservableCurrency<Self::AccountId>;

        /// 直播间标题最大长度
        #[pallet::constant]
        type MaxTitleLen: Get<u32>;

        /// 直播间描述最大长度
        #[pallet::constant]
        type MaxDescriptionLen: Get<u32>;

        /// CID 最大长度
        #[pallet::constant]
        type MaxCidLen: Get<u32>;

        /// 礼物名称最大长度
        #[pallet::constant]
        type MaxGiftNameLen: Get<u32>;

        /// 最大连麦人数
        #[pallet::constant]
        type MaxCoHostsPerRoom: Get<u32>;

        /// 平台抽成比例 (百分比, 如 20 表示 20%)
        #[pallet::constant]
        type PlatformFeePercent: Get<u8>;

        /// 最小提现金额
        #[pallet::constant]
        type MinWithdrawAmount: Get<BalanceOf<Self>>;

        /// 创建直播间保证金兜底值（DUST数量，pricing不可用时使用）
        #[pallet::constant]
        type RoomBond: Get<BalanceOf<Self>>;

        /// 创建直播间保证金USD价值（精度10^6，5_000_000 = 5 USDT）
        #[pallet::constant]
        type RoomBondUsd: Get<u64>;

        /// 保证金计算器（统一的 USD 价值动态计算）
        type DepositCalculator: pallet_trading_common::DepositCalculator<BalanceOf<Self>>;

        /// Pallet ID (用于生成模块账户)
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// 治理来源 (用于封禁等管理操作)
        type GovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// 权重信息
        type WeightInfo: WeightInfo;

        // 注意：直播间数据是临时数据，直播结束后即废弃，不适合 IPFS PIN
        // 原因：直播结束后封面图无用，继续 PIN 会造成存储浪费
        // 建议：前端直接上传到公共 IPFS 网关，链上仅存 CID 引用
    }

    // ============ 存储 ============

    /// 直播间信息
    #[pallet::storage]
    #[pallet::getter(fn live_rooms)]
    pub type LiveRooms<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // room_id
        LiveRoom<
            T::AccountId,
            BalanceOf<T>,
            T::MaxTitleLen,
            T::MaxDescriptionLen,
            T::MaxCidLen,
        >,
    >;

    /// 主播的直播间 (一个主播只能有一个活跃直播间)
    #[pallet::storage]
    #[pallet::getter(fn host_room)]
    pub type HostRoom<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    /// 下一个直播间 ID (自增)
    #[pallet::storage]
    #[pallet::getter(fn next_room_id)]
    pub type NextRoomId<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// 直播间保证金记录
    #[pallet::storage]
    #[pallet::getter(fn room_deposits)]
    pub type RoomDeposits<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // room_id
        (T::AccountId, BalanceOf<T>), // (depositor, amount)
        OptionQuery,
    >;

    /// 礼物定义
    #[pallet::storage]
    #[pallet::getter(fn gifts)]
    pub type Gifts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u32, // gift_id
        Gift<BalanceOf<T>, T::MaxGiftNameLen, T::MaxCidLen>,
    >;

    /// 下一个礼物 ID
    #[pallet::storage]
    #[pallet::getter(fn next_gift_id)]
    pub type NextGiftId<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// 用户在直播间的累计打赏
    #[pallet::storage]
    #[pallet::getter(fn user_room_gifts)]
    pub type UserRoomGifts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,           // room_id
        Blake2_128Concat,
        T::AccountId,  // user
        BalanceOf<T>,  // total_gifted
        ValueQuery,
    >;

    /// 主播累计收入
    #[pallet::storage]
    #[pallet::getter(fn host_earnings)]
    pub type HostEarnings<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

    /// 付费直播门票持有者
    #[pallet::storage]
    #[pallet::getter(fn ticket_holders)]
    pub type TicketHolders<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,           // room_id
        Blake2_128Concat,
        T::AccountId,  // buyer
        u64,           // purchase_time (block number)
    >;

    /// 直播间黑名单
    #[pallet::storage]
    #[pallet::getter(fn room_blacklist)]
    pub type RoomBlacklist<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,           // room_id
        Blake2_128Concat,
        T::AccountId,  // banned_user
        (),
    >;

    /// 当前连麦者列表
    #[pallet::storage]
    #[pallet::getter(fn active_co_hosts)]
    pub type ActiveCoHosts<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64, // room_id
        BoundedVec<T::AccountId, T::MaxCoHostsPerRoom>,
        ValueQuery,
    >;

    // ============ 创世配置 ============

    /// 创世配置 - 用于初始化礼物
    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// 初始礼物列表: (名称, 价格, 图标CID)
        pub gifts: Vec<(Vec<u8>, BalanceOf<T>, Vec<u8>)>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            for (name, price, icon_cid) in &self.gifts {
                let gift_id = NextGiftId::<T>::get();
                NextGiftId::<T>::put(gift_id.saturating_add(1));

                let name: BoundedVec<u8, T::MaxGiftNameLen> = name
                    .clone()
                    .try_into()
                    .expect("Gift name too long in genesis config");
                let icon_cid: BoundedVec<u8, T::MaxCidLen> = icon_cid
                    .clone()
                    .try_into()
                    .expect("Gift icon CID too long in genesis config");

                let gift = Gift {
                    id: gift_id,
                    name,
                    price: *price,
                    icon_cid,
                    enabled: true,
                };

                Gifts::<T>::insert(gift_id, gift);
            }
        }
    }

    // ============ 事件 ============

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 直播间已创建
        RoomCreated {
            host: T::AccountId,
            room_id: u64,
            room_type: LiveRoomType,
        },
        /// 直播已开始
        LiveStarted {
            room_id: u64,
            started_at: BlockNumberFor<T>,
        },
        /// 直播已暂停
        LivePaused {
            room_id: u64,
        },
        /// 直播已恢复
        LiveResumed {
            room_id: u64,
        },
        /// 直播已结束
        LiveEnded {
            room_id: u64,
            duration: u64,
            total_viewers: u64,
            peak_viewers: u32,
            total_gifts: BalanceOf<T>,
        },
        /// 直播间信息已更新
        RoomUpdated {
            room_id: u64,
        },
        /// 门票已购买
        TicketPurchased {
            room_id: u64,
            buyer: T::AccountId,
            price: BalanceOf<T>,
        },
        /// 礼物已发送
        GiftSent {
            room_id: u64,
            sender: T::AccountId,
            receiver: T::AccountId,
            gift_id: u32,
            quantity: u32,
            value: BalanceOf<T>,
        },
        /// 收益已提现
        EarningsWithdrawn {
            host: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 观众被踢出
        ViewerKicked {
            room_id: u64,
            viewer: T::AccountId,
        },
        /// 观众从黑名单移除
        ViewerUnbanned {
            room_id: u64,
            viewer: T::AccountId,
        },
        /// 直播间被封禁
        RoomBanned {
            room_id: u64,
            reason: Vec<u8>,
        },
        /// 直播间保证金被扣除（投诉裁决）
        RoomBondSlashed {
            room_id: u64,
            host: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 超额保证金已提取
        ExcessBondWithdrawn {
            room_id: u64,
            host: T::AccountId,
            withdrawn: BalanceOf<T>,
            remaining: BalanceOf<T>,
        },
        /// 保证金被罚没（封禁）
        BondConfiscated {
            room_id: u64,
            host: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// 连麦已开始
        CoHostStarted {
            room_id: u64,
            co_host: T::AccountId,
        },
        /// 连麦已结束
        CoHostEnded {
            room_id: u64,
            co_host: T::AccountId,
        },
        /// 礼物已创建
        GiftCreated {
            gift_id: u32,
            price: BalanceOf<T>,
        },
        /// 礼物状态已更新
        GiftUpdated {
            gift_id: u32,
            enabled: bool,
        },
        /// 直播统计已同步
        LiveStatsSynced {
            room_id: u64,
            total_viewers: u64,
            peak_viewers: u32,
        },
    }

    // ============ 错误 ============

    #[pallet::error]
    pub enum Error<T> {
        /// 直播间不存在
        RoomNotFound,
        /// 不是直播间房主
        NotRoomHost,
        /// 直播间未在直播中
        RoomNotLive,
        /// 直播间已在直播中
        RoomAlreadyLive,
        /// 直播间已结束
        RoomEnded,
        /// 直播间已被封禁
        RoomBanned,
        /// 主播已有活跃直播间
        HostAlreadyHasRoom,
        /// 礼物不存在
        GiftNotFound,
        /// 礼物已禁用
        GiftDisabled,
        /// 余额不足
        InsufficientBalance,
        /// 已购买门票
        AlreadyHasTicket,
        /// 需要购买门票
        TicketRequired,
        /// 已在黑名单中
        AlreadyInBlacklist,
        /// 不在黑名单中
        NotInBlacklist,
        /// 连麦人数已满
        TooManyCoHosts,
        /// 已在连麦中
        AlreadyCoHost,
        /// 不在连麦中
        NotCoHost,
        /// 数值溢出
        Overflow,
        /// 无权限
        NotAuthorized,
        /// 标题过长
        TitleTooLong,
        /// 描述过长
        DescriptionTooLong,
        /// CID 过长
        CidTooLong,
        /// 礼物名称过长
        GiftNameTooLong,
        /// 提现金额过低
        WithdrawAmountTooLow,
        /// 收益不足
        InsufficientEarnings,
        /// 直播间状态无效
        InvalidRoomStatus,
        /// 票价无效
        InvalidTicketPrice,
        /// 数量无效
        InvalidQuantity,
        /// 没有超额保证金可提取
        NoBondExcess,
    }

    // ============ 调用函数 ============

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // -------- 直播间管理 --------

        /// 创建直播间
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::create_room())]
        pub fn create_room(
            origin: OriginFor<T>,
            title: Vec<u8>,
            description: Option<Vec<u8>>,
            room_type: LiveRoomType,
            cover_cid: Option<Vec<u8>>,
            ticket_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let host = ensure_signed(origin)?;

            // 检查主播是否已有活跃直播间
            ensure!(!HostRoom::<T>::contains_key(&host), Error::<T>::HostAlreadyHasRoom);

            // 验证标题长度
            let title: BoundedVec<u8, T::MaxTitleLen> =
                title.try_into().map_err(|_| Error::<T>::TitleTooLong)?;

            // 验证描述长度
            let description: Option<BoundedVec<u8, T::MaxDescriptionLen>> = description
                .map(|d| d.try_into().map_err(|_| Error::<T>::DescriptionTooLong))
                .transpose()?;

            // 验证封面 CID 长度
            let cover_cid: Option<BoundedVec<u8, T::MaxCidLen>> = cover_cid
                .map(|c| c.try_into().map_err(|_| Error::<T>::CidTooLong))
                .transpose()?;

            // 付费直播必须设置票价
            if room_type == LiveRoomType::Paid {
                ensure!(
                    ticket_price.map(|p| !p.is_zero()).unwrap_or(false),
                    Error::<T>::InvalidTicketPrice
                );
            }

            // 锁定保证金：使用统一的 DepositCalculator 计算
            let bond = Self::calculate_room_bond();
            T::Currency::reserve(&host, bond)?;

            // 生成直播间 ID
            let room_id = NextRoomId::<T>::get();
            NextRoomId::<T>::put(room_id.saturating_add(1));
            
            // 记录保证金
            RoomDeposits::<T>::insert(room_id, (host.clone(), bond));

            let current_block = <frame_system::Pallet<T>>::block_number();

            // 创建直播间
            let room = LiveRoom {
                id: room_id,
                host: host.clone(),
                title,
                description,
                room_type: room_type.clone(),
                status: LiveRoomStatus::Preparing,
                cover_cid,
                total_viewers: 0,
                peak_viewers: 0,
                total_gifts: Zero::zero(),
                ticket_price,
                created_at: current_block.try_into().unwrap_or(0),
                started_at: None,
                ended_at: None,
            };

            LiveRooms::<T>::insert(room_id, room);
            HostRoom::<T>::insert(&host, room_id);

            // 注意：不 PIN 直播间封面到 IPFS
            // 原因：直播间是临时数据，直播结束后封面即废弃，PIN 会造成存储浪费
            // 前端应直接上传到公共 IPFS 网关，链上仅存 CID 引用

            Self::deposit_event(Event::RoomCreated {
                host,
                room_id,
                room_type,
            });

            Ok(())
        }

        /// 开始直播
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::start_live())]
        pub fn start_live(origin: OriginFor<T>, room_id: u64) -> DispatchResult {
            let host = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                ensure!(room.host == host, Error::<T>::NotRoomHost);
                ensure!(
                    room.status == LiveRoomStatus::Preparing || room.status == LiveRoomStatus::Paused,
                    Error::<T>::InvalidRoomStatus
                );

                let current_block = <frame_system::Pallet<T>>::block_number();
                room.status = LiveRoomStatus::Live;
                room.started_at = Some(current_block.try_into().unwrap_or(0));

                Self::deposit_event(Event::LiveStarted {
                    room_id,
                    started_at: current_block,
                });

                Ok(())
            })
        }

        /// 暂停直播
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::pause_live())]
        pub fn pause_live(origin: OriginFor<T>, room_id: u64) -> DispatchResult {
            let host = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                ensure!(room.host == host, Error::<T>::NotRoomHost);
                ensure!(room.status == LiveRoomStatus::Live, Error::<T>::RoomNotLive);

                room.status = LiveRoomStatus::Paused;

                Self::deposit_event(Event::LivePaused { room_id });

                Ok(())
            })
        }

        /// 恢复直播
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::resume_live())]
        pub fn resume_live(origin: OriginFor<T>, room_id: u64) -> DispatchResult {
            let host = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                ensure!(room.host == host, Error::<T>::NotRoomHost);
                ensure!(room.status == LiveRoomStatus::Paused, Error::<T>::InvalidRoomStatus);

                room.status = LiveRoomStatus::Live;

                Self::deposit_event(Event::LiveResumed { room_id });

                Ok(())
            })
        }

        /// 结束直播
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::end_live())]
        pub fn end_live(origin: OriginFor<T>, room_id: u64) -> DispatchResult {
            let host = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                ensure!(room.host == host, Error::<T>::NotRoomHost);
                ensure!(
                    room.status == LiveRoomStatus::Live || room.status == LiveRoomStatus::Paused,
                    Error::<T>::InvalidRoomStatus
                );

                let current_block: u64 = <frame_system::Pallet<T>>::block_number()
                    .try_into()
                    .unwrap_or(0);
                let started_at = room.started_at.unwrap_or(current_block);
                let duration = current_block.saturating_sub(started_at);

                room.status = LiveRoomStatus::Ended;
                room.ended_at = Some(current_block);

                // 清除连麦者
                ActiveCoHosts::<T>::remove(room_id);

                // 移除主播的活跃直播间记录
                HostRoom::<T>::remove(&host);

                // 解锁保证金
                if let Some((depositor, amount)) = RoomDeposits::<T>::take(room_id) {
                    T::Currency::unreserve(&depositor, amount);
                }

                Self::deposit_event(Event::LiveEnded {
                    room_id,
                    duration,
                    total_viewers: room.total_viewers,
                    peak_viewers: room.peak_viewers,
                    total_gifts: room.total_gifts,
                });

                Ok(())
            })
        }

        /// 更新直播间信息
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::update_room())]
        pub fn update_room(
            origin: OriginFor<T>,
            room_id: u64,
            title: Option<Vec<u8>>,
            description: Option<Vec<u8>>,
            cover_cid: Option<Vec<u8>>,
        ) -> DispatchResult {
            let host = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                ensure!(room.host == host, Error::<T>::NotRoomHost);
                ensure!(room.status != LiveRoomStatus::Ended, Error::<T>::RoomEnded);
                ensure!(room.status != LiveRoomStatus::Banned, Error::<T>::RoomBanned);

                if let Some(new_title) = title {
                    room.title = new_title.try_into().map_err(|_| Error::<T>::TitleTooLong)?;
                }

                if let Some(new_desc) = description {
                    room.description =
                        Some(new_desc.try_into().map_err(|_| Error::<T>::DescriptionTooLong)?);
                }

                if let Some(new_cid) = cover_cid {
                    room.cover_cid = Some(new_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?);
                }

                Self::deposit_event(Event::RoomUpdated { room_id });

                Ok(())
            })
        }


        // -------- 门票系统 --------

        /// 购买付费直播门票
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::buy_ticket())]
        pub fn buy_ticket(origin: OriginFor<T>, room_id: u64) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;

            // 检查是否为付费直播
            ensure!(room.room_type == LiveRoomType::Paid, Error::<T>::InvalidRoomStatus);
            ensure!(room.status != LiveRoomStatus::Ended, Error::<T>::RoomEnded);
            ensure!(room.status != LiveRoomStatus::Banned, Error::<T>::RoomBanned);

            // 检查是否已购票
            ensure!(
                !TicketHolders::<T>::contains_key(room_id, &buyer),
                Error::<T>::AlreadyHasTicket
            );

            // 获取票价
            let price = room.ticket_price.ok_or(Error::<T>::InvalidTicketPrice)?;

            // 转账给主播
            T::Currency::transfer(&buyer, &room.host, price, ExistenceRequirement::KeepAlive)?;

            // 记录购票
            let current_block: u64 = <frame_system::Pallet<T>>::block_number()
                .try_into()
                .unwrap_or(0);
            TicketHolders::<T>::insert(room_id, &buyer, current_block);

            Self::deposit_event(Event::TicketPurchased {
                room_id,
                buyer,
                price,
            });

            Ok(())
        }

        // -------- 礼物系统 --------

        /// 发送礼物
        #[pallet::call_index(20)]
        #[pallet::weight(T::WeightInfo::send_gift())]
        pub fn send_gift(
            origin: OriginFor<T>,
            room_id: u64,
            gift_id: u32,
            quantity: u32,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(quantity > 0, Error::<T>::InvalidQuantity);

            // 检查直播间状态
            let mut room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.status == LiveRoomStatus::Live, Error::<T>::RoomNotLive);

            // 检查礼物存在且启用
            let gift = Gifts::<T>::get(gift_id).ok_or(Error::<T>::GiftNotFound)?;
            ensure!(gift.enabled, Error::<T>::GiftDisabled);

            // 计算总价 (防溢出)
            let total = gift
                .price
                .checked_mul(&quantity.into())
                .ok_or(Error::<T>::Overflow)?;

            // 计算分成
            let platform_fee_percent: BalanceOf<T> = T::PlatformFeePercent::get().into();
            let hundred: BalanceOf<T> = 100u32.into();
            let platform_fee = total
                .saturating_mul(platform_fee_percent)
                .checked_div(&hundred)
                .unwrap_or(Zero::zero());
            let host_amount = total.saturating_sub(platform_fee);

            // 转账给主播
            T::Currency::transfer(
                &sender,
                &room.host,
                host_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            // 转账给国库 (模块账户)
            let treasury = Self::account_id();
            T::Currency::transfer(&sender, &treasury, platform_fee, ExistenceRequirement::KeepAlive)?;

            // 更新统计
            room.total_gifts = room.total_gifts.saturating_add(total);
            LiveRooms::<T>::insert(room_id, room.clone());

            HostEarnings::<T>::mutate(&room.host, |e| *e = e.saturating_add(host_amount));
            UserRoomGifts::<T>::mutate(room_id, &sender, |g| *g = g.saturating_add(total));

            Self::deposit_event(Event::GiftSent {
                room_id,
                sender,
                receiver: room.host,
                gift_id,
                quantity,
                value: total,
            });

            Ok(())
        }

        /// 主播提现收益
        ///
        /// 注意: 收益在 send_gift 时已经直接转账给主播，HostEarnings 只是记录累计收益。
        /// 此函数用于清零收益记录（例如用于统计目的），实际资金已在主播账户中。
        #[pallet::call_index(21)]
        #[pallet::weight(T::WeightInfo::withdraw_earnings())]
        pub fn withdraw_earnings(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let host = ensure_signed(origin)?;

            ensure!(amount >= T::MinWithdrawAmount::get(), Error::<T>::WithdrawAmountTooLow);

            let earnings = HostEarnings::<T>::get(&host);
            ensure!(earnings >= amount, Error::<T>::InsufficientEarnings);

            // 扣除收益记录 (资金已在 send_gift 时转账给主播)
            HostEarnings::<T>::mutate(&host, |e| *e = e.saturating_sub(amount));

            Self::deposit_event(Event::EarningsWithdrawn { host, amount });

            Ok(())
        }

        /// 同步直播统计数据 (后端调用)
        #[pallet::call_index(22)]
        #[pallet::weight(T::WeightInfo::sync_live_stats())]
        pub fn sync_live_stats(
            origin: OriginFor<T>,
            room_id: u64,
            total_viewers: u64,
            peak_viewers: u32,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                // 只有房主可以同步统计
                ensure!(room.host == caller, Error::<T>::NotRoomHost);

                room.total_viewers = total_viewers;
                room.peak_viewers = peak_viewers;

                Self::deposit_event(Event::LiveStatsSynced {
                    room_id,
                    total_viewers,
                    peak_viewers,
                });

                Ok(())
            })
        }

        // -------- 管理功能 --------

        /// 踢出观众并加入黑名单
        #[pallet::call_index(30)]
        #[pallet::weight(T::WeightInfo::kick_viewer())]
        pub fn kick_viewer(
            origin: OriginFor<T>,
            room_id: u64,
            viewer: T::AccountId,
        ) -> DispatchResult {
            let host = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.host == host, Error::<T>::NotRoomHost);

            // 检查是否已在黑名单
            ensure!(
                !RoomBlacklist::<T>::contains_key(room_id, &viewer),
                Error::<T>::AlreadyInBlacklist
            );

            RoomBlacklist::<T>::insert(room_id, &viewer, ());

            Self::deposit_event(Event::ViewerKicked { room_id, viewer });

            Ok(())
        }

        /// 从黑名单移除
        #[pallet::call_index(31)]
        #[pallet::weight(T::WeightInfo::remove_from_blacklist())]
        pub fn remove_from_blacklist(
            origin: OriginFor<T>,
            room_id: u64,
            viewer: T::AccountId,
        ) -> DispatchResult {
            let host = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.host == host, Error::<T>::NotRoomHost);

            // 检查是否在黑名单
            ensure!(
                RoomBlacklist::<T>::contains_key(room_id, &viewer),
                Error::<T>::NotInBlacklist
            );

            RoomBlacklist::<T>::remove(room_id, &viewer);

            Self::deposit_event(Event::ViewerUnbanned { room_id, viewer });

            Ok(())
        }

        /// 封禁直播间 (管理员)
        /// 
        /// 封禁时罚没全部保证金到平台国库
        #[pallet::call_index(40)]
        #[pallet::weight(T::WeightInfo::ban_room())]
        pub fn ban_room(
            origin: OriginFor<T>,
            room_id: u64,
            reason: Vec<u8>,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            LiveRooms::<T>::try_mutate(room_id, |maybe_room| -> DispatchResult {
                let room = maybe_room.as_mut().ok_or(Error::<T>::RoomNotFound)?;

                room.status = LiveRoomStatus::Banned;

                // 清除连麦者
                ActiveCoHosts::<T>::remove(room_id);

                // 移除主播的活跃直播间记录
                HostRoom::<T>::remove(&room.host);

                // 🆕 罚没保证金到平台国库
                if let Some((host, bond_amount)) = RoomDeposits::<T>::take(room_id) {
                    // 解锁保证金
                    let actually_slashed = T::Currency::unreserve(&host, bond_amount);
                    // 转入平台国库
                    if !actually_slashed.is_zero() {
                        let _ = T::Currency::transfer(
                            &host,
                            &Self::account_id(),
                            actually_slashed,
                            ExistenceRequirement::AllowDeath,
                        );
                    }
                    Self::deposit_event(Event::BondConfiscated {
                        room_id,
                        host: host.clone(),
                        amount: actually_slashed,
                    });
                }

                Self::deposit_event(Event::RoomBanned {
                    room_id,
                    reason,
                });

                Ok(())
            })
        }

        /// 处理直播间违规（治理权限）
        /// 
        /// 根据违规类型按比例扣除保证金，不封禁直播间
        #[pallet::call_index(41)]
        #[pallet::weight(T::WeightInfo::ban_room())]
        pub fn handle_room_violation(
            origin: OriginFor<T>,
            room_id: u64,
            violation_type: LiveRoomViolationType,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.status != LiveRoomStatus::Banned, Error::<T>::RoomBanned);

            // 获取保证金
            let (host, bond_amount) = RoomDeposits::<T>::get(room_id)
                .ok_or(Error::<T>::InsufficientBalance)?;

            // 计算扣除金额
            let slash_bps = violation_type.slash_bps();
            let slash_amount = bond_amount.saturating_mul(slash_bps.into()) / 10000u32.into();

            if !slash_amount.is_zero() {
                // 解锁保证金
                let actually_slashed = T::Currency::unreserve(&host, slash_amount);
                
                // 转入平台国库
                if !actually_slashed.is_zero() {
                    let _ = T::Currency::transfer(
                        &host,
                        &Self::account_id(),
                        actually_slashed,
                        ExistenceRequirement::AllowDeath,
                    );
                }

                // 更新保证金记录
                let remaining = bond_amount.saturating_sub(actually_slashed);
                if remaining.is_zero() {
                    RoomDeposits::<T>::remove(room_id);
                } else {
                    RoomDeposits::<T>::insert(room_id, (host.clone(), remaining));
                }

                Self::deposit_event(Event::RoomBondSlashed {
                    room_id,
                    host,
                    amount: actually_slashed,
                });
            }

            Ok(())
        }

        /// 补充直播间保证金
        #[pallet::call_index(42)]
        #[pallet::weight(T::WeightInfo::create_room())]
        pub fn top_up_room_bond(
            origin: OriginFor<T>,
            room_id: u64,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.host == who, Error::<T>::NotRoomHost);
            ensure!(room.status != LiveRoomStatus::Banned, Error::<T>::RoomBanned);

            // 锁定保证金
            T::Currency::reserve(&who, amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;

            // 更新保证金记录
            let (host, current) = RoomDeposits::<T>::get(room_id)
                .unwrap_or((who.clone(), Zero::zero()));
            let new_total = current.saturating_add(amount);
            RoomDeposits::<T>::insert(room_id, (host, new_total));

            Ok(())
        }

        // -------- 连麦功能 --------

        /// 开始连麦 (主播调用)
        #[pallet::call_index(50)]
        #[pallet::weight(T::WeightInfo::start_co_host())]
        pub fn start_co_host(
            origin: OriginFor<T>,
            room_id: u64,
            co_host: T::AccountId,
        ) -> DispatchResult {
            let host = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;
            ensure!(room.host == host, Error::<T>::NotRoomHost);
            ensure!(room.status == LiveRoomStatus::Live, Error::<T>::RoomNotLive);

            ActiveCoHosts::<T>::try_mutate(room_id, |co_hosts| -> DispatchResult {
                ensure!(
                    (co_hosts.len() as u32) < T::MaxCoHostsPerRoom::get(),
                    Error::<T>::TooManyCoHosts
                );
                ensure!(!co_hosts.contains(&co_host), Error::<T>::AlreadyCoHost);

                co_hosts
                    .try_push(co_host.clone())
                    .map_err(|_| Error::<T>::TooManyCoHosts)?;

                Ok(())
            })?;

            Self::deposit_event(Event::CoHostStarted { room_id, co_host });

            Ok(())
        }

        /// 结束连麦 (主播或连麦者调用)
        #[pallet::call_index(51)]
        #[pallet::weight(T::WeightInfo::end_co_host())]
        pub fn end_co_host(
            origin: OriginFor<T>,
            room_id: u64,
            co_host: Option<T::AccountId>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;

            let room = LiveRooms::<T>::get(room_id).ok_or(Error::<T>::RoomNotFound)?;

            // 确定要移除的连麦者
            let target = co_host.unwrap_or(caller.clone());

            // 验证权限: 房主可以移除任何人，连麦者只能移除自己
            if caller != room.host {
                ensure!(caller == target, Error::<T>::NotAuthorized);
            }

            ActiveCoHosts::<T>::try_mutate(room_id, |co_hosts| -> DispatchResult {
                let pos = co_hosts
                    .iter()
                    .position(|x| x == &target)
                    .ok_or(Error::<T>::NotCoHost)?;
                co_hosts.remove(pos);
                Ok(())
            })?;

            Self::deposit_event(Event::CoHostEnded {
                room_id,
                co_host: target,
            });

            Ok(())
        }

        // -------- 礼物管理 (管理员) --------

        /// 创建礼物 (管理员)
        #[pallet::call_index(60)]
        #[pallet::weight(T::WeightInfo::create_gift())]
        pub fn create_gift(
            origin: OriginFor<T>,
            name: Vec<u8>,
            price: BalanceOf<T>,
            icon_cid: Vec<u8>,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            let name: BoundedVec<u8, T::MaxGiftNameLen> =
                name.try_into().map_err(|_| Error::<T>::GiftNameTooLong)?;
            let icon_cid: BoundedVec<u8, T::MaxCidLen> =
                icon_cid.try_into().map_err(|_| Error::<T>::CidTooLong)?;

            let gift_id = NextGiftId::<T>::get();
            NextGiftId::<T>::put(gift_id.saturating_add(1));

            let gift = Gift {
                id: gift_id,
                name,
                price,
                icon_cid,
                enabled: true,
            };

            Gifts::<T>::insert(gift_id, gift);

            Self::deposit_event(Event::GiftCreated { gift_id, price });

            Ok(())
        }

        /// 更新礼物状态 (管理员)
        #[pallet::call_index(61)]
        #[pallet::weight(T::WeightInfo::update_gift())]
        pub fn update_gift(
            origin: OriginFor<T>,
            gift_id: u32,
            enabled: bool,
        ) -> DispatchResult {
            T::GovernanceOrigin::ensure_origin(origin)?;

            Gifts::<T>::try_mutate(gift_id, |maybe_gift| -> DispatchResult {
                let gift = maybe_gift.as_mut().ok_or(Error::<T>::GiftNotFound)?;
                gift.enabled = enabled;
                Ok(())
            })?;

            Self::deposit_event(Event::GiftUpdated { gift_id, enabled });

            Ok(())
        }

        // 注：举报功能已迁移到统一仲裁模块 (pallet-arbitration)
        // 使用 arbitration.file_complaint 替代原有的 report_room 等函数

        /// 提取超额保证金
        /// 
        /// 当 DUST 价值上涨后，主播可以提取超过所需 USD 价值的保证金部分。
        /// 保证金将调整为当前价格下 5 USDT 对应的 DUST 数量。
        #[pallet::call_index(62)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn withdraw_excess_bond(
            origin: OriginFor<T>,
            room_id: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // 验证直播间存在
            let room = LiveRooms::<T>::get(room_id)
                .ok_or(Error::<T>::RoomNotFound)?;
            
            // 验证调用者是主播
            ensure!(room.host == who, Error::<T>::NotRoomHost);
            
            // 获取当前保证金
            let (host, current_bond) = RoomDeposits::<T>::get(room_id)
                .ok_or(Error::<T>::RoomNotFound)?;
            
            // 计算当前所需保证金（使用统一的 DepositCalculator）
            let required_bond = Self::calculate_room_bond();
            
            // 计算可提取的超额部分
            ensure!(current_bond > required_bond, Error::<T>::NoBondExcess);
            let excess = current_bond.saturating_sub(required_bond);
            
            // 解锁超额保证金
            T::Currency::unreserve(&host, excess);
            
            // 更新保证金记录
            RoomDeposits::<T>::insert(room_id, (host.clone(), required_bond));
            
            Self::deposit_event(Event::ExcessBondWithdrawn {
                room_id,
                host,
                withdrawn: excess,
                remaining: required_bond,
            });
            
            Ok(())
        }
    }

    // ============ 辅助函数 ============

    impl<T: Config> Pallet<T> {
        /// 获取模块账户 ID (国库)
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// 检查用户是否有门票
        pub fn has_ticket(room_id: u64, user: &T::AccountId) -> bool {
            TicketHolders::<T>::contains_key(room_id, user)
        }

        /// 检查用户是否在黑名单
        pub fn is_blacklisted(room_id: u64, user: &T::AccountId) -> bool {
            RoomBlacklist::<T>::contains_key(room_id, user)
        }

        /// 获取直播间状态
        pub fn room_status(room_id: u64) -> Option<LiveRoomStatus> {
            LiveRooms::<T>::get(room_id).map(|r| r.status)
        }

        /// 获取直播间信息 (用于 RuntimeApi)
        pub fn get_room_info(
            room_id: u64,
        ) -> Option<runtime_api::LiveRoomInfo<T::AccountId, BalanceOf<T>>> {
            LiveRooms::<T>::get(room_id).map(|room| runtime_api::LiveRoomInfo {
                id: room.id,
                host: room.host,
                title: room.title.into_inner(),
                description: room.description.map(|d| d.into_inner()),
                room_type: match room.room_type {
                    LiveRoomType::Normal => 0,
                    LiveRoomType::Paid => 1,
                    LiveRoomType::Private => 2,
                    LiveRoomType::MultiHost => 3,
                },
                status: match room.status {
                    LiveRoomStatus::Preparing => 0,
                    LiveRoomStatus::Live => 1,
                    LiveRoomStatus::Paused => 2,
                    LiveRoomStatus::Ended => 3,
                    LiveRoomStatus::Banned => 4,
                },
                cover_cid: room.cover_cid.map(|c| c.into_inner()),
                total_viewers: room.total_viewers,
                peak_viewers: room.peak_viewers,
                total_gifts: room.total_gifts,
                ticket_price: room.ticket_price,
                created_at: room.created_at,
                started_at: room.started_at,
                ended_at: room.ended_at,
            })
        }

        /// 获取礼物信息 (用于 RuntimeApi)
        pub fn get_gift_info(gift_id: u32) -> Option<runtime_api::GiftInfo<BalanceOf<T>>> {
            Gifts::<T>::get(gift_id).map(|gift| runtime_api::GiftInfo {
                id: gift.id,
                name: gift.name.into_inner(),
                price: gift.price,
                icon_cid: gift.icon_cid.into_inner(),
                enabled: gift.enabled,
            })
        }

        /// 获取所有启用的礼物 (用于 RuntimeApi)
        pub fn get_enabled_gifts() -> Vec<runtime_api::GiftInfo<BalanceOf<T>>> {
            Gifts::<T>::iter()
                .filter_map(|(_, gift)| {
                    if gift.enabled {
                        Some(runtime_api::GiftInfo {
                            id: gift.id,
                            name: gift.name.into_inner(),
                            price: gift.price,
                            icon_cid: gift.icon_cid.into_inner(),
                            enabled: gift.enabled,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        }

        /// 获取所有活跃直播间 ID (直播中)
        pub fn get_live_room_ids() -> Vec<u64> {
            LiveRooms::<T>::iter()
                .filter_map(|(room_id, room)| {
                    if room.status == LiveRoomStatus::Live {
                        Some(room_id)
                    } else {
                        None
                    }
                })
                .collect()
        }

        /// 获取直播间连麦者列表
        pub fn get_co_host_list(room_id: u64) -> Vec<T::AccountId> {
            ActiveCoHosts::<T>::get(room_id).into_inner()
        }
    }

    // ==================== 🆕 仲裁集成：保证金扣除接口 ====================

    impl<T: Config> Pallet<T> {
        /// 投诉裁决后扣除主播保证金
        /// 
        /// ## 参数
        /// - `room_id`: 直播间ID
        /// - `slash_bps`: 扣除比例（基点，5000 = 50%）
        /// - `to_complainant`: 赔付目标账户（投诉方）
        /// 
        /// ## 返回
        /// - `Ok(slashed_amount)`: 实际扣除金额
        pub fn slash_room_bond(
            room_id: u64,
            slash_bps: u16,
            to_complainant: Option<&T::AccountId>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let (host, bond_amount) = RoomDeposits::<T>::get(room_id)
                .ok_or(Error::<T>::RoomNotFound)?;
            
            // 计算扣除金额
            let slash_amount = sp_runtime::Permill::from_parts((slash_bps as u32) * 100)
                .mul_floor(bond_amount);
            
            if slash_amount.is_zero() {
                return Ok(Zero::zero());
            }
            
            // 从保证金中扣除（unreserve 后转移）
            let actually_slashed = T::Currency::unreserve(&host, slash_amount);
            
            // 赔付给投诉方
            if let Some(complainant) = to_complainant {
                if !actually_slashed.is_zero() {
                    let _ = T::Currency::transfer(
                        &host,
                        complainant,
                        actually_slashed,
                        ExistenceRequirement::AllowDeath,
                    );
                }
            }
            
            // 更新保证金记录
            let remaining = bond_amount.saturating_sub(actually_slashed);
            if remaining.is_zero() {
                RoomDeposits::<T>::remove(room_id);
            } else {
                RoomDeposits::<T>::insert(room_id, (host.clone(), remaining));
            }
            
            Self::deposit_event(Event::RoomBondSlashed {
                room_id,
                host,
                amount: actually_slashed,
            });
            
            Ok(actually_slashed)
        }

        /// 计算直播间保证金金额（5 USDT 等值的 DUST）
        /// 
        /// 使用统一的 DepositCalculator trait 计算
        pub fn calculate_room_bond() -> BalanceOf<T> {
            use pallet_trading_common::DepositCalculator;
            T::DepositCalculator::calculate_deposit(
                T::RoomBondUsd::get(),
                T::RoomBond::get(),
            )
        }
    }
}
