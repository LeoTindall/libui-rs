use super::Control;
use std::os::raw::c_void;
use std::mem;
use ui::UI;
use ui_sys::{self, uiButton, uiControl, uiLabel};
use str_tools::{from_toolkit_string, to_toolkit_string};

define_control!{
    /// A non-interactable piece of text.
    rust_type: Label,
    sys_type: uiLabel
}

define_control!{
    /// A textual button which users can click on, causing a callback to run.
    rust_type: Button,
    sys_type: uiButton
}

impl Button {
    /// Create a new button with the given text as its label.
    pub fn new(_ctx: &UI, text: &str) -> Button {
        let c_string = to_toolkit_string(text);
        unsafe {
            Button::from_raw(ui_sys::uiNewButton(c_string.as_ptr()))
        }
    }

    /// Get a copy of the existing text on the button.
    pub fn text(&self, _ctx: &UI) -> String {
        unsafe {
            from_toolkit_string(ui_sys::uiButtonText(self.uiButton))
        }
    }

    /// Set the text on the button.
    pub fn set_text(&mut self, _ctx: &UI, text: &str) {
        let c_string = to_toolkit_string(text);
        unsafe {
            ui_sys::uiButtonSetText(self.uiButton, c_string.as_ptr())
        }
    }

    /// Run the given callback when the button is clicked.
    pub fn on_clicked<'ctx, F: FnMut(&mut Button) + 'ctx>(&mut self, _ctx: &'ctx UI, callback: F) {
        unsafe {
            let mut data: Box<Box<FnMut(&mut Button)>> = Box::new(Box::new(callback));
            ui_sys::uiButtonOnClicked(
                self.uiButton,
                Some(c_callback),
                &mut *data as *mut Box<FnMut(&mut Button)> as *mut c_void,
            );
            mem::forget(data);
        }

        extern "C" fn c_callback(button: *mut uiButton, data: *mut c_void) {
            unsafe {
                let mut button = Button { uiButton: button };
                mem::transmute::<*mut c_void, &mut Box<FnMut(&mut Button)>>(data)(&mut button)
            }
        }
    }
}

impl Label {
    /// Create a new label with the given string as its text.
    /// Note that labels do not auto-wrap their text; they will expand as far as needed
    /// to fit.
    pub fn new(_ctx: &UI, text: &str) -> Label {
        let c_string = to_toolkit_string(text);
        unsafe {
            Label::from_raw(ui_sys::uiNewLabel(c_string.as_ptr()))
        }
    }

    /// Get a copy of the existing text on the label.
    pub fn text(&self, _ctx: &UI) -> String {
        unsafe {
            from_toolkit_string(ui_sys::uiLabelText(self.uiLabel))
        }
    }

    /// Set the text on the label.
    pub fn set_text(&mut self, _ctx: &UI, text: &str) {
        let c_string = to_toolkit_string(text);
        unsafe {
            ui_sys::uiLabelSetText(self.uiLabel, c_string.as_ptr())
        }
    }
}
