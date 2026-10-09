use gpui::{
    App, AppContext, Application, Bounds, SharedString, TitlebarOptions, WindowBounds,
    WindowOptions, px, size,
};

mod animation_inspector;
mod components;
mod compositing_inspector;
mod hierarchy;
mod inspector_edit;
mod narration_inspector;
mod panels;
mod review;
mod session;
mod shell;
mod theme;

#[cfg(test)]
mod compositing_predecessor;
#[cfg(test)]
mod compositing_tests;
#[cfg(test)]
mod narration_tests;
#[cfg(test)]
mod review_tests;
#[cfg(test)]
mod tests;

use shell::Shell;

fn main() {
    #[cfg(target_os = "linux")]
    {
        let is_wsl = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|release| release.to_ascii_lowercase().contains("microsoft"));
        let x11_configured = std::env::var_os("DISPLAY").is_some_and(|display| !display.is_empty());
        let wayland_configured =
            std::env::var_os("WAYLAND_DISPLAY").is_some_and(|display| !display.is_empty());

        if is_wsl && x11_configured && wayland_configured {
            // GPUI 0.2.2 requires xdg_wm_base v2+, while current WSLg advertises v1 and panics.
            // Safety: this is the first statement in `main`, before GPUI or anything
            // else has spawned a thread. No other thread exists yet, so keep this
            // block first if anything is added above it.
            unsafe {
                std::env::remove_var("WAYLAND_DISPLAY");
            }
        }
    }

    let startup = session::Startup::parse(std::env::args_os().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(600.)), cx);
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("OpenCut")),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Maximized(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let shell = cx.new(|cx| {
                    cx.observe_window_appearance(window, |_, window, _| {
                        window.refresh();
                    })
                    .detach();

                    Shell::new(startup.clone(), cx)
                });
                let closing = shell.clone();
                window.on_window_should_close(cx, move |_, cx| {
                    closing.update(cx, |shell, cx| {
                        if shell.session.busy {
                            shell.review.feedback =
                                Some("Wait for active work to finish before closing.".into());
                            cx.notify();
                            false
                        } else {
                            true
                        }
                    })
                });
                shell
            },
        )
        .expect("failed to open the main window");
    });
}
