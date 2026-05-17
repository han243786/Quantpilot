// ── 数据源断路器 (v2.1.0) ──
// 标准三态断路器: Closed → Open → HalfOpen → Closed
// 连续失败达阈值→熔断(Open), 冷却后进入半开(HalfOpen)探测, 探测成功→恢复(Closed)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure_ms: u64,
    open_until_ms: u64,
    threshold: u32,
    cooldown_ms: u64,
    half_open_limit: u32,
    half_open_successes: u32,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, cooldown_ms: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_ms: 0,
            open_until_ms: 0,
            threshold: threshold.max(1),
            cooldown_ms: cooldown_ms.max(1000),
            half_open_limit: 1,
            half_open_successes: 0,
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// 是否拒绝请求（断路器打开且未到冷却时间，或半开已用完探测配额）
    pub fn is_open(&self) -> bool {
        match self.state {
            CircuitState::Closed => false,
            CircuitState::Open => true,
            CircuitState::HalfOpen => {
                self.half_open_successes >= self.half_open_limit
            }
        }
    }

    /// 请求成功
    pub fn on_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.half_open_successes += 1;
                if self.half_open_successes >= self.half_open_limit {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.half_open_successes = 0;
                }
            }
            CircuitState::Open => {
                // Open 状态下不应有请求通过, 但防御性处理
            }
        }
    }

    /// 请求失败, now_ms 用于计算冷却窗口
    pub fn on_failure(&mut self, now_ms: u64) {
        self.last_failure_ms = now_ms;

        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.threshold {
                    self.state = CircuitState::Open;
                    self.open_until_ms = now_ms.saturating_add(self.cooldown_ms);
                    self.failure_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                // 探测失败, 重新熔断
                self.state = CircuitState::Open;
                self.open_until_ms = now_ms.saturating_add(self.cooldown_ms);
                self.half_open_successes = 0;
            }
            CircuitState::Open => {
                // 已在 Open 状态, 延长冷却
                self.open_until_ms = now_ms.saturating_add(self.cooldown_ms);
            }
        }
    }

    /// 在每次请求前调用: 如果冷却时间已到, 从 Open → HalfOpen
    pub fn try_half_open(&mut self, now_ms: u64) {
        if self.state == CircuitState::Open && now_ms >= self.open_until_ms {
            self.state = CircuitState::HalfOpen;
            self.half_open_successes = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed() {
        let cb = CircuitBreaker::new(5, 60_000);
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(!cb.is_open());
    }

    #[test]
    fn opens_after_threshold_failures() {
        let mut cb = CircuitBreaker::new(3, 60_000);
        cb.on_failure(1000);
        cb.on_failure(2000);
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.on_failure(3000);
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(cb.is_open());
    }

    #[test]
    fn transitions_to_half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new(2, 1000);
        cb.on_failure(0); // fail 1
        cb.on_failure(0); // fail 2 → Open, open_until = 1000
        assert_eq!(cb.state(), CircuitState::Open);

        cb.try_half_open(500);
        assert_eq!(cb.state(), CircuitState::Open); // not yet

        cb.try_half_open(1500);
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(!cb.is_open()); // HalfOpen allows one probe
    }

    #[test]
    fn recovers_after_probe_success() {
        let mut cb = CircuitBreaker::new(2, 1000);
        cb.on_failure(0);
        cb.on_failure(0);

        cb.try_half_open(1500);
        cb.on_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn reopens_after_probe_failure() {
        let mut cb = CircuitBreaker::new(2, 1000);
        cb.on_failure(0);
        cb.on_failure(0);

        cb.try_half_open(1500);
        cb.on_failure(1600); // probe fails
        assert_eq!(cb.state(), CircuitState::Open); // back to Open
    }

    #[test]
    fn success_resets_failure_count_in_closed() {
        let mut cb = CircuitBreaker::new(5, 60_000);
        cb.on_failure(1000);
        cb.on_failure(2000);
        cb.on_success();
        cb.on_failure(3000);
        cb.on_failure(4000);
        assert_eq!(cb.state(), CircuitState::Closed); // count reset by success
    }

    #[test]
    fn threshold_minimum_one() {
        let cb = CircuitBreaker::new(0, 60_000);
        assert_eq!(cb.threshold, 1);
    }

    #[test]
    fn cooldown_minimum_one_second() {
        let cb = CircuitBreaker::new(3, 500);
        assert_eq!(cb.cooldown_ms, 1000);
    }
}
