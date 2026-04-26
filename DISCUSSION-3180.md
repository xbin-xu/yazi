# DISCUSSION-3180: Windows bash Ctrl+C 终端混乱

- **Discussion:** <https://github.com/sxyazi/yazi/discussions/3180>
- **状态:** 已修复

## 问题描述

Windows 上通过 yazi 的 `shell "bash" --block --confirm` 启动 bash 后，按 Ctrl+C 导致终端混乱（提示符重复、输入不可见、命令拼接）。PowerShell 和直接运行 bash 不受影响。

## 根因

**Windows 上 Ctrl+C 生成 `CTRL_C_EVENT` 广播给控制台组内所有进程。yazi 没有处理该信号，默认行为是 `ExitProcess`，导致 yazi 被 Ctrl+C 终止。**

具体流程：

```
1. yazi 运行中，用户触发 shell "bash" --block
2. Term::stop() → disable_raw_mode() → 恢复 ENABLE_PROCESSED_INPUT
3. cmd.exe /C "bash" → bash 启动
4. 用户按 Ctrl+C
5. 系统向控制台组内所有进程广播 CTRL_C_EVENT
6. bash 收到 → 正常处理（显示 ^C）
7. yazi 也收到 → 默认 handler 调用 ExitProcess → yazi 退出
8. Process::block() 的 child.wait() 提前返回
9. _permit Drop 触发 AppProxy::resume() → yazi 恢复 UI
10. 但 bash 作为孤儿进程仍在运行
11. bash + yazi 同时争抢 stdin → 终端混乱
```

PowerShell 不受影响是因为它是原生 Windows 应用，自行处理 `CTRL_C_EVENT`，不会导致 yazi 的行为变化。bash 通过 cmd.exe 启动后，Ctrl+C 的 `CTRL_C_EVENT` 被广播到整个控制台组，yazi 作为同一控制台组的进程也会收到。

## 修复

仅修改 `yazi-term/src/term.rs`，添加 `SetConsoleCtrlHandler` 生命周期管理：

- **`Term::stop()`** — 阻塞 shell 启动前，安装 handler 忽略 `CTRL_C_EVENT`，只让子进程处理 Ctrl+C
- **`Term::start()`** — shell 退出后 yazi 恢复时，移除 handler，恢复正常 Ctrl+C 行为

### 改动内容

`yazi-term/Cargo.toml` — 添加 Windows 依赖：

```toml
[target."cfg(windows)".dependencies]
windows-sys = { version = "0.61.2", features = [ "Win32_System_Console" ] }
```

`yazi-term/src/term.rs` — 三处改动：

1. `Term::start()` 顶部添加 `Self::disable_ctrl_c()`（移除 handler）
2. `Term::stop()` 的 Windows 分支中添加 `Self::enable_ctrl_c()`（安装 handler）
3. 新增三个 `#[cfg(windows)]` 方法：`enable_ctrl_c()`、`disable_ctrl_c()`、`ctrl_c_handler()`

handler 仅对 `CTRL_C_EVENT`（ctrl_type == 0）返回 TRUE（已处理），其他事件（如 Ctrl+Break、关闭窗口）返回 FALSE（交给默认处理）。

## 验证

1. 在 yazi 中输入 `:bash` 启动 bash
2. 按 Ctrl+C → 只显示 `^C` 和新提示符，无重复提示符
3. 输入 `exit` → yazi 正常恢复
4. 在 yazi 中按 Ctrl+C → 正常行为（handler 已移除）

## 变更记录

- 2026-04-25: 初始分析，排除 cmd.exe 假设
- 2026-04-25: C 验证程序确认 `SetConsoleMode` 高位 bits 问题
- 2026-04-26: 确认真正根因为 `CTRL_C_EVENT` 终止 yazi 进程，实现 `SetConsoleCtrlHandler` 修复
- 2026-04-26: 移除 `restore_console_mode()`，仅保留 Ctrl+C handler 生命周期管理
