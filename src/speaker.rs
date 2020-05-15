pub const EMPTY: u64 = vst3_sys::vst::kEmpty;
pub const MONO: u64 = vst3_sys::vst::kMono;
pub const STEREO: u64 = vst3_sys::vst::kStereo;

pub fn get_channel_count(mut arr: u64) -> i32 {
    let mut count = 0;
    while arr != 0 {
        if arr & 1 == 1 {
            count += 1;
        }
        arr >>= 1;
    }
    count
}
