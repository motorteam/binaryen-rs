use std::ffi::CString;

use binaryen_sys::*;

#[repr(transparent)]
pub struct Module(BinaryenModuleRef);

impl Module {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let module = unsafe { BinaryenModuleRead(bytes.as_ptr() as *mut i8, bytes.len()) };
        Self(module)
    }

    pub fn from_bytes_with_features(bytes: &[u8], features: Features) -> Self {
        let module = unsafe {
            BinaryenModuleReadWithFeatures(bytes.as_ptr() as *mut i8, bytes.len(), features.0)
        };
        Self(module)
    }

    pub fn set_always_inline_max_size(&mut self, max_size: u32) {
        unsafe { BinaryenSetAlwaysInlineMaxSize(max_size) };
    }

    pub fn set_one_caller_inline_max_size(&mut self, max_size: u32) {
        unsafe { BinaryenSetOneCallerInlineMaxSize(max_size) };
    }

    pub fn set_debug_info(&mut self, debug: bool) {
        unsafe { BinaryenSetDebugInfo(debug) };
    }

    pub fn set_optimize_level(&mut self, optimize_level: i32) {
        unsafe { BinaryenSetOptimizeLevel(optimize_level) };
    }

    pub fn set_shrink_level(&mut self, shrink_level: i32) {
        unsafe { BinaryenSetShrinkLevel(shrink_level) };
    }

    pub fn run_passes(&mut self, passes: &[&str]) {
        let c_passes: Vec<CString> = passes.iter().map(|s| CString::new(*s).unwrap()).collect();

        let mut ptrs: Vec<*const i8> = c_passes.iter().map(|cs| cs.as_ptr()).collect();

        unsafe {
            BinaryenModuleRunPasses(self.0, ptrs.as_mut_ptr(), ptrs.len().try_into().unwrap())
        };
    }

    pub fn optimize(&mut self) {
        unsafe { BinaryenModuleOptimize(self.0) };
    }

    pub fn into_binary(self) -> Vec<u8> {
        unsafe {
            let data = BinaryenModuleAllocateAndWrite(self.0, std::ptr::null());
            let result =
                std::slice::from_raw_parts(data.binary as *mut u8, data.binaryBytes).to_vec();
            libc::free(data.binary as *mut libc::c_void);
            result
        }
    }
}

#[repr(transparent)]
pub struct Features(BinaryenFeatures);

impl Features {
    pub fn all() -> Self {
        Self(unsafe { BinaryenFeatureAll() })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { BinaryenModuleDispose(self.0) };
    }
}
