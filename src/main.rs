//! Suzhou: a desktop home for a team of always-on Bots.

mod anim;
mod assets;
mod backend;
mod data;
mod model;
mod store;
mod theme;
mod ui;

use gpui::{
    App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, point, prelude::*, px,
    size,
};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args: Vec<String> = std::env::args().collect();
    let options = ui::app::LaunchOptions {
        demo: args.iter().any(|a| a == "--demo"),
        reset: args.iter().any(|a| a == "--reset"),
    };
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "suzhou [--demo] [--reset]\n\n  --demo   start signed in with a sample team\n  --reset  forget everything and start at sign-in\n\nSUZHOU_BACKEND=mock|pi chooses who does the work (default: mock)."
        );
        return;
    }

    Application::new()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
            assets::load_fonts(cx);
            ui::init(cx);
            let bounds = Bounds::centered(None, size(px(1180.), px(780.)), cx);
            let window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        titlebar: Some(TitlebarOptions {
                            title: Some("Suzhou".into()),
                            appears_transparent: true,
                            traffic_light_position: Some(point(px(18.), px(18.))),
                        }),
                        window_min_size: Some(size(px(720.), px(520.))),
                        app_id: Some("suzhou".into()),
                        ..Default::default()
                    },
                    |window, cx| cx.new(|cx| ui::app::AppView::new(options, window, cx)),
                )
                .expect("could not open the main window");
            window
                .update(cx, |view, window, cx| {
                    view.focus_initial(window, cx);
                    cx.activate(true);
                })
                .ok();
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
        });
}
