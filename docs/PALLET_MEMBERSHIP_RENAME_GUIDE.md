# pallet-membership 重命名为 pallet-divination-membership 指南

**日期**: 2026-01-25  
**原因**: 更明确地表示这是占卜平台专用的会员模块，避免与其他可能的 membership 模块混淆

---

## 一、重命名理由

### ✅ 优势

1. **命名更清晰**
   - `pallet-divination-membership` 明确表示这是占卜模块的一部分
   - 避免与 Substrate 官方的 `pallet_collective_membership` 混淆
   - 符合项目组织结构（位于 `pallets/divination/membership/`）

2. **避免命名冲突**
   - 如果未来需要添加其他会员系统（如婚恋模块会员），命名不会冲突
   - 更符合 Rust 命名规范（模块路径与包名一致）

3. **代码可读性**
   - 从包名就能看出模块归属
   - 文档和注释更清晰

---

## 二、需要修改的文件清单

### 🔴 必须修改（核心文件）

#### 1. 包定义文件

**文件**: `pallets/divination/membership/Cargo.toml`
```toml
# 修改前
[package]
name = "pallet-membership"

# 修改后
[package]
name = "pallet-divination-membership"
```

#### 2. 工作区依赖配置

**文件**: `Cargo.toml` (根目录)
```toml
# 修改前
pallet-membership = { path = "./pallets/divination/membership", default-features = false }

# 修改后
pallet-divination-membership = { path = "./pallets/divination/membership", default-features = false }
```

**注意**: 第 160 行有一个错误的配置：
```toml
# ❌ 错误配置（需要删除或修正）
pallet-collective-membership = { package = "pallet-membership", version = "45.0.0", default-features = false }
```
这行配置是错误的，应该使用官方的 `pallet-collective-membership`，而不是指向 `pallet-membership`。

#### 3. Mock 测试文件

**文件**: `pallets/divination/membership/src/mock.rs`
```rust
// 修改前
use crate as pallet_membership;
// ...
pub struct Test;
construct_runtime!(
    pub struct Test {
        // ...
        Membership: pallet_membership,
    }
);
impl pallet_membership::Config for Test {

// 修改后
use crate as pallet_divination_membership;
// ...
pub struct Test;
construct_runtime!(
    pub struct Test {
        // ...
        Membership: pallet_divination_membership,
    }
);
impl pallet_divination_membership::Config for Test {
```

### 🟡 建议修改（文档和注释）

#### 4. README 文档

**文件**: `pallets/divination/membership/README.md`
```markdown
# 修改前
# 会员系统模块 (pallet-membership)

# 修改后
# 会员系统模块 (pallet-divination-membership)
```

```markdown
# 修改前
pallet-membership = { path = "../membership", default-features = false }

# 修改后
pallet-divination-membership = { path = "../membership", default-features = false }
```

#### 5. 注释中的引用

**文件**: `pallets/divination/affiliate/src/lib.rs` (第 1598-1599 行)
```rust
// 修改前
/// - pallet-membership::purchase() 购买会员时调用
/// - pallet-membership::upgrade_to_year10() 升级会员时调用

// 修改后
/// - pallet-divination-membership::purchase() 购买会员时调用
/// - pallet-divination-membership::upgrade_to_year10() 升级会员时调用
```

**文件**: `pallets/divination/affiliate/README.md`
```markdown
# 修改前
### 与 pallet-membership 集成
// pallet-membership 实现 MembershipProvider
- `pallet-membership`: 会员系统

# 修改后
### 与 pallet-divination-membership 集成
// pallet-divination-membership 实现 MembershipProvider
- `pallet-divination-membership`: 会员系统
```

**文件**: `pallets/referral/README.md` (如果存在相关引用)
```markdown
# 修改前
│         pallet-membership / pallet-affiliate            │
        pallet_membership::Pallet::<Runtime>::is_member(who)

# 修改后
│         pallet-divination-membership / pallet-affiliate            │
        pallet_divination_membership::Pallet::<Runtime>::is_member(who)
```

### 🟢 可选修改（如果已集成到 runtime）

#### 6. Runtime 配置（如果已集成）

**文件**: `runtime/Cargo.toml`
```toml
# 如果存在，需要添加
pallet-divination-membership = { workspace = true }
```

**文件**: `runtime/Cargo.toml` (features 部分)
```toml
# 如果存在，需要添加
"pallet-divination-membership/std",
"pallet-divination-membership/runtime-benchmarks",
"pallet-divination-membership/try-runtime",
```

**文件**: `runtime/src/configs/mod.rs` (如果已配置)
```rust
// 如果存在，需要修改
impl pallet_divination_membership::Config for Runtime {
    // ...
}
```

**文件**: `runtime/src/lib.rs` (如果已注册)
```rust
// 如果存在，需要修改
#[runtime::pallet_index(XX)]
pub type Membership = pallet_divination_membership;
```

---

## 三、重命名步骤

### 步骤 1: 修改包名

```bash
# 1. 修改 Cargo.toml
cd pallets/divination/membership
# 编辑 Cargo.toml，修改 name = "pallet-divination-membership"
```

### 步骤 2: 修改工作区配置

```bash
# 2. 修改根目录 Cargo.toml
cd ../../..
# 编辑 Cargo.toml，修改依赖名称
```

### 步骤 3: 修改代码引用

```bash
# 3. 修改 mock.rs
cd pallets/divination/membership/src
# 编辑 mock.rs，修改所有 pallet_membership 为 pallet_divination_membership
```

### 步骤 4: 修改文档

```bash
# 4. 修改所有文档中的引用
# 使用 grep 查找所有引用
grep -r "pallet-membership" --include="*.md" --include="*.rs"
# 逐个修改
```

### 步骤 5: 清理和重建

```bash
# 5. 清理构建缓存
cargo clean

# 6. 重新构建
cargo build

# 7. 运行测试
cargo test -p pallet-divination-membership
```

---

## 四、注意事项

### ⚠️ 重要提醒

1. **Cargo.lock 会自动更新**
   - 修改包名后，运行 `cargo build` 会自动更新 `Cargo.lock`
   - 不需要手动修改 `Cargo.lock`

2. **检查所有依赖关系**
   - 确保没有其他 pallet 直接依赖 `pallet-membership`
   - 如果通过 trait 接口使用，可能不需要修改

3. **Runtime 集成状态**
   - 当前检查发现 `runtime/Cargo.toml` 中没有 `pallet-membership` 的依赖
   - 如果未来集成到 runtime，需要使用新名称

4. **向后兼容性**
   - 如果已有链上数据，重命名不会影响存储结构
   - 但需要确保所有节点同时升级

5. **Git 历史**
   - 重命名后，Git 可能认为这是新文件
   - 可以使用 `git mv` 保留历史：
   ```bash
   # 虽然目录名不变，但可以记录重命名
   git add -A
   git commit -m "Rename pallet-membership to pallet-divination-membership"
   ```

---

## 五、验证清单

重命名完成后，请验证：

- [ ] `cargo build` 成功编译
- [ ] `cargo test -p pallet-divination-membership` 测试通过
- [ ] 所有文档中的引用已更新
- [ ] 所有注释中的引用已更新
- [ ] `Cargo.lock` 已自动更新
- [ ] 没有编译警告或错误
- [ ] 如果已集成到 runtime，runtime 编译成功

---

## 六、影响范围评估

### 低风险修改
- ✅ 包名修改（Cargo.toml）
- ✅ 工作区依赖配置
- ✅ 文档和注释

### 中等风险修改
- ⚠️ Mock 测试文件（需要确保测试通过）
- ⚠️ 如果已集成到 runtime，需要修改 runtime 配置

### 高风险修改
- 🔴 如果已有生产链，需要协调升级
- 🔴 如果有外部依赖，需要通知更新

---

## 七、总结

重命名 `pallet-membership` 为 `pallet-divination-membership` 是一个**很好的建议**，因为：

1. ✅ 命名更清晰，明确表示模块归属
2. ✅ 避免与官方模块混淆
3. ✅ 符合项目组织结构
4. ✅ 降低未来命名冲突风险

**建议执行时间**: 在下一个版本发布前完成，避免影响生产环境。

---

**文档版本**: v1.0  
**最后更新**: 2026-01-25

