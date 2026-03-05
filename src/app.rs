// SPDX-License-Identifier: MPL-2.0

use crate::core::pomodoro::Pomodoro;
use crate::core::timer::{TimerState, TimerType};
use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::{Alignment, Length, Limits, Subscription, time, window::Id};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget::{self};
use std::time::Duration;

/// Dva pogleda (Main i Settings), kao u starom template-u.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PopupView {
    Main,
    Settings,
}

impl Default for PopupView {
    fn default() -> Self {
        PopupView::Main
    }
}

/// Snapshot stanja koji vraća `update_and_return_state`.
#[derive(Debug, Clone)]
struct PomodoroTickState {
    remaining: u64,
    timer_state: TimerState,
    timer_type: TimerType,
    started: bool,
}

impl PomodoroTickState {
    fn from_tuple(value: (u64, TimerState, TimerType, bool)) -> Self {
        Self {
            remaining: value.0,
            timer_state: value.1,
            timer_type: value.2,
            started: value.3,
        }
    }
}

impl Default for PomodoroTickState {
    fn default() -> Self {
        Self {
            remaining: 0,
            timer_state: TimerState::Created,
            timer_type: TimerType::Work,
            started: false,
        }
    }
}

/// Centralno stanje tajmera u jednom mestu.
struct PomodoroState {
    pomodoro: Pomodoro,
    last_tick_state: Option<PomodoroTickState>,
    settings_changed: bool,
}

impl PomodoroState {
    fn new(config: &Config) -> Self {
        let mut this = Self {
            pomodoro: Self::new_pomodoro(config),
            last_tick_state: None,
            settings_changed: false,
        };

        this.refresh_state();
        this
    }

    fn new_pomodoro(config: &Config) -> Pomodoro {
        Pomodoro::new(
            config.long_break_interval,
            config.work_time * 60,
            config.short_break_time * 60,
            config.long_break_time * 60,
            config.auto_start_work,
            config.auto_start_break,
        )
    }

    fn mark_settings_changed(&mut self) {
        self.settings_changed = true;
    }

    fn apply_settings_if_needed(&mut self, config: &Config) {
        if self.settings_changed {
            self.pomodoro = Self::new_pomodoro(config);
            self.settings_changed = false;
            self.refresh_state();
        }
    }

    fn refresh_state(&mut self) {
        self.last_tick_state = self
            .pomodoro
            .update_and_return_state()
            .map(PomodoroTickState::from_tuple);
    }

    fn is_started(&self) -> bool {
        self.last_tick_state
            .as_ref()
            .map(|s| s.started)
            .unwrap_or(false)
    }
}

/// Aplet model — stanje aplikacije.
pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    config: Config,
    current_view: PopupView,
    pomodoro_state: PomodoroState,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    PomodoroTick,
    StartPomodoro,
    PausePomodoro,
    ForwardPomodoro,
    RestartPomodoro,
    UpdateConfig(Config),
    OpenSettingsView,
    BackToMainView,
    Settings(SettingsMessage),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ResetToDefault,
    SetLongBreakInterval(u64),
    SetWorkTime(u64),
    SetShortBreakTime(u64),
    SetLongBreakTime(u64),
    SetAutoStartWork(bool),
    SetAutoStartBreak(bool),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "com.github.petar030.cosmic-pomodoro";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let config = cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
            .map(|context| match Config::get_entry(&context) {
                Ok(config) => config,
                Err((_errors, config)) => config,
            })
            .unwrap_or_default();

        let pomodoro_state = PomodoroState::new(&config);

        (
            AppModel {
                core,
                popup: None,
                config,
                current_view: PopupView::Main,
                pomodoro_state,
            },
            Task::none(),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: Id) -> Element<'_, Self::Message> {
        match self.current_view {
            PopupView::Main => view_main(
                &self.core,
                self.current_view,
                id,
                &self.config,
                &self.pomodoro_state,
            ),
            PopupView::Settings => view_settings(&self.core, self.current_view, id, &self.config),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            time::every(Duration::from_millis(250)).map(|_| Message::PomodoroTick),
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::PomodoroTick => {
                self.pomodoro_state.refresh_state();
            }
            Message::StartPomodoro => {
                self.pomodoro_state.pomodoro.start();
                self.pomodoro_state.refresh_state();
            }
            Message::PausePomodoro => {
                self.pomodoro_state.pomodoro.pause();
                self.pomodoro_state.refresh_state();
            }
            Message::ForwardPomodoro => {
                self.pomodoro_state.pomodoro.forward();
                self.pomodoro_state.refresh_state();
            }
            Message::RestartPomodoro => {
                self.pomodoro_state = PomodoroState::new(&self.config);
            }
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            Message::OpenSettingsView => {
                self.current_view = PopupView::Settings;
            }
            Message::BackToMainView => {
                self.pomodoro_state.apply_settings_if_needed(&self.config);
                self.current_view = PopupView::Main;
            }
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::Settings(msg) => match msg {
                SettingsMessage::ResetToDefault => {
                    self.config = Config::default();
                    self.pomodoro_state.mark_settings_changed();
                    if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                        let _ = self.config.write_entry(&ctx);
                    }
                }
                SettingsMessage::SetLongBreakInterval(v) => {
                    if self.config.long_break_interval != v {
                        self.config.long_break_interval = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
                SettingsMessage::SetWorkTime(v) => {
                    if self.config.work_time != v {
                        self.config.work_time = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
                SettingsMessage::SetShortBreakTime(v) => {
                    if self.config.short_break_time != v {
                        self.config.short_break_time = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
                SettingsMessage::SetLongBreakTime(v) => {
                    if self.config.long_break_time != v {
                        self.config.long_break_time = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
                SettingsMessage::SetAutoStartWork(v) => {
                    if self.config.auto_start_work != v {
                        self.config.auto_start_work = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
                SettingsMessage::SetAutoStartBreak(v) => {
                    if self.config.auto_start_break != v {
                        self.config.auto_start_break = v;
                        self.pomodoro_state.mark_settings_changed();
                        if let Ok(ctx) = cosmic_config::Config::new(Self::APP_ID, Config::VERSION) {
                            let _ = self.config.write_entry(&ctx);
                        }
                    }
                }
            },
        }

        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

fn view_main<'a>(
    core: &'a cosmic::Core,
    _current_view: PopupView,
    _id: Id,
    _config: &'a Config,
    pomodoro_state: &'a PomodoroState,
) -> Element<'a, Message> {
    let mut header = widget::row().padding(0).spacing(0);
    if !pomodoro_state.is_started() {
        header = header.push(
            core.applet
                .icon_button("preferences-system-symbolic")
                .on_press(Message::OpenSettingsView),
        );
    }

    let controls = widget::row()
        .padding(0)
        .spacing(10)
        .push(core.applet.text_button("Start", Message::StartPomodoro))
        .push(core.applet.text_button("Pause", Message::PausePomodoro))
        .push(core.applet.text_button("Forward", Message::ForwardPomodoro))
        .push(core.applet.text_button("Restart", Message::RestartPomodoro));

    let state_text = if let Some(s) = &pomodoro_state.last_tick_state {
        format!(
            "Pomodoro state:\nremaining={}s\nstate={:?}\ntype={:?}\nstarted={}",
            s.remaining, s.timer_state, s.timer_type, s.started
        )
    } else {
        "Pomodoro state: (no state available)".to_string()
    };

    let content_list = widget::column()
        .padding(0)
        .spacing(10)
        .push(header)
        .push(controls)
        .push(widget::text(state_text).size(15));

    core.applet
        .popup_container(content_list.padding(10).width(Length::Fixed(320.0)))
        .into()
}

fn view_settings<'a>(
    core: &'a cosmic::Core,
    _current_view: PopupView,
    _id: Id,
    config: &'a Config,
) -> Element<'a, Message> {
    let header = widget::row().padding(2).spacing(0).push(
        core.applet
            .icon_button("go-previous-symbolic")
            .on_press(Message::BackToMainView),
    );

    let work_time_row = widget::row()
        .padding(0)
        .spacing(0)
        .push(widget::spin_button(
            format!("{}", config.work_time),
            config.work_time,
            5,
            1,
            180,
            |v| Message::Settings(SettingsMessage::SetWorkTime(v as u64)),
        ));
    let short_break_time_row = widget::row()
        .padding(0)
        .spacing(0)
        .push(widget::spin_button(
            format!("{}", config.short_break_time),
            config.short_break_time,
            1,
            1,
            60,
            |v| Message::Settings(SettingsMessage::SetShortBreakTime(v as u64)),
        ));
    let long_break_time_row = widget::row()
        .padding(0)
        .spacing(0)
        .push(widget::spin_button(
            format!("{}", config.long_break_time),
            config.long_break_time,
            1,
            1,
            180,
            |v| Message::Settings(SettingsMessage::SetLongBreakTime(v as u64)),
        ));
    let long_break_interval_row = widget::row()
        .padding(0)
        .spacing(0)
        .push(widget::spin_button(
            format!("{}", config.long_break_interval),
            config.long_break_interval,
            1,
            1,
            10,
            |v| Message::Settings(SettingsMessage::SetLongBreakInterval(v as u64)),
        ));

    let content_list = widget::column()
        .width(Length::Fill)
        .padding(0)
        .spacing(10)
        .push(header)
        .push(
            widget::column()
                .width(Length::Fill)
                .align_x(Alignment::Center)
                .spacing(16)
                .push(widget::text("Work time (minutes)").size(15))
                .push(work_time_row)
                .push(widget::text("Short break time (minutes)").size(15))
                .push(short_break_time_row)
                .push(widget::text("Long break time (minutes)").size(15))
                .push(long_break_time_row)
                .push(widget::text("Long break interval").size(15))
                .push(long_break_interval_row)
                .push(
                    widget::column()
                        .spacing(12)
                        .push(
                            widget::toggler(config.auto_start_work)
                                .spacing(10)
                                .label("Auto start work timer")
                                .size(15)
                                .on_toggle(|v| {
                                    Message::Settings(SettingsMessage::SetAutoStartWork(v))
                                }),
                        )
                        .push(
                            widget::toggler(config.auto_start_break)
                                .spacing(10)
                                .label("Auto start break timer")
                                .size(15)
                                .on_toggle(|v| {
                                    Message::Settings(SettingsMessage::SetAutoStartBreak(v))
                                }),
                        ),
                )
                .push(widget::row().spacing(10).push(core.applet.text_button(
                    "Reset to default settings",
                    Message::Settings(SettingsMessage::ResetToDefault),
                ))),
        );

    core.applet.popup_container(content_list.padding(10)).into()
}
