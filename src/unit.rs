use crate::wstrcpy;

pub const ROOT_UNIT_ID: i32 = 0;
pub const NO_PARENT_UNIT_ID: i32 = -1;

pub const NO_PROGRAM_LIST_ID: i32 = -1;

pub struct Unit {
    pub id: i32,
    pub parent_unit_id: i32,
    pub name: String,
    pub program_list_id: i32,
}

impl Unit {
    pub fn get_info(&self) -> vst3_sys::vst::UnitInfo {
        let mut unit_info = vst3_sys::vst::UnitInfo {
            id: self.id,
            parent_unit_id: self.parent_unit_id,
            name: [0; 128],
            program_list_id: self.program_list_id,
        };
        unsafe { wstrcpy(&self.name, unit_info.name.as_mut_ptr() as *mut i16) };
        unit_info
    }
}

pub struct UnitBuilder {
    id: i32,
    parent_unit_id: i32,
    name: String,
    program_list_id: i32,
}

impl UnitBuilder {
    pub fn new(name: &str, id: i32) -> Self {
        Self {
            id,
            parent_unit_id: ROOT_UNIT_ID,
            name: name.to_string(),
            program_list_id: NO_PROGRAM_LIST_ID,
        }
    }

    pub fn parent_unit_id(mut self, parent_unit_id: i32) -> Self {
        self.parent_unit_id = parent_unit_id;
        self
    }

    pub fn program_list_id(mut self, program_list_id: i32) -> Self {
        self.program_list_id = program_list_id;
        self
    }

    pub fn build(&self) -> Unit {
        Unit {
            id: self.id,
            parent_unit_id: self.parent_unit_id,
            name: self.name.clone(),
            program_list_id: self.program_list_id,
        }
    }
}

pub struct ProgramList {
    pub id: i32,
    pub name: String,
    pub program_count: i32,
}

impl ProgramList {
    pub fn new(id: i32, name: &str, program_count: i32) -> Self {
        Self {
            id,
            name: name.to_string(),
            program_count,
        }
    }

    pub fn get_program_list_info(&self) -> vst3_sys::vst::ProgramListInfo {
        let mut program_list_info = vst3_sys::vst::ProgramListInfo {
            id: self.id,
            name: [0; 128],
            program_count: self.program_count,
        };
        unsafe {
            wstrcpy(&self.name, program_list_info.name.as_mut_ptr() as *mut i16);
        }

        program_list_info
    }
}
