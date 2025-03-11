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

    pub fn optimize(&mut self, debug: bool, optimize_level: i32, shrink_level: i32) {
        unsafe {
            BinaryenSetDebugInfo(debug);
            BinaryenSetOptimizeLevel(optimize_level);
            BinaryenSetShrinkLevel(shrink_level);
            BinaryenModuleOptimize(self.0)
        };
    }

    pub fn to_binary(self) -> Vec<u8> {
        unsafe {
            let data = BinaryenModuleAllocateAndWrite(self.0, std::ptr::null());
            let data_slice = std::slice::from_raw_parts(data.binary as *mut u8, data.binaryBytes);
            let result = Vec::from(data_slice);
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
