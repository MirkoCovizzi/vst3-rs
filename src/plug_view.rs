use std::ptr::null_mut;

use vst3_com::c_void;
use vst3_sys::gui::{IPlugView, ViewRect};
use vst3_sys::VST3;

// todo: implement this!
pub struct PlugFrame {}

use crate::unknown::ResultErr;

pub trait PlugView {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn is_platform_type_supported(&self, type_: String) -> ResultErr;
    fn attached(&self, parent: *mut c_void, type_: String) -> ResultErr;
    fn removed(&self) -> ResultErr;
    fn on_wheel(&self, distance: f32) -> ResultErr;
    fn on_key_down(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr;
    fn on_key_up(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr;
    fn get_size(&self) -> Result<ViewRect, ResultErr>;
    fn on_size(&self) -> Result<ViewRect, ResultErr>;
    fn on_focus(&self, state: bool) -> ResultErr;
    fn set_frame(&self, frame: PlugFrame) -> ResultErr;
    fn can_resize(&self) -> ResultErr;
    fn check_size_constraint(&self, rect: &mut ViewRect) -> ResultErr;
}

#[VST3(implements(IPlugView))]
pub(crate) struct VST3PlugView {
    inner: *mut c_void,
}

impl VST3PlugView {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(null_mut())
    }

    pub(crate) fn set_plug_view(&mut self, plug_view: *mut c_void) {
        self.inner = plug_view
    }

    // todo: wrap with Mutex?
    #[allow(clippy::borrowed_box)]
    unsafe fn get_plug_view(&self) -> &Box<dyn PlugView> {
        *(self.inner as *mut &Box<dyn PlugView>)
    }
}

impl IPlugView for VST3PlugView {
    unsafe fn is_platform_type_supported(&self, type_: *const i8) -> i32 {
        unimplemented!()
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: *const i8) -> i32 {
        unimplemented!()
    }

    unsafe fn removed(&self) -> i32 {
        unimplemented!()
    }

    unsafe fn on_wheel(&self, distance: f32) -> i32 {
        unimplemented!()
    }

    unsafe fn on_key_down(&self, key: i16, key_code: i16, modifiers: i16) -> i32 {
        unimplemented!()
    }

    unsafe fn on_key_up(&self, key: i16, key_code: i16, modifiers: i16) -> i32 {
        unimplemented!()
    }

    unsafe fn get_size(&self, size: *mut ViewRect) -> i32 {
        unimplemented!()
    }

    unsafe fn on_size(&self, new_size: *mut ViewRect) -> i32 {
        unimplemented!()
    }

    unsafe fn on_focus(&self, state: u8) -> i32 {
        unimplemented!()
    }

    unsafe fn set_frame(&self, frame: *mut c_void) -> i32 {
        unimplemented!()
    }

    unsafe fn can_resize(&self) -> i32 {
        unimplemented!()
    }

    unsafe fn check_size_constraint(&self, rect: *mut ViewRect) -> i32 {
        unimplemented!()
    }
}
