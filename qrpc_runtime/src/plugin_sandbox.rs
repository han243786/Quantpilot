use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

/// v2.0.0 插件沙箱子进程隔离
///
/// 将插件调用隔离在独立的子进程中执行, 通过 stdin/stdout 传递数据。
/// - 超时硬限制: 超过 `timeout_ms` 后强制 kill 子进程
/// - 内存硬限制: 通过平台特定机制限制子进程虚拟内存用量
///   - Unix: 使用 `setrlimit(RLIMIT_AS)` 在 exec 前设置, 精确硬限制
///   - Windows: 当前版本不设置内存硬限制(需 Job Object API), 超时限制仍然生效
///
/// ## 已知安全限制
/// - **无网络隔离**: 子进程继承宿主全部网络能力。manifest中`allow_network: false`仅为声明式策略,
///   无OS级强制(net namespace/job object), v2.1.0计划实现。
/// - **无文件系统隔离**: 子进程可访问宿主机所有文件。v2.1.0计划引入chroot/命名空间隔离。
pub struct PluginSandbox {
    /// 单次执行最大耗时(毫秒), 超时则 kill
    pub timeout_ms: u64,
    /// 单次执行最大内存(MB), 超过则 OOM kill (Unix 精确, Windows 最佳努力)
    pub max_memory_mb: u64,
}

impl PluginSandbox {
    pub fn new(timeout_ms: u64, max_memory_mb: u64) -> Self {
        Self {
            timeout_ms,
            max_memory_mb,
        }
    }

    /// 在子进程中执行插件, 通过 stdin 发送 `input`, 从 stdout 读取输出。
    ///
    /// # 错误
    /// - 子进程无法启动
    /// - 超时 (`timeout_ms` 耗尽)
    /// - 子进程异常退出 (非 0 退出码)
    /// - I/O 错误
    pub fn execute(&self, plugin_path: &Path, input: &[u8]) -> Result<Vec<u8>, String> {
        let mut child = self.spawn_child(plugin_path)?;

        // 写入输入到子进程 stdin, 然后关闭管道以发送 EOF
        if let Some(ref mut stdin) = child.stdin {
            use std::io::Write;
            stdin
                .write_all(input)
                .map_err(|e| format!("写入插件 stdin 失败: {e}"))?;
        }
        // 主动 drop stdin 关闭管道, 通知子进程输入结束
        drop(child.stdin.take());

        // 带超时等待子进程退出
        let duration = Duration::from_millis(self.timeout_ms);
        let status = child
            .wait_timeout(duration)
            .map_err(|e| format!("等待子进程失败: {e}"))?;

        match status {
            Some(exit_status) => {
                if !exit_status.success() {
                    let code = exit_status.code().unwrap_or(-1);
                    return Err(format!("插件进程异常退出, 退出码: {code}"));
                }
            }
            None => {
                // 超时 — 强制终止子进程
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("插件执行超时 ({} ms)", self.timeout_ms));
            }
        }

        // 读取 stdout
        let mut output = Vec::new();
        if let Some(mut stdout) = child.stdout {
            stdout
                .read_to_end(&mut output)
                .map_err(|e| format!("读取插件 stdout 失败: {e}"))?;
        }

        Ok(output)
    }

    // ── 子进程启动 (平台相关) ──────────────────────────

    /// Unix: 在 `fork` 后 `exec` 前通过 `pre_exec` 设置 `RLIMIT_AS` 限制虚拟内存。
    /// 这是精确的硬限制: 子进程即使尝试 mmap 也会被内核拒绝。
    #[cfg(unix)]
    fn spawn_child(&self, plugin_path: &Path) -> Result<Child, String> {
        use std::os::unix::process::CommandExt;

        let mut cmd = Command::new(plugin_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let max_memory = self.max_memory_mb;
        unsafe {
            cmd.pre_exec(move || {
                if max_memory > 0 {
                    let bytes = (max_memory as u64).saturating_mul(1024 * 1024);
                    let limits = libc::rlimit {
                        rlim_cur: bytes,
                        rlim_max: bytes,
                    };
                    // v2.1.1: 检查 setrlimit 返回值，失败时记录错误
                    let rc = libc::setrlimit(libc::RLIMIT_AS, &limits);
                    if rc != 0 {
                        let err = std::io::Error::last_os_error();
                        eprintln!("[plugin_sandbox] setrlimit 失败: {} (errno={})", err, rc);
                    }
                }
                Ok(())
            });
        }

        cmd.spawn().map_err(|e| format!("无法启动插件子进程: {e}"))
    }

    /// Windows: 使用 `std::process::Command` 直接启动, 不设置内存硬限制。
    /// `max_memory_mb` 在 Windows 上为最佳努力, 超时限制仍然精确执行。
    #[cfg(windows)]
    fn spawn_child(&self, plugin_path: &Path) -> Result<Child, String> {
        let mut cmd = Command::new(plugin_path);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        cmd.spawn().map_err(|e| format!("无法启动插件子进程: {e}"))
    }
}

// ── 测试 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_construct_default() {
        let sandbox = PluginSandbox::new(5000, 128);
        assert_eq!(sandbox.timeout_ms, 5000);
        assert_eq!(sandbox.max_memory_mb, 128);
    }

    #[test]
    fn sandbox_timeout_zero_is_valid() {
        let sandbox = PluginSandbox::new(0, 128);
        assert_eq!(sandbox.timeout_ms, 0);
    }

    #[test]
    fn sandbox_memory_zero_disables_limit() {
        let sandbox = PluginSandbox::new(5000, 0);
        assert_eq!(sandbox.max_memory_mb, 0);
    }

    /// 验证基本子进程执行: 启动 /bin/cat, 通过 stdin 发送数据,
    /// 验证 stdout 收到相同数据。
    #[test]
    #[cfg(unix)]
    fn execute_cat_echoes_stdin() {
        let sandbox = PluginSandbox::new(5000, 64);
        let path = Path::new("/bin/cat");
        let input = b"hello plugin sandbox\n";

        let result = sandbox.execute(path, input);
        assert!(result.is_ok(), "execute 失败: {:?}", result.err());
        let output = result.unwrap();
        assert_eq!(
            String::from_utf8_lossy(&output).trim(),
            "hello plugin sandbox"
        );
    }

    /// 验证超时: 启动 sleep 10, 确认 100ms 后被 kill
    #[test]
    #[cfg(unix)]
    fn execute_timeout_kills_sleeping_child() {
        let sandbox = PluginSandbox::new(100, 64); // 100ms 超时
        let path = Path::new("/bin/sleep");
        let result = sandbox.execute(path, b"10"); // sleep 10 秒
        assert!(result.is_err(), "长时间休眠应该超时");
        let err = result.unwrap_err();
        assert!(err.contains("超时"), "错误消息应包含'超时', 实际: {err}");
    }

    /// 验证错误退出码传播
    #[test]
    #[cfg(unix)]
    fn execute_non_zero_exit_code_returns_error() {
        let sandbox = PluginSandbox::new(5000, 64);
        let path = Path::new("/bin/false");
        let result = sandbox.execute(path, b"");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("退出码"),
            "错误消息应包含'退出码', 实际: {err}"
        );
    }

    /// Windows 平台基本验证: 确认 cmd 能正常启动
    #[test]
    #[cfg(windows)]
    fn execute_cmd_echo_on_windows() {
        use std::process::Command;

        let output = Command::new("cmd.exe")
            .args(["/C", "echo hello plugin sandbox"])
            .output()
            .expect("无法启动 cmd.exe");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello plugin sandbox"));
    }
}
