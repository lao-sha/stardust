//! 函数级中文注释：资金托管子模块
//!
//! 功能：
//! - 托管账户管理
//! - 资金存入（deposit）
//! - 资金提取（withdraw）
//! - 余额查询（escrow_balance）
//!
//! 整合自：pallet-affiliate（原托管层）

use super::*;
use frame_support::traits::{Currency, ExistenceRequirement};
use sp_runtime::traits::{AccountIdConversion, Saturating};

/// 函数级中文注释：资金托管功能实现
impl<T: Config> Pallet<T> {
    /// 函数级中文注释：获取托管账户地址
    ///
    /// 使用 EscrowPalletId 派生独立托管账户
    pub fn escrow_account() -> T::AccountId {
        T::EscrowPalletId::get().into_account_truncating()
    }

    /// 函数级中文注释：查询托管账户余额
    pub fn escrow_balance() -> BalanceOf<T> {
        let escrow_account = Self::escrow_account();
        T::Currency::free_balance(&escrow_account)
    }

    /// 函数级中文注释：存入资金到托管账户
    ///
    /// 参数：
    /// - from: 转出账户
    /// - amount: 转账金额
    ///
    /// 返回：Result
    pub fn do_deposit(
        from: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), &'static str> {
        let escrow_account = Self::escrow_account();

        // 转账到托管账户
        T::Currency::transfer(
            from,
            &escrow_account,
            amount,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| "Transfer to escrow failed")?;

        // 更新累计统计
        TotalDeposited::<T>::mutate(|total| {
            *total = total.saturating_add(amount);
        });

        // 发射事件
        Self::deposit_event(Event::Deposited {
            from: from.clone(),
            amount,
        });

        Ok(())
    }

    /// 函数级中文注释：从托管账户提取资金
    ///
    /// 参数：
    /// - to: 接收账户
    /// - amount: 提取金额
    ///
    /// 验证：
    /// - 托管账户余额充足
    ///
    /// 返回：Result
    pub fn do_withdraw(
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), &'static str> {
        let escrow_account = Self::escrow_account();

        // 检查余额
        let balance = T::Currency::free_balance(&escrow_account);
        if balance < amount {
            return Err("Insufficient escrow balance");
        }

        // 从托管账户转出
        T::Currency::transfer(
            &escrow_account,
            to,
            amount,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| "Transfer from escrow failed")?;

        // 更新累计统计
        TotalWithdrawn::<T>::mutate(|total| {
            *total = total.saturating_add(amount);
        });

        // 发射事件
        Self::deposit_event(Event::Withdrawn {
            to: to.clone(),
            amount,
        });

        Ok(())
    }

    /// 函数级中文注释：批量转账（内部辅助函数）
    ///
    /// 用于周结算批量转账
    ///
    /// 参数：
    /// - transfers: (接收账户, 金额) 列表
    ///
    /// 返回：Result
    pub fn do_batch_withdraw(
        transfers: &[(T::AccountId, BalanceOf<T>)],
    ) -> Result<(), &'static str> {
        let escrow_account = Self::escrow_account();

        // 计算总金额
        let mut total_amount = BalanceOf::<T>::zero();
        for (_, amount) in transfers {
            total_amount = total_amount.saturating_add(*amount);
        }

        // 检查余额
        let balance = T::Currency::free_balance(&escrow_account);
        if balance < total_amount {
            return Err("Insufficient escrow balance for batch withdraw");
        }

        // 批量转账
        for (to, amount) in transfers {
            T::Currency::transfer(
                &escrow_account,
                to,
                *amount,
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| "Batch transfer failed")?;
        }

        // 更新累计统计
        TotalWithdrawn::<T>::mutate(|total| {
            *total = total.saturating_add(total_amount);
        });

        Ok(())
    }
}

