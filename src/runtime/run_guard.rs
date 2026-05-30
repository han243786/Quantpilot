use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct RunInProgressGuard<'a>(pub(super) &'a AtomicBool);

impl Drop for RunInProgressGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_run_guard_resets_on_drop() {
        let run_in_progress = AtomicBool::new(true);

        {
            let _guard = RunInProgressGuard(&run_in_progress);
            assert!(run_in_progress.load(Ordering::Acquire));
        }

        assert!(!run_in_progress.load(Ordering::Acquire));
    }
}
