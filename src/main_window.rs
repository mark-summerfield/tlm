// Copyright © 2022 Mark Summerfield. All rights reserved.
// License: GPLv3

use super::CONFIG;
use crate::fixed::{
    Action, APPNAME, BUTTON_HEIGHT, DEMOTE_ICON, FILE_NEW_ICON,
    FILE_OPEN_ICON, FILE_SAVE_ICON, HISTORY_ICON, ICON, LIST_ADD_ICON,
    LIST_ICON, MOVE_DOWN_ICON, MOVE_UP_ICON, NEXT_ICON, PAD, PLAY_ICON,
    PREV_ICON, PROMOTE_ICON, REPLAY_ICON, TIME_ICON, TOOLBAR_HEIGHT,
    TOOLBUTTON_SIZE, TRACK_ADD_ICON, TRACK_FIND_ICON, TREE_ICON_SIZE,
    VOLUME_ICON, WINDOW_HEIGHT_MIN, WINDOW_WIDTH_MIN,
};
use crate::util;
use fltk::{
    app,
    app::Sender,
    button::Button,
    enums::{Event, Font, FrameType, Key, Shortcut},
    frame::Frame,
    group::Flex,
    image::SvgImage,
    menu::{MenuButton, MenuFlag, SysMenuBar},
    misc::HelpView,
    prelude::*,
    tree::{Tree, TreeSelect},
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
    pub history_menu_button: MenuButton,
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
    let mut vbox = Flex::default().column().size_of_parent();
    let menubar = add_menubar(sender, width);
    let (history_menu_button, toolbar) = add_toolbar(sender, width);
    let (track_tree, info_view) = add_views(sender, width);
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
        history_menu_button,
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
    menubar.add_emit(
        "&File/&New…\t",
        Shortcut::Ctrl | 'n',
        MenuFlag::Normal,
        sender,
        Action::FileNew,
    );
    menubar.add_emit(
        "&File/&Open…\t",
        Shortcut::Ctrl | 'o',
        MenuFlag::Normal,
        sender,
        Action::FileOpen,
    );
    menubar.add_emit(
        "&File/Open &Recent…",
        Shortcut::None,
        MenuFlag::Normal,
        sender,
        Action::FileOpenRecent,
    );
    menubar.add_emit(
        "&File/&Save\t",
        Shortcut::Ctrl | 's',
        MenuFlag::Normal,
        sender,
        Action::FileSave,
    );
    menubar.add_emit(
        "&File/Save &As…\t",
        Shortcut::None,
        MenuFlag::MenuDivider,
        sender,
        Action::FileSaveAs,
    );
    menubar.add_emit(
        "&File/&Configure…\t",
        Shortcut::None,
        MenuFlag::MenuDivider,
        sender,
        Action::FileConfigure,
    );
    menubar.add_emit(
        "&File/&Quit\t",
        Shortcut::Ctrl | 'q',
        MenuFlag::Normal,
        sender,
        Action::FileQuit,
    );
    menubar.add_emit(
        "&Edit/Move &Up\t",
        Shortcut::None,
        MenuFlag::Normal,
        sender,
        Action::EditMoveUp,
    );
    menubar.add_emit(
        "&Edit/Move &Down\t",
        Shortcut::None,
        MenuFlag::MenuDivider,
        sender,
        Action::EditMoveDown,
    );
    menubar.add_emit(
        "&Edit/&Promote\t",
        Shortcut::None,
        MenuFlag::Normal,
        sender,
        Action::EditPromote,
    );
    menubar.add_emit(
        "&Edit/De&mote\t",
        Shortcut::None,
        MenuFlag::MenuDivider,
        sender,
        Action::EditDemote,
    );
    menubar.add_emit(
        "&Edit/&Find…\t",
        Shortcut::Ctrl | 'f',
        MenuFlag::Normal,
        sender,
        Action::EditFind,
    );
    menubar.add_emit(
        "&Edit/Find &Again\t",
        Shortcut::from_key(Key::F3),
        MenuFlag::MenuDivider,
        sender,
        Action::EditFindAgain,
    );
    menubar.add_emit(
        "&Edit/D&elete\t",
        Shortcut::None,
        MenuFlag::Normal,
        sender,
        Action::EditDelete,
    );
    menubar.add_emit(
        "&List/&Add…\t",
        Shortcut::Ctrl | 'l',
        MenuFlag::Normal,
        sender,
        Action::ListAdd,
    );
    menubar.add_emit(
        "&List/&Rename\t",
        Shortcut::None,
        MenuFlag::MenuDivider,
        sender,
        Action::ListRename,
    );
    menubar.add_emit(
        "&Track/&Add…\t",
        Shortcut::Ctrl | 't',
        MenuFlag::MenuDivider,
        sender,
        Action::TrackAdd,
    );
    menubar.add_emit(
        "&Track/Play Pre&vious\t",
        Shortcut::from_key(Key::F4),
        MenuFlag::Normal,
        sender,
        Action::TrackPrevious,
    );
    menubar.add_emit(
        "&Track/&Replay\t",
        Shortcut::from_key(Key::F5),
        MenuFlag::Normal,
        sender,
        Action::TrackReplay,
    );
    menubar.add_emit(
        "&Track/&Play or Pause\t",
        Shortcut::from_key(Key::F6),
        MenuFlag::Normal,
        sender,
        Action::TrackPlayOrPause,
    );
    menubar.add_emit(
        "&Track/Play &Next\t",
        Shortcut::from_key(Key::F7),
        MenuFlag::MenuDivider,
        sender,
        Action::TrackNext,
    );
    menubar.add_emit(
        "&Track/&History…\t",
        Shortcut::Ctrl | 'g',
        MenuFlag::MenuDivider,
        sender,
        Action::TrackHistory,
    );
    menubar.add_emit(
        "&Track/&Decrease Volume\t",
        Shortcut::from_key(Key::F8),
        MenuFlag::Normal,
        sender,
        Action::TrackQuieter,
    );
    menubar.add_emit(
        "&Track/&Increase Volume\t",
        Shortcut::from_key(Key::F9),
        MenuFlag::Normal,
        sender,
        Action::TrackLouder,
    );
    menubar.add_emit(
        "&Help/&Help\t",
        Shortcut::from_key(Key::F1),
        MenuFlag::Normal,
        sender,
        Action::HelpHelp,
    );
    menubar.add_emit(
        "&Help/&About\t",
        Shortcut::None,
        MenuFlag::Normal,
        sender,
        Action::HelpAbout,
    );
    menubar
}

fn add_views(sender: Sender<Action>, width: i32) -> (Tree, HelpView) {
    const HEIGHT: i32 = 70;
    let mut row = Flex::default().column();
    let mut icon = SvgImage::from_data(LIST_ICON).unwrap();
    icon.scale(TREE_ICON_SIZE, TREE_ICON_SIZE, true, true);
    let mut track_tree = Tree::default();
    track_tree.set_show_root(false);
    track_tree.set_select_mode(TreeSelect::Single);
    track_tree.set_user_icon(Some(icon));
    #[allow(clippy::clone_on_copy)]
    let sender = sender.clone();
    track_tree.handle(move |_, _| {
        if app::event() == Event::Push && app::event_clicks() {
            sender.send(Action::TreeItemDoubleClicked);
        }
        false
    });
    let mut info_view = HelpView::default().with_size(width, HEIGHT);
    info_view.set_value(
        "<font color=green>Click <b>List→New</b> to add a folder of tracks
         or <b>Track→New</b> to add an individual track 
         or <b>File→Open</b> to open an existing TLM file…</font>",
    );
    info_view.set_text_font(Font::Helvetica);
    info_view.set_text_size((info_view.text_size() as f64 * 1.3) as i32);
    row.set_size(&info_view, HEIGHT);
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
    let mut row = Flex::default().row().with_size(width, TOOLBAR_HEIGHT);
    let prev_button = add_toolbutton(
        sender,
        "Previous track • F4",
        Action::TrackPrevious,
        PREV_ICON,
        &mut row,
    );
    let replay_button = add_toolbutton(
        sender,
        "Replay the current track • F5",
        Action::TrackReplay,
        REPLAY_ICON,
        &mut row,
    );
    let play_pause_button = add_toolbutton(
        sender,
        "Play or Pause the current track • F6",
        Action::TrackPlayOrPause,
        PLAY_ICON,
        &mut row,
    );
    let next_button = add_toolbutton(
        sender,
        "Next track • F7",
        Action::TrackNext,
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

fn add_menubutton(
    tooltip: &str,
    icon: &str,
    button_box: &mut Flex,
) -> MenuButton {
    let width = TOOLBUTTON_SIZE + PAD + 8;
    let mut button = MenuButton::default();
    button.set_size(width, TOOLBUTTON_SIZE + PAD);
    button.visible_focus(false);
    button.set_label_size(0);
    button.set_tooltip(tooltip);
    let mut icon = SvgImage::from_data(icon).unwrap();
    icon.scale(TOOLBUTTON_SIZE, TOOLBUTTON_SIZE, true, true);
    button.set_image(Some(icon));
    button_box.set_size(&button, width);
    button
}

fn add_sliders(
    row: &mut Flex,
) -> (HorFillSlider, Frame, HorFillSlider, Frame) {
    let (time_icon_label, time_slider, time_label) =
        add_slider_row(TIME_ICON, "0″/0″");
    let (volume_icon_label, volume_slider, volume_label) = add_volume_row();
    row.set_size(&time_icon_label, TOOLBUTTON_SIZE);
    row.set_size(&time_label, 100);
    row.set_size(&volume_label, 50);
    row.set_size(&volume_icon_label, TOOLBUTTON_SIZE);
    (time_slider, time_label, volume_slider, volume_label)
}

fn add_volume_row() -> (Frame, HorFillSlider, Frame) {
    let (icon_label, mut volume_slider, volume_label) =
        add_slider_row(VOLUME_ICON, "0%");
    volume_slider.set_range(0.0, 1.0);
    volume_slider.set_step(1.0, 10); // 1/10
    volume_slider.set_tooltip("Decrease • F8 | Increase • F9");
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

fn add_toolbar(sender: Sender<Action>, width: i32) -> (MenuButton, Flex) {
    let mut row = Flex::default().row().with_size(width, TOOLBAR_HEIGHT);
    add_toolbutton(
        sender,
        "New TLM file…",
        Action::FileNew,
        FILE_NEW_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Open a TLM file…",
        Action::FileOpen,
        FILE_OPEN_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Save the TLM file",
        Action::FileSave,
        FILE_SAVE_ICON,
        &mut row,
    );
    add_separator(&mut row);
    add_toolbutton(
        sender,
        "Move Up",
        Action::EditMoveUp,
        MOVE_UP_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Move Down",
        Action::EditMoveDown,
        MOVE_DOWN_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Promote",
        Action::EditPromote,
        PROMOTE_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Demote",
        Action::EditDemote,
        DEMOTE_ICON,
        &mut row,
    );
    add_toolbutton(
        sender,
        "Find…",
        Action::EditFind,
        TRACK_FIND_ICON,
        &mut row,
    );
    add_separator(&mut row);
    add_toolbutton(
        sender,
        "Add List…",
        Action::ListAdd,
        LIST_ADD_ICON,
        &mut row,
    );
    add_separator(&mut row);
    add_toolbutton(
        sender,
        "Add Track…",
        Action::TrackAdd,
        TRACK_ADD_ICON,
        &mut row,
    );
    let history_menu_button =
        add_menubutton("History", HISTORY_ICON, &mut row);
    row.end();
    (history_menu_button, row)
}

fn add_separator(row: &mut Flex) {
    let mut frame =
        Frame::default().with_size(PAD / 2, TOOLBUTTON_SIZE + PAD);
    frame.set_frame(FrameType::DownBox);
    row.set_size(&frame, PAD / 2);
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
            sender.send(Action::FileQuit);
        }
    });
    main_window.handle(move |_, event| {
        if event == Event::KeyUp && app::event_key() == Key::Help {
            sender.send(Action::HelpHelp);
            return true;
        }
        false
    });
}

pub fn update_widgets_from_config(widgets: &mut Widgets) -> bool {
    let config = CONFIG.get().read().unwrap();
    widgets.volume_slider.set_value(config.volume);
    widgets
        .volume_label
        .set_label(&format!("{}%", (config.volume * 100.0).round()));
    config.last_file.exists()
}
