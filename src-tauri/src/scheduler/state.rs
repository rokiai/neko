use std::{path::PathBuf, sync::Arc, time::Instant};

use parking_lot::Mutex;
use serde_json::{Value, json};

use crate::{
    config::{RuntimeSettings, StoredConfig, default_daily_stats, serialize, write},
    core::audio::AudioPlayer,
    monitors::idle::IdleProbe,
};

pub struct AppState {
    /// Shared so a detached save can serialize the latest state off-thread.
    pub config: Arc<Mutex<StoredConfig>>,
    pub config_path: PathBuf,
    /// Shared so a detached write can serialize against concurrent saves.
    save_lock: Arc<Mutex<()>>,
    /// Opens its output device on first playback, not at startup.
    pub audio: Mutex<AudioPlayer>,
    pub(crate) scheduler: Mutex<SchedulerState>,
    /// Separate from `scheduler` on purpose: the OS idle/lock probes must not
    /// run while the scheduler lock is held.
    pub(crate) idle: Mutex<IdleProbe>,
}

pub(crate) struct SchedulerState {
    pub break_time_ms: Option<i64>,
    pub having_break: bool,
    pub postponed_count: u32,
    pub idle_start_at_ms: Option<i64>,
    pub last_tick_at_ms: Option<i64>,
    pub last_completed_at_ms: Option<i64>,
    pub break_started_at: Option<Instant>,
    pub break_end_at_ms: Option<i64>,
    pub started_from_tray: bool,
    pub pending_break_due: bool,
    pub currently_idle: bool,
    pub was_in_working_hours: bool,
    pub preview_active: bool,
    pub preview_relaunch_pending: bool,
    pub active_break_settings: Option<Value>,
    pub break_window_labels: Vec<String>,
    pub break_window_ready_labels: Vec<String>,
    pub pending_work_seconds: i64,
    pub last_stats_flush_at_ms: i64,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            break_time_ms: None,
            having_break: false,
            postponed_count: 0,
            idle_start_at_ms: None,
            last_tick_at_ms: None,
            last_completed_at_ms: None,
            break_started_at: None,
            break_end_at_ms: None,
            started_from_tray: false,
            pending_break_due: false,
            currently_idle: false,
            was_in_working_hours: true,
            preview_active: false,
            preview_relaunch_pending: false,
            active_break_settings: None,
            break_window_labels: Vec::new(),
            break_window_ready_labels: Vec::new(),
            pending_work_seconds: 0,
            last_stats_flush_at_ms: 0,
        }
    }
}

impl AppState {
    pub fn new(config: StoredConfig, config_path: PathBuf) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            config_path,
            save_lock: Arc::new(Mutex::new(())),
            audio: Mutex::new(AudioPlayer::default()),
            scheduler: Mutex::new(SchedulerState::default()),
            idle: Mutex::new(IdleProbe::default()),
        }
    }

    /// Serializes under the config lock, then writes with the lock released:
    /// the write fsyncs, and holding the config lock across it would stall the
    /// tick and every settings IPC command.
    pub fn save_config(&self) -> anyhow::Result<()> {
        let data = serialize(&self.config.lock())?;
        let _save_lock = self.save_lock.lock();
        write(&self.config_path, &data)
    }

    /// Same, but the fsync happens on a blocking worker. Used by the 1 Hz tick
    /// so disk latency never stalls the scheduler's async task.
    ///
    /// Serialization happens inside the worker, after the save lock is taken:
    /// blocking tasks carry no ordering guarantee, so serializing at submit
    /// time would let an out-of-order write put stale stats back on disk.
    /// Serializing here means whichever write runs always persists the latest
    /// state.
    pub fn save_config_detached(&self) {
        let config = Arc::clone(&self.config);
        let save_lock = Arc::clone(&self.save_lock);
        let path = self.config_path.clone();
        tauri::async_runtime::spawn_blocking(move || {
            let _save_lock = save_lock.lock();
            let data = match serialize(&config.lock()) {
                Ok(data) => data,
                Err(error) => {
                    tracing::warn!("could not serialize configuration: {error}");
                    return;
                }
            };
            if let Err(error) = write(&path, &data) {
                tracing::warn!("could not persist configuration: {error}");
            }
        });
    }

    /// Projects the stored settings into scalars for the 1 Hz paths. Keeps the
    /// config lock for the projection only, and never clones the JSON.
    pub fn runtime_settings(&self) -> RuntimeSettings {
        RuntimeSettings::from_settings(&self.config.lock().settings)
    }

    pub fn record_break(&self, duration_ms: u64) -> bool {
        let mut config = self.config.lock();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        if !config.daily_stats.is_object() {
            config.daily_stats = default_daily_stats();
        }
        let current_day = config
            .daily_stats
            .get("dayKey")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if current_day.as_deref() != Some(today.as_str()) {
            config.daily_stats = json!({
                "dayKey": today,
                "workSeconds": 0,
                "restSeconds": 0,
                "completedBreaks": 0
            });
        }
        let rest_seconds = ((duration_ms + 500) / 1_000) as i64;
        let break_length = config
            .settings
            .get("breakLengthSeconds")
            .and_then(Value::as_i64)
            .unwrap_or(120);
        let complete_threshold_ms = break_length.max(1) as u64 * 1_000 / 2;
        let stats = config
            .daily_stats
            .as_object_mut()
            .expect("daily stats object");
        let rest = stats
            .get("restSeconds")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + rest_seconds;
        stats.insert("restSeconds".to_owned(), Value::from(rest));
        if duration_ms >= complete_threshold_ms {
            let completed = stats
                .get("completedBreaks")
                .and_then(Value::as_i64)
                .unwrap_or(0)
                + 1;
            stats.insert("completedBreaks".to_owned(), Value::from(completed));
            return true;
        }
        false
    }

    pub fn add_work_seconds(&self, seconds: i64) {
        if seconds <= 0 {
            return;
        }

        let mut config = self.config.lock();
        ensure_today_stats(&mut config.daily_stats);
        let stats = config
            .daily_stats
            .as_object_mut()
            .expect("daily stats object");
        let worked = stats
            .get("workSeconds")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + seconds;
        stats.insert("workSeconds".to_owned(), Value::from(worked));
    }

    pub fn complete_notification_break(&self, break_length_seconds: i64, now_ms: i64) {
        let mut config = self.config.lock();
        ensure_today_stats(&mut config.daily_stats);
        let stats = config
            .daily_stats
            .as_object_mut()
            .expect("daily stats object");
        let completed = stats
            .get("completedBreaks")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + 1;
        let rested = stats
            .get("restSeconds")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            + break_length_seconds.max(1);
        stats.insert("completedBreaks".to_owned(), Value::from(completed));
        stats.insert("restSeconds".to_owned(), Value::from(rested));

        self.scheduler.lock().last_completed_at_ms = Some(now_ms);
    }
}

fn ensure_today_stats(stats: &mut Value) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if !stats.is_object() || stats.get("dayKey").and_then(Value::as_str) != Some(today.as_str()) {
        *stats = json!({
            "dayKey": today,
            "workSeconds": 0,
            "restSeconds": 0,
            "completedBreaks": 0
        });
    }
}

/// Read-only view of today's stats: returns the stored object when its
/// `dayKey` is still today, otherwise a zeroed day (without persisting it).
pub(crate) fn today_stats_snapshot(stats: &Value) -> Value {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if stats.is_object() && stats.get("dayKey").and_then(Value::as_str) == Some(today.as_str()) {
        stats.clone()
    } else {
        json!({
            "dayKey": today,
            "workSeconds": 0,
            "restSeconds": 0,
            "completedBreaks": 0
        })
    }
}
