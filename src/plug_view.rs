use std::ptr::null_mut;

use vst3_sys::gui::{IPlugView, ViewRect};
use vst3_sys::VST3;

use crate::unknown::ResultErr;
use std::os::raw::c_void;
use std::sync::{mpsc, Arc, Condvar, Mutex};

use core::mem::size_of;
use core::mem::MaybeUninit;
use core::panic::PanicInfo;

use crate::ResultErr::ResultFalse;
use crate::ResultOk;
use crate::ResultOk::ResOk;
use log::Log;
use std::any::Any;
use std::ffi::CString;
use vst3_com::offset::Offset;

pub trait PlugView {
    fn new() -> Box<Self>
    where
        Self: Sized + Default,
    {
        Box::new(Default::default())
    }

    fn is_platform_type_supported(&self, platform_type: String) -> ResultErr;
    fn attached(&mut self, parent: *mut c_void, platform_type: String) -> ResultErr;
    fn removed(&mut self) -> ResultErr;
    fn on_wheel(&self, distance: f32) -> ResultErr;
    fn on_key_down(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr;
    fn on_key_up(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr;
    fn get_size(&self) -> Result<ViewRect, ResultErr>;
    fn on_size(&self) -> Result<ViewRect, ResultErr>;
    fn on_focus(&self, state: bool) -> ResultErr;
    // todo: change i32 to PlugFrame
    fn set_frame(&self, frame: i32) -> ResultErr;
    fn can_resize(&self) -> ResultErr;
    fn check_size_constraint(&self, rect: &mut ViewRect) -> ResultErr;
}

struct DummyPlugView {}

impl Default for DummyPlugView {
    fn default() -> Self {
        Self {}
    }
}

impl PlugView for DummyPlugView {
    fn is_platform_type_supported(&self, platform_type: String) -> ResultErr {
        unimplemented!()
    }

    fn attached(&mut self, parent: *mut c_void, platform_type: String) -> ResultErr {
        unimplemented!()
    }

    fn removed(&mut self) -> ResultErr {
        unimplemented!()
    }

    fn on_wheel(&self, distance: f32) -> ResultErr {
        unimplemented!()
    }

    fn on_key_down(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr {
        unimplemented!()
    }

    fn on_key_up(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr {
        unimplemented!()
    }

    fn get_size(&self) -> Result<ViewRect, ResultErr> {
        unimplemented!()
    }

    fn on_size(&self) -> Result<ViewRect, ResultErr> {
        unimplemented!()
    }

    fn on_focus(&self, state: bool) -> ResultErr {
        unimplemented!()
    }

    fn set_frame(&self, frame: i32) -> ResultErr {
        unimplemented!()
    }

    fn can_resize(&self) -> ResultErr {
        unimplemented!()
    }

    fn check_size_constraint(&self, rect: &mut ViewRect) -> ResultErr {
        unimplemented!()
    }
}

pub struct WebPlugView {}

impl Default for WebPlugView {
    fn default() -> Self {
        Self {}
    }
}

impl PlugView for WebPlugView {
    fn is_platform_type_supported(&self, platform_type: String) -> ResultErr {
        unimplemented!()
    }

    fn attached(&mut self, parent: *mut c_void, platform_type: String) -> ResultErr {
        ResultFalse
    }

    fn removed(&mut self) -> ResultErr {
        ResultFalse
    }

    fn on_wheel(&self, distance: f32) -> ResultErr {
        unimplemented!()
    }

    fn on_key_down(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr {
        unimplemented!()
    }

    fn on_key_up(&self, key: char, key_code: i16, modifiers: i16) -> ResultErr {
        unimplemented!()
    }

    fn get_size(&self) -> Result<ViewRect, ResultErr> {
        unimplemented!()
    }

    fn on_size(&self) -> Result<ViewRect, ResultErr> {
        unimplemented!()
    }

    fn on_focus(&self, state: bool) -> ResultErr {
        unimplemented!()
    }

    fn set_frame(&self, frame: i32) -> ResultErr {
        unimplemented!()
    }

    fn can_resize(&self) -> ResultErr {
        unimplemented!()
    }

    fn check_size_constraint(&self, rect: &mut ViewRect) -> ResultErr {
        unimplemented!()
    }
}

#[repr(C)]
pub(crate) struct VST3PlugView<'a> {
    __iplugviewvptr: *const <dyn IPlugView as vst3_com::ComInterface>::VTable,
    __refcnt: std::cell::Cell<u32>,
    inner: Mutex<Option<&'a mut Box<dyn PlugView>>>,
}

struct Offset0;

impl Offset for Offset0 {
    const VALUE: usize = 0;
}

impl<'a> VST3PlugView<'a> {
    fn allocate(inner: Mutex<Option<&mut Box<dyn PlugView>>>) -> Box<VST3PlugView> {
        let iplugview_vtable = <dyn IPlugView as ::vst3_com::ProductionComInterface<
            VST3PlugView,
        >>::vtable::<Offset0>();
        let __iplugviewvptr = Box::into_raw(Box::new(iplugview_vtable));
        let out = VST3PlugView {
            __iplugviewvptr,
            __refcnt: std::cell::Cell::new(1),
            inner,
        };
        Box::new(out)
    }
}

unsafe impl<'a> vst3_com::CoClass for VST3PlugView<'a> {}

impl<'a> vst3_com::interfaces::IUnknown for VST3PlugView<'a> {
    unsafe fn query_interface(
        &self,
        riid: *const vst3_com::sys::IID,
        ppv: *mut *mut std::ffi::c_void,
    ) -> vst3_com::sys::HRESULT {
        let riid = &*riid;
        if riid == &vst3_com::interfaces::iunknown::IID_IUNKNOWN {
            *ppv = &self.__iplugviewvptr as *const _ as *mut std::ffi::c_void;
        } else if <dyn IPlugView as vst3_com::ComInterface>::is_iid_in_inheritance_chain(riid) {
            *ppv = &self.__iplugviewvptr as *const _ as *mut std::ffi::c_void;
        } else {
            *ppv = std::ptr::null_mut::<std::ffi::c_void>();
            return vst3_com::sys::E_NOINTERFACE;
        }
        self.add_ref();
        vst3_com::sys::NOERROR
    }

    unsafe fn add_ref(&self) -> u32 {
        let value = self
            .__refcnt
            .get()
            .checked_add(1)
            .expect("Overflow of reference count");
        self.__refcnt.set(value);
        value
    }

    unsafe fn release(&self) -> u32 {
        let value = self
            .__refcnt
            .get()
            .checked_sub(1)
            .expect("Underflow of reference count");
        self.__refcnt.set(value);
        let __refcnt = self.__refcnt.get();
        if __refcnt == 0 {
            Box::from_raw(
                self.__iplugviewvptr as *mut <dyn IPlugView as vst3_com::ComInterface>::VTable,
            );
            Box::from_raw(self as *const _ as *mut VST3PlugView);
        }
        __refcnt
    }
}

impl<'a> VST3PlugView<'a> {
    pub(crate) fn new() -> Box<Self> {
        Self::allocate(Mutex::new(None))
    }

    pub(crate) fn set_plug_view(&mut self, plug_view: &'a mut Box<dyn PlugView>) {
        self.inner = Mutex::new(Some(plug_view))
    }
}

impl<'a> IPlugView for VST3PlugView<'a> {
    unsafe fn is_platform_type_supported(&self, type_: *const i8) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn attached(&self, parent: *mut c_void, _type_: *const i8) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn removed(&self) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn on_wheel(&self, distance: f32) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn on_key_down(&self, key: i16, key_code: i16, modifiers: i16) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn on_key_up(&self, key: i16, key_code: i16, modifiers: i16) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn get_size(&self, size: *mut ViewRect) -> i32 {
        (*size).top = 0;
        (*size).left = 0;
        (*size).bottom = 600;
        (*size).right = 600;
        vst3_sys::base::kResultOk
    }

    unsafe fn on_size(&self, new_size: *mut ViewRect) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn on_focus(&self, state: u8) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn set_frame(&self, frame: *mut c_void) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn can_resize(&self) -> i32 {
        vst3_sys::base::kResultOk
    }

    unsafe fn check_size_constraint(&self, rect: *mut ViewRect) -> i32 {
        vst3_sys::base::kResultOk
    }
}
