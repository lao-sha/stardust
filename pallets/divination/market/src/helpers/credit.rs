//! # 信用体系辅助函数
//!
//! 本模块包含信用分数计算相关的辅助函数

use crate::*;
use crate::types::*;
use frame_system::pallet_prelude::BlockNumberFor;

#[allow(dead_code)]
impl<T: Config> Pallet<T> {
    // ==================== 信用更新函数 ====================

    /// 在评价提交后更新信用档案
    ///
    /// 由 submit_review 函数内部调用
    pub(crate) fn update_credit_on_review(
        provider: &T::AccountId,
        overall_rating: u8,
        accuracy_rating: u8,
        attitude_rating: u8,
        response_rating: u8,
    ) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                let current_block = <frame_system::Pallet<T>>::block_number();

                // 更新评价计数
                profile.total_reviews = profile.total_reviews.saturating_add(1);

                // 滑动平均更新评分
                let count = profile.total_reviews;
                profile.avg_overall_rating = Self::update_average(
                    profile.avg_overall_rating,
                    overall_rating as u16 * 100,
                    count,
                );
                profile.avg_accuracy_rating = Self::update_average(
                    profile.avg_accuracy_rating,
                    accuracy_rating as u16 * 100,
                    count,
                );
                profile.avg_attitude_rating = Self::update_average(
                    profile.avg_attitude_rating,
                    attitude_rating as u16 * 100,
                    count,
                );
                profile.avg_response_rating = Self::update_average(
                    profile.avg_response_rating,
                    response_rating as u16 * 100,
                    count,
                );

                // 更新好评/差评计数
                if overall_rating == 5 {
                    profile.five_star_count = profile.five_star_count.saturating_add(1);
                    profile.consecutive_positive_days =
                        profile.consecutive_positive_days.saturating_add(1);

                    // 更新修复任务进度
                    Self::update_repair_task_progress(
                        provider,
                        RepairTaskType::GetPositiveReviews,
                        1,
                    );
                } else if overall_rating == 1 {
                    profile.one_star_count = profile.one_star_count.saturating_add(1);
                    profile.consecutive_positive_days = 0;

                    // 差评扣分
                    let deduction = DeductionReason::NegativeReview.default_deduction();
                    profile.total_deductions = profile.total_deductions.saturating_add(deduction);
                    profile.last_deduction_reason = Some(DeductionReason::NegativeReview);
                    profile.last_deduction_at = Some(current_block);
                }

                // 重新计算信用分
                Self::recalculate_credit_score(profile);
                profile.updated_at = current_block;
            }
        });
    }

    /// 在订单完成后更新信用档案
    pub(crate) fn update_credit_on_order_complete(provider: &T::AccountId, was_on_time: bool) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.total_orders = profile.total_orders.saturating_add(1);
                profile.completed_orders = profile.completed_orders.saturating_add(1);

                // 更新完成率
                profile.completion_rate = if profile.total_orders > 0 {
                    ((profile.completed_orders as u32 * 10000) / profile.total_orders as u32) as u16
                } else {
                    10000
                };

                // 更新按时完成率
                if was_on_time {
                    // 简化处理：假设 on_time_rate 基于历史数据
                } else {
                    profile.timeout_count = profile.timeout_count.saturating_add(1);
                }

                // 更新修复任务进度
                Self::update_repair_task_progress(provider, RepairTaskType::CompleteOrders, 1);

                Self::recalculate_credit_score(profile);
                profile.updated_at = <frame_system::Pallet<T>>::block_number();
            }
        });
    }

    /// 在订单取消后更新信用档案
    pub(crate) fn update_credit_on_order_cancel(
        provider: &T::AccountId,
        is_provider_cancel: bool,
    ) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                let current_block = <frame_system::Pallet<T>>::block_number();

                profile.total_orders = profile.total_orders.saturating_add(1);

                if is_provider_cancel {
                    profile.active_cancel_count = profile.active_cancel_count.saturating_add(1);

                    // 扣分
                    let deduction = DeductionReason::OrderCancellation.default_deduction();
                    profile.total_deductions = profile.total_deductions.saturating_add(deduction);
                    profile.last_deduction_reason = Some(DeductionReason::OrderCancellation);
                    profile.last_deduction_at = Some(current_block);
                }

                // 更新取消率
                profile.cancellation_rate = if profile.total_orders > 0 {
                    ((profile.active_cancel_count as u32 * 10000) / profile.total_orders as u32)
                        as u16
                } else {
                    0
                };

                // 更新完成率
                profile.completion_rate = if profile.total_orders > 0 {
                    ((profile.completed_orders as u32 * 10000) / profile.total_orders as u32) as u16
                } else {
                    10000
                };

                Self::recalculate_credit_score(profile);
                profile.updated_at = current_block;
            }
        });
    }

    /// 在悬赏被采纳后更新信用档案
    pub(crate) fn update_credit_on_bounty_adoption(provider: &T::AccountId, rank: u8) {
        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
            if let Some(profile) = maybe_profile {
                profile.bounty_adoption_count = profile.bounty_adoption_count.saturating_add(1);

                // 根据名次给予奖励（减少扣分）
                let bonus = match rank {
                    1 => 10u16, // 第一名 +10 分
                    2 => 5,     // 第二名 +5 分
                    3 => 3,     // 第三名 +3 分
                    _ => 1,     // 参与奖 +1 分
                };

                profile.total_deductions = profile.total_deductions.saturating_sub(bonus);
                Self::recalculate_credit_score(profile);
                profile.updated_at = <frame_system::Pallet<T>>::block_number();
            }
        });
    }

    // ==================== 信用分数计算函数 ====================

    /// 重新计算信用分数
    pub(crate) fn recalculate_credit_score(profile: &mut CreditProfile<BlockNumberFor<T>>) {
        // 计算服务质量分 (满分 350)
        let service_quality = Self::calculate_service_quality_score(profile);

        // 计算行为规范分 (满分 250)
        let behavior = Self::calculate_behavior_score(profile);

        // 计算履约能力分 (满分 300)
        let fulfillment = Self::calculate_fulfillment_score(profile);

        // 计算加分项 (满分 100)
        let bonus = Self::calculate_bonus_score(profile);

        // 更新各维度分数
        profile.service_quality_score = service_quality;
        profile.behavior_score = behavior;
        profile.fulfillment_score = fulfillment;
        profile.bonus_score = bonus;

        // 基础分 500 + 加权分数
        let base_score: u32 = 500;
        let weighted_total = (service_quality as u32 * 35 / 35)
            + (behavior as u32 * 25 / 25)
            + (fulfillment as u32 * 30 / 30)
            + (bonus as u32 * 10 / 10);

        // 映射到 0-500 范围
        let additional = (weighted_total * 500) / 1000;
        let total = base_score.saturating_add(additional);

        // 扣除累计扣分
        let final_score = total.saturating_sub(profile.total_deductions as u32);
        let new_score = final_score.min(1000) as u16;

        let new_level = CreditLevel::from_score(new_score);

        profile.score = new_score;
        profile.level = new_level;

        // 更新最高/最低分
        if new_score > profile.highest_score {
            profile.highest_score = new_score;
        }
        if new_score < profile.lowest_score {
            profile.lowest_score = new_score;
        }
    }

    /// 计算服务质量分 (满分 350)
    fn calculate_service_quality_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let mut score: u32 = 0;

        // 综合评分贡献 (最高 150)
        let rating_score = if profile.avg_overall_rating >= 450 {
            150
        } else if profile.avg_overall_rating >= 400 {
            130
        } else if profile.avg_overall_rating >= 350 {
            100
        } else if profile.avg_overall_rating >= 300 {
            70
        } else if profile.avg_overall_rating > 0 {
            (profile.avg_overall_rating as u32 * 70) / 300
        } else {
            0
        };
        score += rating_score;

        // 好评率 (最高 100)
        let total = profile.five_star_count.saturating_add(profile.one_star_count);
        if total > 0 {
            let positive_rate = (profile.five_star_count as u32 * 10000) / total as u32;
            let rate_score = if positive_rate >= 9500 {
                100
            } else if positive_rate >= 9000 {
                85
            } else if positive_rate >= 8000 {
                70
            } else {
                (positive_rate * 70 / 8000) as u32
            };
            score += rate_score;
        }

        // 评分一致性奖励 (最高 50)
        let consistency = 50u32; // 简化处理
        score += consistency;

        // 差评惩罚
        let penalty = (profile.one_star_count as u32 * 5).min(50);
        score = score.saturating_sub(penalty);

        score.min(350) as u16
    }

    /// 计算行为规范分 (满分 250)
    fn calculate_behavior_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let mut score: u32 = 250; // 起始满分

        // 违规扣分
        let violation_deduction = match profile.violation_count {
            0 => 0,
            1 => 20,
            2..=3 => 50,
            4..=5 => 100,
            _ => 150,
        };
        score = score.saturating_sub(violation_deduction);

        // 投诉扣分
        let complaint_deduction = (profile.complaint_upheld_count as u32 * 15).min(80);
        score = score.saturating_sub(complaint_deduction);

        // 活跃违规额外扣分
        let active_deduction = (profile.active_violations as u32 * 30).min(60);
        score = score.saturating_sub(active_deduction);

        score.min(250) as u16
    }

    /// 计算履约能力分 (满分 300)
    fn calculate_fulfillment_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let mut score: u32 = 0;

        // 完成率 (最高 120)
        let completion_score = if profile.completion_rate >= 9800 {
            120
        } else if profile.completion_rate >= 9500 {
            100
        } else if profile.completion_rate >= 9000 {
            80
        } else {
            (profile.completion_rate as u32 * 80 / 9000) as u32
        };
        score += completion_score;

        // 按时完成率 (最高 80)
        let ontime_score = if profile.on_time_rate >= 9500 {
            80
        } else if profile.on_time_rate >= 9000 {
            65
        } else {
            (profile.on_time_rate as u32 * 65 / 9000) as u32
        };
        score += ontime_score;

        // 响应速度 (最高 60)
        let response_score = if profile.avg_response_blocks == 0 {
            30 // 新用户默认
        } else if profile.avg_response_blocks <= 600 {
            60
        } else if profile.avg_response_blocks <= 1800 {
            50
        } else {
            20
        };
        score += response_score;

        // 取消率惩罚
        let cancel_penalty = if profile.cancellation_rate <= 200 {
            0
        } else if profile.cancellation_rate <= 500 {
            15
        } else {
            30
        };
        score = score.saturating_sub(cancel_penalty);

        score.min(300) as u16
    }

    /// 计算加分项 (满分 100)
    fn calculate_bonus_score(profile: &CreditProfile<BlockNumberFor<T>>) -> u16 {
        let mut score: u32 = 0;

        // 实名认证 (+15)
        if profile.is_verified {
            score += 15;
        }

        // 保证金 (+10)
        if profile.has_deposit {
            score += 10;
        }

        // 资质认证 (每个 +8，最多 40)
        let cert_bonus = (profile.certification_count as u32 * 8).min(40);
        score += cert_bonus;

        // 悬赏被采纳 (每次 +2，最多 20)
        let bounty_bonus = (profile.bounty_adoption_count as u32 * 2).min(20);
        score += bounty_bonus;

        // 连续好评天数 (每 7 天 +3，最多 15)
        let consecutive_bonus = ((profile.consecutive_positive_days as u32 / 7) * 3).min(15);
        score += consecutive_bonus;

        score.min(100) as u16
    }

    // ==================== 工具函数 ====================

    /// 更新滑动平均
    pub(crate) fn update_average(old_avg: u16, new_value: u16, total_count: u32) -> u16 {
        if total_count <= 1 {
            return new_value;
        }

        let old_sum = old_avg as u32 * (total_count - 1);
        let new_sum = old_sum + new_value as u32;
        (new_sum / total_count) as u16
    }

    /// 更新修复任务进度
    pub(crate) fn update_repair_task_progress(
        provider: &T::AccountId,
        task_type: RepairTaskType,
        increment: u32,
    ) {
        RepairTasks::<T>::mutate(provider, |tasks| {
            for task in tasks.iter_mut() {
                if task.task_type == task_type && !task.is_completed {
                    task.current_progress = task.current_progress.saturating_add(increment);

                    if task.current_progress >= task.target_value {
                        task.is_completed = true;
                        task.completed_at = Some(<frame_system::Pallet<T>>::block_number());

                        // 恢复信用分
                        CreditProfiles::<T>::mutate(provider, |maybe_profile| {
                            if let Some(profile) = maybe_profile {
                                profile.total_deductions =
                                    profile.total_deductions.saturating_sub(task.reward_points);
                                Self::recalculate_credit_score(profile);
                            }
                        });

                        Self::deposit_event(Event::CreditRepairCompleted {
                            provider: provider.clone(),
                            task_type,
                            restored_points: task.reward_points,
                        });
                    }

                    break;
                }
            }
        });
    }
}
