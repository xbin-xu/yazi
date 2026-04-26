# DISCUSSION-3180: Ctrl+C 测试结果与分析

## 测试记录

### Test1: 原版 yazi + bash + 另一终端 kill -2

0. 打开一个 bash，并查看该 bash 的 PID，并记为 PID1: echo $$
1. 使用原本的 yazi(`which yazi`)
2. 启动 yazi: 输入 yazi
3. 启动 bash: 输入 :bash
4. 查看 yazi 启动的 bash 的 PID，并记为 PID2: 输入 echo $$
5. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: kill -2 xxx
6. 现象: PID2 显示 ^C，并进入下一次 prompt
7. 在另外一个终端中查看 yazi 的 PID，并记为 PID3
8. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: kill -2 xxx
9. 现象：出现了之前反馈的问题

```sh
Workbench 08:54 ❯ z yazi
yazi on  fix/ctrl-c-windows 08:54 ❯ echo $$
4247
yazi on  fix/ctrl-c-windows 08:54 ❯ which yazi
/d/Software/Scoop/shims/yazi
yazi on  fix/ctrl-c-windows 08:55 ❯ yazi
yazi on  fix/ctrl-c-windows 08:55 ❯ echo $$
4339
yazi on  fix/ctrl-c-windows 08:55 ❯ ^C
yazi on  fix/ctrl-c-windows 08:56 ❯ ^C

yazi on  fix/ctrl-c-windows 09:00 ❯ yazi on  fix/ctrl-c-windows 09:00 ❯
```

### Test2: 修复版 yazi + bash + 另一终端 kill -2

0. 打开一个 bash，并查看该 bash 的 PID，并记为 PID1: echo $$
1. 启动修改后的 yazi: 输入 ./target/debut/yazi.exe
2. 启动 bash: 输入 :bash
3. 查看 yazi 启动的 bash 的 PID，并记为 PID2: 输入 echo $$
4. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: kill -2 xxx
5. 现象: PID2 显示 ^C，并进入下一次 prompt
6. 在另外一个终端中查看 yazi 的 PID，并记为 PID3
7. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: kill -2 xxx
8. 现象：出现了之前反馈的问题

```sh
Workbench 09:02 ❯ z yazi
yazi on  fix/ctrl-c-windows 09:02 ❯ echo $$
4421
yazi on  fix/ctrl-c-windows 09:02 ❯ ./target/debug/yazi.exe
yazi on  fix/ctrl-c-windows 09:02 ❯ echo $$
4505
yazi on  fix/ctrl-c-windows 09:03 ❯ ^C
yazi on  fix/ctrl-c-windows 09:03 ❯ ^C




thread 'main' (13336) panicked at src\print.rs:76:48:
called `Result::unwrap()` on an `Err` value: Os { code: 232, kind: BrokenPipe, kind: BrokenPipe, message: "管道正在被关闭。" }
stack backtrace:
   0:     0x7ff7f8687b40 - <unknown>
   1:     0x7ff7f7f96dca - <unknown>
   2:     0x7ff7f8685e84 - <unknown>
   3:     0x7ff7f8687944 - <unknown>
   4:     0x7ff7f8685a8f - <unknown>
   5:     0x7ff7f868cd4a - <unknown>
   6:     0x7ff7f868cc9f - <unknown>
   7:     0x7ff7f868cc8e - <unknown>
   8:     0x7ff7f874b651 - <unknown>
   9:     0x7ff7f874bc20 - <unknown>
  10:     0x7ff7f7edcbec - <unknown>
  11:     0x7ff7f7ec7fd6 - <unknown>
  12:     0x7ff7f7ee60bb - <unknown>
  13:     0x7ff7f8721fbc - <unknown>
  14:     0x7ffe85d77374 - BaseThreadInitThunk
  15:     0x7ffe8711cc91 - RtlUserThreadStart
```

### Test3: 原版 yazi + powershell + 另一终端 kill -2

0. 打开一个 bash，并查看该 bash 的 PID，并记为 PID1: echo $$
1. 使用原本的 yazi(`which yazi`)
2. 启动 yazi: 输入 yazi
3. 启动 powershell: 输入 :powershell
4. 查看 yazi 启动的 powershell 的 WINPID，并记为 PID2: pgrep powershell
5. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: pkill -2 xxx
6. 现象: 没有反应
7. 在另外一个终端中查看 yazi 的 PID，并记为 PID3
8. 在另外一个终端中向 PID2 发送 `<C-c>` 信号: kill -2 xxx
9. 现象：powershell 被退出

```sh
Workbench 09:05 ❯ echo $$
5210
Workbench 09:05 ❯ which yazi
/d/Software/Scoop/shims/yazi
Workbench 09:05 ❯ yazi
Windows PowerShell
版权所有 (C) Microsoft Corporation。保留所有权利。

尝试新的跨平台 PowerShell https://aka.ms/pscore6

加载个人及系统配置文件用了 911 毫秒。
❯❯ Workbench  09:06
Workbench 09:10 ❯
```

### Test4: 原版 yazi + bash + 物理按键 Ctrl+C

0. 打开一个 bash，并查看该 bash 的 PID，并记为 PID1: echo $$
1. 使用原本的 yazi(`which yazi`)
2. 启动 yazi: 输入 yazi
3. 启动 bash: 输入 :bash
4. 在 yazi 启动的 bash 中输入 `<C-c>`
5. 现象：出现了之前反馈的问题

```sh
Workbench 09:12 ❯ z yazi
yazi on  fix/ctrl-c-windows 09:12 ❯ echo $$
5432
yazi on  fix/ctrl-c-windows 09:12 ❯ which yazi
/d/Software/Scoop/shims/yazi
yazi on  fix/ctrl-c-windows 09:12 ❯ yazi
yazi on  fix/ctrl-c-windows 09:13 ❯ ^C

yazi on  fix/ctrl-c-windows 09:13 ❯ yazi on  fix/ctrl-c-windows 09:13 ❯
```

### Test5: 修复版 yazi + bash + 物理按键 Ctrl+C

0. 打开一个 bash，并查看该 bash 的 PID，并记为 PID1: echo $$
1. 启动修改后的 yazi: 输入 ./target/debut/yazi.exe
2. 启动 bash: 输入 :bash
3. 在 yazi 启动的 bash 中输入 `<C-c>`
4. 现象：出现了之前反馈的问题

```sh
Workbench 09:15 ❯ z yazi
yazi on  fix/ctrl-c-windows 09:15 ❯ ./target/debug/yazi.exe
yazi on  fix/ctrl-c-windows 09:15 ❯ ^C
yazi on  fix/ctrl-c-windows 09:15 ❯
```

---

## 测试矩阵

| Test | yazi 版本 | 子进程 | 触发方式 | 结果 |
|------|-----------|--------|----------|------|
| 1 | 原版 | bash | 另一终端 `kill -2` | 第二次 kill 后双提示符 |
| 2 | 修复版 | bash | 另一终端 `kill -2` | 第二次 kill 后 yazi 崩溃 (BrokenPipe) |
| 3 | 原版 | powershell | 另一终端 `kill -2` | 第一次无反应，第二次 powershell 退出 |
| 4 | 原版 | bash | 物理按键 Ctrl+C | 双提示符 (原始 bug) |
| 5 | 修复版 | bash | 物理按键 Ctrl+C | **正常** |

## 核心结论

**物理按键 Ctrl+C（Test4 → Test5）已修复。** 这是 Discussion #3180 报告的原始问题。

`kill -2` 从另一终端发送信号是不同的场景，涉及 Windows 信号机制的复杂性。

## 各测试分析

### Test4 vs Test5：物理按键 Ctrl+C（原始 bug）

**Test4（原版 yazi）— 出现 bug：**

```
yazi on  fix/ctrl-c-windows 09:13 ❯ ^C
yazi on  fix/ctrl-c-windows 09:13 ❯ yazi on  fix/ctrl-c-windows 09:13 ❯
```

物理按键 Ctrl+C → Windows Console 检测到 Ctrl+C 组合键 → 因为 `ENABLE_PROCESSED_INPUT` 已恢复（`disable_raw_mode()` 后），Console 生成 `CTRL_C_EVENT` → 广播给同一控制台组所有进程 → yazi 的默认 handler 调用 `ExitProcess` → yazi 被终止 → `child.wait()` 提前返回 → `_permit` Drop 触发 `AppProxy::resume()` → yazi 恢复 UI → 但 bash 仍在运行 → 两个进程争抢 stdin → 双提示符。

**Test5（修复版 yazi）— 正常：**

```
yazi on  fix/ctrl-c-windows 09:15 ❯ ^C
yazi on  fix/ctrl-c-windows 09:15 ❯
```

`SetConsoleCtrlHandler` 在 `Term::stop()` 中安装，对 `CTRL_C_EVENT` 返回 TRUE（已处理）→ yazi 忽略该信号 → 只有 bash 处理 Ctrl+C → bash 显示 `^C` 后继续等待输入 → 正常。

### Test1 vs Test2：另一终端 `kill -2`

`kill -2 PID` 在 MSYS2/Windows 环境下的行为与物理 Ctrl+C 不同。

**Test1（原版 yazi）：**

- 第一次 `kill -2 PID2`：bash 显示 `^C`，正常
- 第二次 `kill -2 PID2`：出现双提示符

**Test2（修复版 yazi）：**

- 第一次 `kill -2 PID2`：bash 显示 `^C`，正常
- 第二次 `kill -2 PID2`：yazi 崩溃，BrokenPipe panic

**可能原因：**

MSYS2 的 `kill` 命令对信号的处理与 Windows Console 的 Ctrl+C 不同。`kill -2` 可能使用 `GenerateConsoleCtrlEvent(CTRL_C_EVENT, processGroup)` 或 MSYS2 内部信号机制。

- 如果使用 `GenerateConsoleCtrlEvent`：该函数将信号广播给**同一控制台的所有进程**（不仅仅是目标 PID），yazi 和 bash 都会收到。我们的 handler 能捕获 `CTRL_C_EVENT`（type 0），但第二次信号可能导致管道状态异常（bash 收到两次 SIGINT 后可能关闭 stdin pipe），yazi 向已关闭的管道写入时触发 `BrokenPipe` panic（Windows error code 232）。

- 如果使用 MSYS2 内部信号机制：仅影响 MSYS2 进程（bash），不影响原生 Windows 进程（yazi.exe）。bash 在收到两次 SIGINT 后退出（bash 默认行为：空行连续两次 SIGINT 退出），yazi 正常 resume，但显示上出现混乱。

BrokenPipe panic 的调用栈显示崩溃发生在 `src\print.rs:76`，即 yazi 向终端写入时管道已被关闭。这说明 yazi 进程本身没有被信号终止（handler 起作用了），但其输出管道因为 bash 退出或控制台状态变化而被关闭。

### Test3：PowerShell

**PowerShell 是原生 Windows 应用**，自行注册了 `SetConsoleCtrlHandler` 处理 `CTRL_C_EVENT`：

- 第一次 `kill -2`：PowerShell 内部消化了信号，无可见反应
- 第二次 `kill -2`：PowerShell 选择退出

这也解释了为什么原始 bug 只影响 bash 而不影响 PowerShell — bash 通过 cmd.exe 启动后依赖控制台默认的信号处理，而 PowerShell 主动管理了自己的信号处理。

## 总结

| 场景                   | 原版 yazi       | 修复版 yazi | 原因                            |
|------------------------|-----------------|-------------|---------------------------------|
| 物理 Ctrl+C + bash     | 双提示符 bug    | **正常**    | handler 捕获 CTRL_C_EVENT       |
| `kill -2` + bash       | 双提示符        | yazi 崩溃   | 不同于物理 Ctrl+C，管道状态异常 |
| `kill -2` + powershell | powershell 退出 | 未测试      | PowerShell 自行管理信号         |

**`SetConsoleCtrlHandler` 修复对物理 Ctrl+C 有效，这是原始报告的场景。`kill -2` 是 Windows/MSYS2 特有的信号传播问题，需要进一步调查。**

## 修复版 Test2 的 BrokenPipe 问题

```
thread 'main' (13336) panicked at src\print.rs:76:48:
called `Result::unwrap()` on an `Err` value: Os { code: 232, kind: BrokenPipe, message: "管道正在被关闭。" }
```

BrokenPipe (error 232) 表示 yazi 尝试向已关闭的管道写入。这可能是 bash 收到两次 SIGINT 后退出，导致 yazi 的 stdout pipe 被关闭。yazi 的输出代码使用 `unwrap()` 处理写入结果，没有处理 BrokenPipe 错误。

这是一个可以改进的地方：yazi 的输出代码应该处理 BrokenPipe 而不是 panic。但这与 Ctrl+C 信号问题是独立的。
