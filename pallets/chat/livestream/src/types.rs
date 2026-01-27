//! 直播间模块类型定义

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

/// 直播间状态
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum LiveRoomStatus {
    /// 准备中（未开播）
    #[default]
    Preparing,
    /// 直播中
    Live,
    /// 暂停中
    Paused,
    /// 已结束
    Ended,
    /// 被封禁
    Banned,
}

/// 直播间类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum LiveRoomType {
    /// 普通直播
    #[default]
    Normal,
    /// 付费直播（需购票）
    Paid,
    /// 私密直播（仅邀请）
    Private,
    /// 连麦直播
    MultiHost,
}

/// 连麦类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
    Default,
)]
pub enum CoHostType {
    /// 语音连麦
    #[default]
    AudioOnly,
    /// 视频连麦
    VideoAndAudio,
}

/// 直播间违规类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum LiveRoomViolationType {
    /// 轻微违规（5%）- 标题党、轻微误导
    Minor,
    /// 一般违规（10%）- 不当言论
    Moderate,
    /// 严重违规（30%）- 色情、暴力内容
    Severe,
    /// 特别严重（50%）- 诈骗、违法
    Critical,
}

impl LiveRoomViolationType {
    /// 获取扣除比例（基点，10000 = 100%）
    pub fn slash_bps(&self) -> u16 {
        match self {
            LiveRoomViolationType::Minor => 500,       // 5%
            LiveRoomViolationType::Moderate => 1000,   // 10%
            LiveRoomViolationType::Severe => 3000,     // 30%
            LiveRoomViolationType::Critical => 5000,   // 50%
        }
    }
}

/// 直播间信息
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxTitleLen, MaxDescriptionLen, MaxCidLen))]
pub struct LiveRoom<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen>
where
    MaxTitleLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    /// 直播间ID（自增）
    pub id: u64,
    /// 主播账户
    pub host: AccountId,
    /// 直播间标题
    pub title: BoundedVec<u8, MaxTitleLen>,
    /// 直播间描述
    pub description: Option<BoundedVec<u8, MaxDescriptionLen>>,
    /// 直播间类型
    pub room_type: LiveRoomType,
    /// 直播间状态
    pub status: LiveRoomStatus,
    /// 封面图CID (IPFS)
    pub cover_cid: Option<BoundedVec<u8, MaxCidLen>>,
    /// 累计观众数（直播结束时从 LiveKit 同步）
    pub total_viewers: u64,
    /// 峰值观众数
    pub peak_viewers: u32,
    /// 累计礼物收入
    pub total_gifts: Balance,
    /// 付费直播票价（仅Paid类型）
    pub ticket_price: Option<Balance>,
    /// 创建时间（区块号）
    pub created_at: u64,
    /// 开播时间（区块号）
    pub started_at: Option<u64>,
    /// 结束时间（区块号）
    pub ended_at: Option<u64>,
}

// 手动实现 Clone，避免 Get<u32> 类型参数的 Clone 约束
impl<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen> Clone
    for LiveRoom<AccountId, Balance, MaxTitleLen, MaxDescriptionLen, MaxCidLen>
where
    AccountId: Clone,
    Balance: Clone,
    MaxTitleLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            host: self.host.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            room_type: self.room_type,
            status: self.status,
            cover_cid: self.cover_cid.clone(),
            total_viewers: self.total_viewers,
            peak_viewers: self.peak_viewers,
            total_gifts: self.total_gifts.clone(),
            ticket_price: self.ticket_price.clone(),
            created_at: self.created_at,
            started_at: self.started_at,
            ended_at: self.ended_at,
        }
    }
}

/// 礼物定义
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxGiftNameLen, MaxCidLen))]
pub struct Gift<Balance, MaxGiftNameLen, MaxCidLen>
where
    MaxGiftNameLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    /// 礼物ID
    pub id: u32,
    /// 礼物名称
    pub name: BoundedVec<u8, MaxGiftNameLen>,
    /// 礼物价格
    pub price: Balance,
    /// 礼物图标CID
    pub icon_cid: BoundedVec<u8, MaxCidLen>,
    /// 是否启用
    pub enabled: bool,
}

// 手动实现 Clone
impl<Balance, MaxGiftNameLen, MaxCidLen> Clone for Gift<Balance, MaxGiftNameLen, MaxCidLen>
where
    Balance: Clone,
    MaxGiftNameLen: Get<u32>,
    MaxCidLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            price: self.price.clone(),
            icon_cid: self.icon_cid.clone(),
            enabled: self.enabled,
        }
    }
}

// ============ 举报与申诉系统类型 ============

/// 直播间举报类型
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum RoomReportType {
    /// 违规内容（涉黄、暴力等）
    IllegalContent,
    /// 虚假宣传
    FalseAdvertising,
    /// 骚扰观众
    Harassment,
    /// 诈骗行为
    Fraud,
    /// 其他
    Other,
}

/// 举报状态
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum ReportStatus {
    /// 待审核
    Pending,
    /// 审核中
    UnderReview,
    /// 举报成立
    Upheld,
    /// 举报驳回
    Rejected,
    /// 恶意举报
    Malicious,
    /// 已撤回
    Withdrawn,
    /// 已过期
    Expired,
}

/// 申诉结果
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    Clone,
    Copy,
    RuntimeDebug,
    PartialEq,
    Eq,
    TypeInfo,
    MaxEncodedLen,
)]
pub enum AppealResult {
    /// 申诉成立（解除封禁）
    Upheld,
    /// 申诉驳回（维持封禁）
    Rejected,
}

/// 直播间举报记录
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxCidLen, MaxDescriptionLen))]
pub struct RoomReportRecord<AccountId, Balance, BlockNumber, MaxCidLen, MaxDescriptionLen>
where
    MaxCidLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
{
    /// 举报 ID
    pub id: u64,
    /// 举报者账户
    pub reporter: AccountId,
    /// 被举报的直播间 ID
    pub room_id: u64,
    /// 被举报的主播账户
    pub host: AccountId,
    /// 举报类型
    pub report_type: RoomReportType,
    /// 证据 IPFS CID
    pub evidence_cid: BoundedVec<u8, MaxCidLen>,
    /// 举报描述
    pub description: BoundedVec<u8, MaxDescriptionLen>,
    /// 押金金额
    pub deposit: Balance,
    /// 举报状态
    pub status: ReportStatus,
    /// 创建时间（区块号）
    pub created_at: BlockNumber,
    /// 处理时间（区块号）
    pub resolved_at: Option<BlockNumber>,
    /// 是否匿名举报
    pub is_anonymous: bool,
}

// 手动实现 Clone
impl<AccountId, Balance, BlockNumber, MaxCidLen, MaxDescriptionLen> Clone
    for RoomReportRecord<AccountId, Balance, BlockNumber, MaxCidLen, MaxDescriptionLen>
where
    AccountId: Clone,
    Balance: Clone,
    BlockNumber: Clone,
    MaxCidLen: Get<u32>,
    MaxDescriptionLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            reporter: self.reporter.clone(),
            room_id: self.room_id,
            host: self.host.clone(),
            report_type: self.report_type,
            evidence_cid: self.evidence_cid.clone(),
            description: self.description.clone(),
            deposit: self.deposit.clone(),
            status: self.status,
            created_at: self.created_at.clone(),
            resolved_at: self.resolved_at.clone(),
            is_anonymous: self.is_anonymous,
        }
    }
}

/// 直播间封禁记录
#[derive(Encode, Decode, DecodeWithMemTracking, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxDescriptionLen))]
pub struct RoomBanRecord<AccountId, BlockNumber, MaxDescriptionLen>
where
    MaxDescriptionLen: Get<u32>,
{
    /// 被封禁的直播间 ID
    pub room_id: u64,
    /// 被封禁的主播账户
    pub host: AccountId,
    /// 封禁时间（区块号）
    pub banned_at: BlockNumber,
    /// 封禁原因
    pub reason: BoundedVec<u8, MaxDescriptionLen>,
    /// 关联的举报 ID（如果有）
    pub related_report_id: Option<u64>,
    /// 是否已申诉
    pub is_appealed: bool,
    /// 申诉结果
    pub appeal_result: Option<AppealResult>,
}

// 手动实现 Clone
impl<AccountId, BlockNumber, MaxDescriptionLen> Clone
    for RoomBanRecord<AccountId, BlockNumber, MaxDescriptionLen>
where
    AccountId: Clone,
    BlockNumber: Clone,
    MaxDescriptionLen: Get<u32>,
{
    fn clone(&self) -> Self {
        Self {
            room_id: self.room_id,
            host: self.host.clone(),
            banned_at: self.banned_at.clone(),
            reason: self.reason.clone(),
            related_report_id: self.related_report_id,
            is_appealed: self.is_appealed,
            appeal_result: self.appeal_result,
        }
    }
}
