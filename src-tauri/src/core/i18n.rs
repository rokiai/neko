//! Minimal runtime locale table for tray menus and notifications.
//!
//! Values mirror `src/shared/i18n/{en,zh,ja}.ts`; keep both sides in sync when
//! editing copy. Only strings rendered by Rust (tray, system notifications)
//! live here — everything else stays in the frontend i18n bundles.

use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    En,
    Zh,
    Ja,
}

/// Resolves the effective locale from the persisted `settings.locale` value
/// (`system` / `en` / `zh` / `ja`), falling back to the OS locale.
pub fn resolve(settings: &Value) -> Locale {
    match settings.get("locale").and_then(Value::as_str) {
        Some("en") => Locale::En,
        Some("zh") => Locale::Zh,
        Some("ja") => Locale::Ja,
        _ => system_locale(),
    }
}

fn system_locale() -> Locale {
    let tag = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    if tag.starts_with("zh") {
        Locale::Zh
    } else if tag.starts_with("ja") {
        Locale::Ja
    } else {
        Locale::En
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Text {
    TrayDisabled,
    TrayOutsideHours,
    TrayIdle,
    TrayOnBreak,
    TrayScheduling,
    TrayStartNow,
    TrayDisable,
    TrayDisable30m,
    TrayDisable1h,
    TrayDisable2h,
    TrayDisable4h,
    TrayDisableEod,
    TrayDisableIndefinite,
    TrayEnable,
    TraySettings,
    TrayQuit,
    NotifyIdleTitle,
    BreakDefaultTitle,
    BreakDefaultMessage,
}

pub fn text(locale: Locale, key: Text) -> &'static str {
    use Locale::{En, Ja, Zh};
    use Text::*;
    match (locale, key) {
        (En, TrayDisabled) => "Breaks disabled",
        (Zh, TrayDisabled) => "休息提醒已关闭",
        (Ja, TrayDisabled) => "休憩リマインダーはオフです",
        (En, TrayOutsideHours) => "Outside working hours",
        (Zh, TrayOutsideHours) => "非工作时间",
        (Ja, TrayOutsideHours) => "勤務時間外",
        (En, TrayIdle) => "Idle · timer paused",
        (Zh, TrayIdle) => "空闲 · 计时已暂停",
        (Ja, TrayIdle) => "アイドル · タイマー停止中",
        (En, TrayOnBreak) => "On a break",
        (Zh, TrayOnBreak) => "休息中",
        (Ja, TrayOnBreak) => "休憩中",
        (En, TrayScheduling) => "Scheduling…",
        (Zh, TrayScheduling) => "调度中…",
        (Ja, TrayScheduling) => "スケジュール中…",
        (En, TrayStartNow) => "Start break now",
        (Zh, TrayStartNow) => "立即开始休息",
        (Ja, TrayStartNow) => "今すぐ休憩",
        (En, TrayDisable) => "Disable",
        (Zh, TrayDisable) => "暂时关闭",
        (Ja, TrayDisable) => "一時停止",
        (En, TrayDisable30m) => "For 30 minutes",
        (Zh, TrayDisable30m) => "30 分钟",
        (Ja, TrayDisable30m) => "30分",
        (En, TrayDisable1h) => "For 1 hour",
        (Zh, TrayDisable1h) => "1 小时",
        (Ja, TrayDisable1h) => "1時間",
        (En, TrayDisable2h) => "For 2 hours",
        (Zh, TrayDisable2h) => "2 小时",
        (Ja, TrayDisable2h) => "2時間",
        (En, TrayDisable4h) => "For 4 hours",
        (Zh, TrayDisable4h) => "4 小时",
        (Ja, TrayDisable4h) => "4時間",
        (En, TrayDisableEod) => "Until end of day",
        (Zh, TrayDisableEod) => "直到今天结束",
        (Ja, TrayDisableEod) => "今日の終わりまで",
        (En, TrayDisableIndefinite) => "Indefinitely",
        (Zh, TrayDisableIndefinite) => "一直关闭",
        (Ja, TrayDisableIndefinite) => "無期限",
        (En, TrayEnable) => "Enable breaks",
        (Zh, TrayEnable) => "启用休息提醒",
        (Ja, TrayEnable) => "休憩リマインダーを有効化",
        (En, TraySettings) => "Settings…",
        (Zh, TraySettings) => "设置…",
        (Ja, TraySettings) => "設定…",
        (En, TrayQuit) => "Quit",
        (Zh, TrayQuit) => "退出",
        (Ja, TrayQuit) => "終了",
        (En, NotifyIdleTitle) => "Break automatically detected",
        (Zh, NotifyIdleTitle) => "已自动识别休息",
        (Ja, NotifyIdleTitle) => "休憩を自動検出",
        (En, BreakDefaultTitle) => "Time for a break.",
        (Zh, BreakDefaultTitle) => "该休息一下了",
        (Ja, BreakDefaultTitle) => "休憩の時間です。",
        (En, BreakDefaultMessage) => {
            "Give your eyes a rest.\nHave a drink of water.\nTake a moment to unwind."
        }
        (Zh, BreakDefaultMessage) => "让双眼休息一下\n喝点水，补充能量\n放松片刻",
        (Ja, BreakDefaultMessage) => {
            "目を休めましょう。\n水を飲んでエネルギーを補給。\n少しリラックス。"
        }
    }
}

pub fn tray_next_in(locale: Locale, time: &str) -> String {
    match locale {
        Locale::En => format!("Next break in {time}"),
        Locale::Zh => format!("{time} 后休息"),
        Locale::Ja => format!("次の休憩まで {time}"),
    }
}

pub fn tray_disabled_left(locale: Locale, time: &str) -> String {
    match locale {
        Locale::En => format!("Disabled · {time} left"),
        Locale::Zh => format!("已关闭 · 剩余 {time}"),
        Locale::Ja => format!("オフ · 残り {time}"),
    }
}

pub fn tray_about(locale: Locale, name: &str) -> String {
    match locale {
        Locale::En => format!("About {name}"),
        Locale::Zh => format!("关于 {name}"),
        Locale::Ja => format!("{name} について"),
    }
}

pub fn notify_idle_body(locale: Locale, minutes: i64) -> String {
    match locale {
        Locale::En => format!("Away for about {minutes} minute(s). Timer reset."),
        Locale::Zh => format!("离开约 {minutes} 分钟，计时已重置。"),
        Locale::Ja => format!("約 {minutes} 分離席していました。タイマーをリセットしました。"),
    }
}
