use std::{path::PathBuf, time::Instant};

use parking_lot::Mutex;
use serde_json::{Value, json};

use crate::{
    config::{StoredConfig, default_daily_stats, save},
    core::audio::AudioPlayer,
};

pub struct AppState {
    pub config: Mutex<StoredConfig>,
    pub config_path: PathBuf,
    save_lock: Mutex<()>,
    pub audio: Mutex<Option<AudioPlayer>>,
    pub(crate) scheduler: Mutex<SchedulerState>,
}

pub(crate) struct SchedulerState {
    pub break_time_ms: Option<i64>,
    pub having_break: bool,
    pub postponed_count: u32,
    pub idle_start_at_ms: Option<i64>,
    pub lock_start_at_ms: Option<i64>,
    pub last_tick_at_ms: Option<i64>,
    pub last_completed_at_ms: Option<i64>,
    pub break_started_at: Option<Instant>,
    pub break_end_at_ms: Option<i64>,
    pub started_from_tray: bool,
    pub pending_break_due: bool,
    pub currently_idle: bool,
    pub was_in_working_hours: bool,
    pub idle_detection_failures: u8,
    pub idle_detection_disabled: bool,
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
            lock_start_at_ms: None,
            last_tick_at_ms: None,
            last_completed_at_ms: None,
            break_started_at: None,
            break_end_at_ms: None,
            started_from_tray: false,
            pending_break_due: false,
            currently_idle: false,
            was_in_working_hours: true,
            idle_detection_failures: 0,
            idle_detection_disabled: false,
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
    pub fn new(config: StoredConfig, config_path: PathBuf, audio: Option<AudioPlayer>) -> Self {
        Self {
            config: Mutex::new(config),
            config_path,
            save_lock: Mutex::new(()),
            audio: Mutex::new(audio),
            scheduler: Mutex::new(SchedulerState::default()),
        }
    }

    pub fn save_config(&self) -> anyhow::Result<()> {
        let _save_lock = self.save_lock.lock();
        save(&self.config_path, &self.config.lock())
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
