use egui::{Context, Key, Modifiers};

use crate::ui::toolbar::ToolbarAction;

pub fn handle_shortcuts(ctx: &Context, has_file: bool) -> ToolbarAction {
    let mut action = ToolbarAction::default();

    ctx.input(|input| {
        let ctrl = if cfg!(target_os = "macos") {
            input.modifiers.mac_cmd
        } else {
            input.modifiers.ctrl
        };
        let shift = input.modifiers.shift;

        // Open file: Ctrl+O
        if ctrl && input.key_pressed(Key::O) {
            action.open_file = true;
        }

        if !has_file {
            return;
        }

        // Navigation: Left/Right arrow
        if input.key_pressed(Key::ArrowLeft) && !ctrl {
            action.prev_file = true;
        }
        if input.key_pressed(Key::ArrowRight) && !ctrl {
            action.next_file = true;
        }

        // Zoom: Ctrl+Plus / Ctrl+Minus
        if ctrl && input.key_pressed(Key::Plus) {
            action.zoom_in = true;
        }
        if ctrl && input.key_pressed(Key::Equals) {
            action.zoom_in = true;
        }
        if ctrl && input.key_pressed(Key::Minus) {
            action.zoom_out = true;
        }

        // Fit to window: Ctrl+0
        if ctrl && input.key_pressed(Key::Num0) {
            action.fit_to_window = true;
        }

        // Actual size: Ctrl+1
        if ctrl && input.key_pressed(Key::Num1) {
            action.actual_size = true;
        }

        // Rotate: R / Shift+R
        if input.key_pressed(Key::R) && !ctrl {
            if shift {
                action.rotate_ccw = true;
            } else {
                action.rotate_cw = true;
            }
        }

        // Mirror: H / V
        if input.key_pressed(Key::H) && input.modifiers == Modifiers::NONE {
            action.mirror_h = true;
        }
        if input.key_pressed(Key::V) && input.modifiers == Modifiers::NONE {
            action.mirror_v = true;
        }

        // Export: Ctrl+Shift+E
        if ctrl && shift && input.key_pressed(Key::E) {
            action.export = true;
        }

        // Copy: Ctrl+C
        if ctrl && input.key_pressed(Key::C) {
            action.copy_clipboard = true;
        }

        // Toggle background: T
        if input.key_pressed(Key::T) && input.modifiers == Modifiers::NONE {
            action.toggle_bg = true;
        }

        // Reset view: Ctrl+R
        if ctrl && input.key_pressed(Key::R) && !shift {
            action.reset_view = true;
        }

        // Quit: Ctrl+Q
        if ctrl && input.key_pressed(Key::Q) {
            std::process::exit(0);
        }
    });

    action
}
