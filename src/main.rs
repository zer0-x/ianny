extern crate gcd;
extern crate notify_rust;
extern crate single_instance;
extern crate wayland_client;
extern crate wayland_protocols;
extern crate wayland_protocols_plasma;

mod config;

use single_instance::SingleInstance;
use std::time::Duration;
use std::{
    ops::AddAssign,
    sync::{Arc, Condvar, Mutex},
};
use wayland_client::protocol::{wl_registry, wl_seat};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1, ext_idle_notifier_v1,
};
use wayland_protocols_plasma::idle::client::{org_kde_kwin_idle, org_kde_kwin_idle_timeout};

const APP_ID: &str = "io.github.zer0_x.ianny";

struct State {
    idle_notifier: Option<ext_idle_notifier_v1::ExtIdleNotifierV1>,
    kde_kwin_idle: Option<org_kde_kwin_idle::OrgKdeKwinIdle>,
    is_active: Arc<(Mutex<bool>, Condvar)>,
    idle_timeout: Arc<Duration>,
}

impl wayland_client::Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            match interface.as_str() {
                "wl_seat" => {
                    registry.bind::<wl_seat::WlSeat, _, _>(name, 1, queue_handle, ());
                }
                "ext_idle_notifier_v1" => {
                    state.idle_notifier = Some(
                        registry.bind::<ext_idle_notifier_v1::ExtIdleNotifierV1, _, _>(
                            name,
                            1,
                            queue_handle,
                            (),
                        ),
                    );

                    eprintln!("Binded to ext_idle_notifier_v1")
                }
                "org_kde_kwin_idle" => {
                    state.kde_kwin_idle =
                        Some(registry.bind::<org_kde_kwin_idle::OrgKdeKwinIdle, _, _>(
                            name,
                            1,
                            queue_handle,
                            (),
                        ));

                    eprintln!("Binded to org_kde_kwin_idle")
                }
                _ => {}
            }
        }
    }
}

impl wayland_client::Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        state: &mut Self,
        seat: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        if let Some(idle_notifier) = &state.idle_notifier {
            idle_notifier.get_idle_notification(
                state.idle_timeout.as_millis() as u32,
                seat,
                queue_handle,
                (),
            );
        }
        if let Some(kde_kwin_idle) = &state.kde_kwin_idle {
            kde_kwin_idle.get_idle_timeout(
                seat,
                state.idle_timeout.as_millis() as u32,
                queue_handle,
                (),
            );
        }
    }
}

impl wayland_client::Dispatch<ext_idle_notifier_v1::ExtIdleNotifierV1, ()> for State {
    fn event(
        _state: &mut Self,
        _idle_notifier: &ext_idle_notifier_v1::ExtIdleNotifierV1,
        _event: ext_idle_notifier_v1::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        _queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        // No events
    }
}

impl wayland_client::Dispatch<org_kde_kwin_idle::OrgKdeKwinIdle, ()> for State {
    fn event(
        _state: &mut Self,
        _kwin_idle: &org_kde_kwin_idle::OrgKdeKwinIdle,
        _event: org_kde_kwin_idle::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        _queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        // No events
    }
}

impl wayland_client::Dispatch<ext_idle_notification_v1::ExtIdleNotificationV1, ()> for State {
    fn event(
        state: &mut Self,
        _idle_notification: &ext_idle_notification_v1::ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        _queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        let (lock, cvar) = &*state.is_active;

        match &event {
            ext_idle_notification_v1::Event::Idled => {
                *lock.lock().unwrap() = false;
                cvar.notify_one();

                eprintln!("Idled");
            }
            ext_idle_notification_v1::Event::Resumed => {
                *lock.lock().unwrap() = true;
                cvar.notify_one();

                eprintln!("Resumed");
            }
            _ => {}
        }
    }
}

impl wayland_client::Dispatch<org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout, ()> for State {
    fn event(
        state: &mut Self,
        _idle_timeout: &org_kde_kwin_idle_timeout::OrgKdeKwinIdleTimeout,
        event: org_kde_kwin_idle_timeout::Event,
        _: &(),
        _conn: &wayland_client::Connection,
        _queue_handle: &wayland_client::QueueHandle<State>,
    ) {
        let (lock, cvar) = &*state.is_active;

        match &event {
            org_kde_kwin_idle_timeout::Event::Idle => {
                *lock.lock().unwrap() = false;
                cvar.notify_one();

                eprintln!("Idled");
            }
            org_kde_kwin_idle_timeout::Event::Resumed => {
                *lock.lock().unwrap() = true;
                cvar.notify_one();

                eprintln!("Resumed");
            }
            _ => {}
        }
    }
}

fn show_break_notification(break_time: Duration, notification_sound_hint: notify_rust::Hint) {
    use notify_rust::{Hint, Notification, Timeout, Urgency};

    let mut handle = Notification::new()
        // TODO: Localize messages and adapt words with number.
        .summary("Break Time!")
        .body(&format!(
            "Take a break for <b>{} minutes</b>.",
            break_time.as_secs() / 60
        ))
        .icon(APP_ID)
        .appname("Ianny")
        .hint(notification_sound_hint)
        .hint(Hint::Urgency(Urgency::Critical))
        .hint(Hint::Resident(true))
        .timeout(Timeout::Never)
        .show()
        .unwrap();

    // Progress bar.
    let step = break_time.div_f32(100.0);
    for i in 0..100 {
        handle.hint(Hint::CustomInt("value".to_owned(), i));
        handle.update();
        std::thread::sleep(step);
    }

    handle.close();
}

fn main() {
    // Check if the app is already running
    let app_instance = SingleInstance::new(APP_ID).unwrap();
    if !app_instance.is_single() {
        eprintln!("{APP_ID} is already running.");
        std::process::exit(1);
    }

    // Load config file
    let user_config = config::load_config(config::get_config_file());

    eprintln!("{:?}", &user_config);

    // Create main state for the app to store shared things.
    let mut state = State {
        idle_notifier: None,
        kde_kwin_idle: None,
        is_active: Arc::new((Mutex::new(true), Condvar::new())),
        idle_timeout: Arc::new(Duration::from_secs(&user_config.idle_timeout * 60)),
    };

    // Connect to Wayland server
    let conn = wayland_client::Connection::connect_to_env().unwrap();

    let mut event_queue = conn.new_event_queue::<State>();
    let queue_handle = event_queue.handle();

    let display = conn.display();

    let _registry = display.get_registry(&queue_handle, ());

    event_queue.roundtrip(&mut state).unwrap();

    // Thread safe clones.
    let is_active1 = Arc::clone(&state.is_active);
    let idle_timeout1 = Arc::clone(&state.idle_timeout);

    // Timer thread.
    std::thread::spawn(move || {
        let (lock, cvar) = &*is_active1;

        let short_break_timeout = &user_config.short_break_timeout * 60; // secands
        let long_break_timeout = &user_config.long_break_tiemout * 60; // secands

        let pause_duration = std::cmp::min(
            gcd::binary_u64(short_break_timeout, long_break_timeout), // Calculate GCD
            idle_timeout1.as_secs() + 1, // Extra one second to make sure
        ); // secands

        let mut short_time_pased = 0; // secands
        let mut long_time_pased = 0; // secands

        // Timer loop.
        loop {
            std::thread::sleep(Duration::from_secs(pause_duration));
            short_time_pased.add_assign(pause_duration);
            long_time_pased.add_assign(pause_duration);

            let is_active_guard = lock.lock().unwrap();

            if *is_active_guard {
                if long_time_pased >= long_break_timeout {
                    eprintln!("Long break starts");

                    show_break_notification(
                        Duration::from_secs(&user_config.long_break_duration * 60),
                        notify_rust::Hint::SoundName("suspend-error".to_owned()), // Name or file
                    );

                    eprintln!("Long break ends");

                    // Reset timers.
                    long_time_pased = 0;
                    short_time_pased = 0;
                } else if short_time_pased >= short_break_timeout {
                    eprintln!("Short break starts");

                    show_break_notification(
                        Duration::from_secs(&user_config.short_break_duration * 60),
                        notify_rust::Hint::SoundName("suspend-error".to_owned()), // Name or file
                    );

                    eprintln!("Short break ends");

                    // Reset timer.
                    short_time_pased = 0;
                }
            } else if !*is_active_guard {
                // Wait for change, when user resume from idle.
                let _guard = cvar.wait(is_active_guard).unwrap();

                // Reset timers.
                long_time_pased = 0;
                short_time_pased = 0;

                eprintln!("Timer resetted");
            }
        }
    });

    // Main loop.
    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
