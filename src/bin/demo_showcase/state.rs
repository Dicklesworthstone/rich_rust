//! Demo state and simulation data for demo_showcase.

// State types prepared for dashboard/live scene implementations
#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceHealth {
    Ok,
    Warn,
    Err,
}

impl ServiceHealth {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Warn => "WARN",
            Self::Err => "ERR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub health: ServiceHealth,
    pub latency: Duration,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Running,
    Done,
    Failed,
}

impl StageStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    pub status: StageStatus,
    pub progress: f64,
    pub eta: Option<Duration>,
}

impl PipelineStage {
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub t: Duration,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DemoState {
    pub run_id: u64,
    pub seed: u64,
    started_at: Instant,
    pub headline: String,
    pub services: Vec<ServiceInfo>,
    pub pipeline: Vec<PipelineStage>,
    logs: VecDeque<LogLine>,
    log_capacity: usize,
}

impl DemoState {
    #[must_use]
    pub fn new(run_id: u64, seed: u64) -> Self {
        Self {
            run_id,
            seed,
            started_at: Instant::now(),
            headline: String::new(),
            services: Vec::new(),
            pipeline: Vec::new(),
            logs: VecDeque::new(),
            log_capacity: 200,
        }
    }

    #[must_use]
    pub fn with_log_capacity(run_id: u64, seed: u64, log_capacity: usize) -> Self {
        Self {
            log_capacity: log_capacity.max(1),
            ..Self::new(run_id, seed)
        }
    }

    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    pub fn push_log(&mut self, level: LogLevel, message: impl Into<String>) {
        let line = LogLine {
            t: self.elapsed(),
            level,
            message: message.into(),
        };

        self.logs.push_back(line);
        while self.logs.len() > self.log_capacity {
            self.logs.pop_front();
        }
    }

    #[must_use]
    pub fn logs_snapshot(&self) -> Vec<LogLine> {
        self.logs.iter().cloned().collect()
    }

    #[must_use]
    pub fn demo_seeded(run_id: u64, seed: u64) -> Self {
        let mut state = Self::with_log_capacity(run_id, seed, 200);

        state.headline = "Booting Nebula Deploy…".to_string();

        state.services = vec![
            ServiceInfo {
                name: "api".to_string(),
                health: ServiceHealth::Ok,
                latency: Duration::from_millis(12),
                version: "1.2.3".to_string(),
            },
            ServiceInfo {
                name: "worker".to_string(),
                health: ServiceHealth::Warn,
                latency: Duration::from_millis(48),
                version: "1.2.3".to_string(),
            },
            ServiceInfo {
                name: "db".to_string(),
                health: ServiceHealth::Err,
                latency: Duration::from_millis(0),
                version: "13.4".to_string(),
            },
        ];

        let service_logs: Vec<String> = state
            .services
            .iter()
            .map(|service| {
                format!(
                    "svc {}: {} ({}ms) v{}",
                    service.name,
                    service.health.as_str(),
                    service.latency.as_millis(),
                    service.version
                )
            })
            .collect();

        for line in service_logs {
            state.push_log(LogLevel::Info, line);
        }

        let mut stage_plan = PipelineStage {
            name: "plan".to_string(),
            status: StageStatus::Done,
            progress: 1.0,
            eta: None,
        };
        stage_plan.set_progress(1.0);

        let mut stage_deploy = PipelineStage {
            name: "deploy".to_string(),
            status: StageStatus::Running,
            progress: 0.0,
            eta: Some(Duration::from_secs(12)),
        };
        stage_deploy.set_progress(0.42);

        let stage_verify = PipelineStage {
            name: "verify".to_string(),
            status: StageStatus::Pending,
            progress: 0.0,
            eta: None,
        };

        let stage_cleanup = PipelineStage {
            name: "cleanup".to_string(),
            status: StageStatus::Failed,
            progress: 0.0,
            eta: None,
        };

        state.pipeline = vec![stage_plan, stage_deploy, stage_verify, stage_cleanup];

        let stage_logs: Vec<String> = state
            .pipeline
            .iter()
            .map(|stage| {
                let eta = stage
                    .eta
                    .map(|d| format!(" eta={}s", d.as_secs()))
                    .unwrap_or_default();
                format!("stage {} -> {}{}", stage.name, stage.status.as_str(), eta)
            })
            .collect();

        for line in stage_logs {
            state.push_log(LogLevel::Debug, line);
        }

        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            state.push_log(level, format!("{}: demo log line", level.as_str()));
        }

        state
    }
}

#[derive(Debug, Clone)]
pub struct DemoStateSnapshot {
    pub run_id: u64,
    pub seed: u64,
    pub elapsed: Duration,
    pub headline: String,
    pub services: Vec<ServiceInfo>,
    pub pipeline: Vec<PipelineStage>,
    pub logs: Vec<LogLine>,
}

impl From<&DemoState> for DemoStateSnapshot {
    fn from(value: &DemoState) -> Self {
        Self {
            run_id: value.run_id,
            seed: value.seed,
            elapsed: value.elapsed(),
            headline: value.headline.clone(),
            services: value.services.clone(),
            pipeline: value.pipeline.clone(),
            logs: value.logs_snapshot(),
        }
    }
}

#[derive(Clone)]
pub struct SharedDemoState {
    inner: Arc<Mutex<DemoState>>,
}

impl SharedDemoState {
    #[must_use]
    pub fn new(run_id: u64, seed: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DemoState::new(run_id, seed))),
        }
    }

    #[must_use]
    pub fn demo_seeded(run_id: u64, seed: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DemoState::demo_seeded(run_id, seed))),
        }
    }

    /// Update the demo state atomically.
    ///
    /// # Poison Recovery
    /// If a previous holder panicked while holding the lock, we recover by
    /// extracting the inner value. This is acceptable for demo_showcase because:
    /// - The state is ephemeral (demo session only)
    /// - A corrupted state just means visual glitches, not data loss
    /// - We prefer graceful degradation over cascading panics
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut DemoState),
    {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f(&mut guard);
    }

    /// Take a snapshot of the current demo state.
    ///
    /// # Poison Recovery
    /// Same rationale as `update` - we recover from poison rather than panic.
    #[must_use]
    pub fn snapshot(&self) -> DemoStateSnapshot {
        let guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        DemoStateSnapshot::from(&*guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_ring_buffer_caps() {
        let mut state = DemoState::with_log_capacity(1, 2, 2);
        state.push_log(LogLevel::Info, "one");
        state.push_log(LogLevel::Info, "two");
        state.push_log(LogLevel::Info, "three");

        let logs = state.logs_snapshot();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "two");
        assert_eq!(logs[1].message, "three");
    }

    #[test]
    fn shared_snapshot_is_clone_safe() {
        let shared = SharedDemoState::new(123, 456);
        shared.update(|state| {
            state.headline = "Starting".to_string();
            state.services.push(ServiceInfo {
                name: "api".to_string(),
                health: ServiceHealth::Ok,
                latency: Duration::from_millis(12),
                version: "1.2.3".to_string(),
            });
            state.push_log(LogLevel::Info, "hello");
        });

        let snap = shared.snapshot();
        assert_eq!(snap.run_id, 123);
        assert_eq!(snap.seed, 456);
        assert_eq!(snap.headline, "Starting");
        assert_eq!(snap.services.len(), 1);
        assert_eq!(snap.logs.len(), 1);
        assert_eq!(snap.logs[0].level, LogLevel::Info);
    }
}
