// Copyright © 2021-22 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::fixed::{
    Action, APPNAME, BUTTON_HEIGHT, ICON, LOAD_ICON, NEXT_ICON, PAD,
    PLAY_ICON, PREV_ICON, REPLAY_ICON, TIME_ICON, TOOLBAR_HEIGHT,
    TOOLBUTTON_SIZE, VOLUME_ICON, WINDOW_HEIGHT_MIN, WINDOW_WIDTH_MIN,
};
use crate::util;
use fltk::{
    app,
    app::Sender,
    button::Button,
    enums::{Event, Font, FrameType, Key, Shortcut},
    frame::Frame,
    group::{Flex, FlexType},
    image::SvgImage,
    menu::{MenuFlag, SysMenuBar},
    misc::HelpView,
    prelude::*,
    tree::Tree,
    valuator::HorFillSlider,
    window::Window,
};

pub struct Widgets {
    pub main_window: Window,
    pub menubar: SysMenuBar,
    pub prev_button: Button,
    pub replay_button: Button,
    pub play_pause_button: Button,
    pub next_button: Button,
    pub track_tree: Tree,
    pub info_view: HelpView,
    pub volume_slider: HorFillSlider,
    pub volume_label: Frame,
    pub time_slider: HorFillSlider,
    pub time_label: Frame,
}

pub fn make(sender: Sender<Action>) -> Widgets {
    Window::set_default_xclass(APPNAME);
    let icon = SvgImage::from_data(ICON).unwrap();
    let (x, y, width, height) = get_config_window_rect();
    let mut main_window = Window::new(x, y, width, height, APPNAME);
    main_window.set_icon(Some(icon));
    main_window.size_range(
        WINDOW_WIDTH_MIN,
        WINDOW_HEIGHT_MIN,
        app::screen_size().0 as i32,
        app::screen_size().1 as i32,
    );
    main_window.make_resizable(true);
    let mut vbox =
        Flex::default().size_of_parent().with_type(FlexType::Column);
    let menubar = add_menubar(sender, width);
    let toolbar = add_toolbar(sender, width);
    let (track_tree, info_view) = add_views();
    let (
        time_slider,
        time_label,
        volume_slider,
        volume_label,
        prev_button,
        replay_button,
        play_pause_button,
        next_button,
        player_toolbar,
    ) = add_player_toolbar(sender, width);
    vbox.set_size(&menubar, BUTTON_HEIGHT);
    vbox.set_size(&toolbar, TOOLBAR_HEIGHT);
    vbox.set_size(&player_toolbar, TOOLBAR_HEIGHT);
    vbox.end();
    main_window.end();
    Widgets {
        main_window,
        menubar,
        prev_button,
        replay_button,
        play_pause_button,
        next_button,
        track_tree,
        info_view,
        volume_slider,
        volume_label,
        time_slider,
        time_label,
    }
}

fn add_menubar(sender: Sender<Action>, width: i32) -> SysMenuBar {
    let mut menubar = SysMenuBar::default().with_size(width, BUTTON_HEIGHT);
    menubar.set_frame(FrameType::NoBox);
    // TODO New etc.
    menubar.add_emit(
        "&File/&Open...\t",
        Shortcut::Ctrl | 'o',
        MenuFlag::Normal,
        sender,
        Action::OpenMLM,
    );
    // TODO etc.
    // prev flag should be MenuDivider
    menubar.add_emit(
        "&File/&Quit...\t",
        Shortcut::Ctrl | 'q',
        MenuFlag::Normal,
        sender,
        Action::Quit,
    );
    menubar
}

fn add_views() -> (Tree, HelpView) {
    let mut row = Flex::default().with_type(FlexType::Row);
    let track_tree = Tree::default();
    let mut info_view = HelpView::default().with_size(200, 200);
    info_view.set_value(
        "<font color=green>Click File→Open to open a music box…</font>",
    );
    info_view.set_text_font(Font::Helvetica);
    info_view.set_text_size((info_view.text_size() as f64 * 1.3) as i32);
    row.set_size(&info_view, 200);
    row.end();
    (track_tree, info_view)
}

fn add_player_toolbar(
    sender: Sender<Action>,
    width: i32,
) -> (
    HorFillSlider,
    Frame,
    HorFillSlider,
    Frame,
    Button,
    Button,
    Button,
    Button,
    Flex,
) {
    let mut row = Flex::default()
        .with_size(width, TOOLBAR_HEIGHT)
        .with_type(FlexType::Row);
    let prev_button = add_toolbutton(
        sender,
        "Previous track",
        Action::Previous,
        PREV_ICON,
        &mut row,
    );
    let replay_button = add_toolbutton(
        sender,
        "Replay the current track",
        Action::Replay,
        REPLAY_ICON,
        &mut row,
    );
    let play_pause_button = add_toolbutton(
        sender,
        "Play or Pause the current track",
        Action::PlayOrPause,
        PLAY_ICON,
        &mut row,
    );
    let next_button = add_toolbutton(
        sender,
        "Next track",
        Action::Next,
        NEXT_ICON,
        &mut row,
    );
    let (time_slider, time_label, volume_slider, volume_label) =
        add_sliders(&mut row);
    row.end();
    (
        time_slider,
        time_label,
        volume_slider,
        volume_label,
        prev_button,
        replay_button,
        play_pause_button,
        next_button,
        row,
    )
}

fn add_toolbutton(
    sender: Sender<Action>,
    tooltip: &str,
    action: Action,
    icon: &str,
    row: &mut Flex,
) -> Button {
    let width = TOOLBUTTON_SIZE + PAD + 8;
    let mut button = Button::default();
    button.set_size(width, TOOLBUTTON_SIZE + PAD);
    button.visible_focus(false);
    button.set_label_size(0);
    button.set_tooltip(tooltip);
    let mut icon = SvgImage::from_data(icon).unwrap();
    icon.scale(TOOLBUTTON_SIZE, TOOLBUTTON_SIZE, true, true);
    button.set_image(Some(icon));
    button.emit(sender, action);
    row.set_size(&button, width);
    button
}

fn add_sliders(
    row: &mut Flex,
) -> (HorFillSlider, Frame, HorFillSlider, Frame) {
    let (time_icon_label, time_slider, time_label) =
        add_slider_row(TIME_ICON, "0″/0″");
    let (volume_icon_label, volume_slider, volume_label) = add_volume_row();
    row.set_size(&time_icon_label, TOOLBUTTON_SIZE);
    row.set_size(&time_label, 80);
    row.set_size(&volume_label, 50);
    row.set_size(&volume_icon_label, TOOLBUTTON_SIZE);
    (time_slider, time_label, volume_slider, volume_label)
}

fn add_volume_row() -> (Frame, HorFillSlider, Frame) {
    let (icon_label, mut volume_slider, volume_label) =
        add_slider_row(VOLUME_ICON, "0%");
    volume_slider.set_range(0.0, 1.0);
    volume_slider.set_step(1.0, 10); // 1/10
    (icon_label, volume_slider, volume_label)
}

fn add_slider_row(
    icon: &str,
    label: &str,
) -> (Frame, HorFillSlider, Frame) {
    let icon_height = TOOLBUTTON_SIZE;
    let icon_width = icon_height + 8;
    let mut icon_label = Frame::default();
    icon_label.set_size(icon_width, icon_height);
    icon_label.visible_focus(false);
    icon_label.set_label_size(0);
    let mut icon_image = SvgImage::from_data(icon).unwrap();
    icon_image.scale(TOOLBUTTON_SIZE, TOOLBUTTON_SIZE, true, true);
    icon_label.set_image(Some(icon_image));
    let slider = HorFillSlider::default();
    let mut label = Frame::default().with_label(label);
    label.set_frame(FrameType::EngravedFrame);
    (icon_label, slider, label)
}

fn add_toolbar(sender: Sender<Action>, width: i32) -> Flex {
    let mut row = Flex::default()
        .with_size(width, TOOLBAR_HEIGHT)
        .with_type(FlexType::Row);
    add_toolbutton(
        sender,
        "Open a MLM file",
        Action::OpenMLM,
        LOAD_ICON,
        &mut row,
    );
    row.end();
    row
}

fn get_config_window_rect() -> (i32, i32, i32, i32) {
    let mut config = CONFIG.get().write().unwrap();
    let x = if config.window_x >= 0 {
        config.window_x
    } else {
        util::x() - (config.window_width / 2)
    };
    let y = if config.window_y >= 0 {
        config.window_y
    } else {
        util::y() - (config.window_height / 2)
    };
    if x != config.window_x {
        config.window_x = x;
    }
    if y != config.window_y {
        config.window_y = y;
    }
    (x, y, config.window_width, config.window_height)
}

pub fn add_event_handlers(
    main_window: &mut Window,
    sender: Sender<Action>,
) {
    // Both of these are really needed!
    main_window.set_callback(move |_| {
        if app::event() == Event::Close {
            sender.send(Action::Quit);
        }
    });
    main_window.handle(move |_, event| {
        if event == Event::KeyUp && app::event_key() == Key::Help {
            sender.send(Action::Help);
            return true;
        }
        false
    });
}

pub fn update_widgets_from_config(widgets: &mut Widgets) -> bool {
    dbg!("update_widgets_from_config");
    // TODO
    /*
    let config = CONFIG.get().read().unwrap();
    widgets.volume_slider.set_value(config.volume);
    widgets
        .volume_label
        .set_label(&format!("{}%", (config.volume * 100.0).round()));
    config.track.exists()
    */
    false
}
