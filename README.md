# rs_hsm_

一个纯 Rust 的层级状态机（HSM）处理器，语义对齐参考仓库 `sm_hsm`，并通过 `hsmtst` 轨迹验证。

## 当前成果

- 实现了纯状态机处理器内核，支持：
  - 层级状态
  - 事件自底向上的 `super_` 冒泡
  - `Tran` / self-transition
  - `entry_` / `exit_` / `init_` chain
  - ignored event 判定
- 当前核心实现已经采用**纯状态引用链接**风格：
  - 当前状态保存为 `&'static SM_HsmState<_>`
  - 父状态直接保存为 `super_: Option<&'static SM_HsmState<_>>`
  - `Tran` 与 `init_` 目标都直接返回状态引用
- 对外命名风格已统一向参考仓库 `sm_hsm` 靠拢
- 语义已通过 `hsmtst` 黄金轨迹回归测试验证
- 提供了可直接运行的 `hsmtst` 示例
- 核心库为 `no_std`

## 项目结构

- `src/lib.rs`：HSM 核心处理器
- `examples/hsmtst.rs`：可运行示例入口
- `examples/hsmtst_fixture/mod.rs`：`hsmtst` 状态图定义
- `tests/hsmtst.rs`：`hsmtst` 回归测试

## 核心设计

### 1. 纯状态引用链接

状态之间不是通过 ID 查表，而是直接通过静态状态对象相互链接。

```rust
use rs_hsm_::{SM_HsmTrait, SM_HsmState, SM_RetState, SM_StatePtr};

struct MyTrait;
struct MyObject;
enum MyEvt {
    Start,
}

fn MyState_(me: &mut MyObject, e: &MyEvt) -> SM_RetState<MyTrait> {
    let _ = (me, e);
    SM_RetState::Handled
}

static MyState: SM_HsmState<MyTrait> = SM_HsmState {
    super_: None,
    init_: None,
    entry_: None,
    exit_: None,
    handler_: MyState_,
};

impl SM_HsmTrait for MyTrait {
    type ActiveObject = MyObject;
    type AO_Evt = MyEvt;

    fn TOP_initial(_: &mut Self::ActiveObject) -> SM_StatePtr<Self> {
        &MyState
    }
}
```

### 2. 事件按引用派发

状态机处理器只要求状态图提供一个统一事件类型：

```rust
use rs_hsm_::SM_AssertInfo;

impl SM_HsmTrait for MyTrait {
    type ActiveObject = MyObject;
    type AO_Evt = MyEvt;

    fn TOP_initial(_: &mut Self::ActiveObject) -> SM_StatePtr<Self> {
        &MyState
    }
}
```

因此：

- 简单场景可以直接用事件枚举
- 更复杂的应用可以把事件扩展为“事件枚举 + 负载”
- 库本身不需要修改

### 3. 断言钩子（适配 BSP）

运行时契约检查对齐 Quantum Leaps SST 的 `dbc_assert.h`：故障信息只保留 `module + label`，最终通过
系统级 `SM_onAssert()` 统一处理，默认实现是直接 `panic!`。

应用侧需要自定义行为时，在 BSP/启动阶段安装系统级 hook：

```rust
use rs_hsm_::{SM_AssertInfo, SM_setOnAssert};

fn BSP_onAssert(info: SM_AssertInfo) -> ! {
    // 将断言信息记录到日志/LED/trace buffer，再阻塞或复位
    panic!("[{}:{}] {:?}", info.module, info.label, info);
}

unsafe {
    SM_setOnAssert(BSP_onAssert);
}
```

## 运行与测试

### 运行默认 `hsmtst` 序列 B

```bash
cargo run --example hsmtst
```

### 运行 `hsmtst` 序列 A

```bash
cargo run --example hsmtst -- A
```

### 只看启动初始化轨迹

```bash
cargo run --example hsmtst -- startup
```

### 运行回归测试

```bash
cargo test
```

## `hsmtst` 当前说明

当前 `hsmtst` 示例已经采用和核心库一致、且更接近参考 C 版的命名与组织方式：

- `SmHsmTstSig`：信号枚举
- `SmHsmTstEvt`：事件对象
- `SmHsmTst_s` / `SmHsmTst_s1` / `SmHsmTst_s11` ...：静态状态对象
- `SmHsmTst_s_()` / `SmHsmTst_s1_()` ...：状态处理函数
- `TOP_initial()` 直接返回目标状态引用
- `init_()` 直接返回目标状态引用
- `SM_RetState::Tran(...)` 直接返回目标状态引用
- `super_` 直接链接到父状态

它是当前库最直接的参考样例。

## 事件扩展示例：事件枚举带负载

如果你的应用不满足“只有信号，没有参数”，推荐把事件写成：

- 一个统一的 `AppEvt` 枚举
- 每个复杂事件带自己的 payload
- 如果想保留 C 里的 signal 思维，可以再提供 `sig()` 辅助函数

例如：

```rust
use rs_hsm_::{SM_HsmTrait, SM_HsmState, SM_RetState, SM_StatePtr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSig {
    START_SIG,
    TIMEOUT_SIG,
    DATA_SIG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutEvt {
    pub id: u8,
    pub ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataEvt {
    pub channel: u8,
    pub value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvt {
    START_SIG,
    TIMEOUT_SIG(TimeoutEvt),
    DATA_SIG(DataEvt),
}

impl AppEvt {
    pub fn sig(&self) -> AppSig {
        match self {
            AppEvt::START_SIG => AppSig::START_SIG,
            AppEvt::TIMEOUT_SIG(_) => AppSig::TIMEOUT_SIG,
            AppEvt::DATA_SIG(_) => AppSig::DATA_SIG,
        }
    }
}

struct AppTrait;
struct AppObject {
    last_value: u16,
}

fn AppIdle_(me: &mut AppObject, e: &AppEvt) -> SM_RetState<AppTrait> {
    match e {
        AppEvt::START_SIG => SM_RetState::Tran(&AppRunning),
        AppEvt::DATA_SIG(data) if data.channel == 1 => {
            me.last_value = data.value;
            SM_RetState::Handled
        }
        _ => SM_RetState::Super,
    }
}

fn AppRunning_(me: &mut AppObject, e: &AppEvt) -> SM_RetState<AppTrait> {
    let _ = me;
    match e {
        AppEvt::TIMEOUT_SIG(t) if t.id == 1 && t.ms > 100 => SM_RetState::Tran(&AppIdle),
        _ => SM_RetState::Super,
    }
}

static AppIdle: SM_HsmState<AppTrait> = SM_HsmState {
    super_: None,
    init_: None,
    entry_: None,
    exit_: None,
    handler_: AppIdle_,
};

static AppRunning: SM_HsmState<AppTrait> = SM_HsmState {
    super_: None,
    init_: None,
    entry_: None,
    exit_: None,
    handler_: AppRunning_,
};

impl SM_HsmTrait for AppTrait {
    type ActiveObject = AppObject;
    type AO_Evt = AppEvt;

    fn TOP_initial(_: &mut Self::ActiveObject) -> SM_StatePtr<Self> {
        &AppIdle
    }
}
```

### 这种扩展方式的特点

- 不需要修改库
- 保持 `dispatch(&evt)` 的统一入口
- 比 C 风格的事件基类/手工下转型更安全
- payload 读取直接通过 `match` 完成

## 多状态机组合

当前设计支持应用层把多个状态机组合起来。

例如：

- 状态机 A 的上下文里挂一个状态机 B
- A 在处理某个事件时，原地构造一个给 B 的事件
- 然后同步调用 B 的 `dispatch()`

这适合做“协作式正交效果”或主从状态机联动。

注意：

- 可以在 A 的处理过程中派发到 **另一个状态机 B**
- 不支持在同一个状态机的 `dispatch()` 过程中再次对**自己**调用 `dispatch()`

## 当前状态

当前版本已经具备：

- 可直接复用的 HSM 内核
- 与参考 `sm_hsm`/`hsmtst` 对齐的语义验证
- 纯状态引用链接、且命名风格贴近 `sm_hsm` 的示例
- 可继续扩展到复杂事件模型的应用层接口
