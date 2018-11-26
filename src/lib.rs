#![feature(alloc_system)]
extern crate alloc_system;

pub mod describe;
mod string_builder;

use std::{
    ffi::{c_void, CStr}, 
    ptr,
    fs::File,
    io::Read
};

#[allow(non_upper_case_globals)]
const kCFStringEncodingUTF8: u32 = 0x08000100;

#[repr(C)]
pub struct CFUUID {
    _private: [u8; 0],
}
#[repr(C)]
pub struct CFString {
    _private: [u8; 0],
}
#[repr(C)]
pub struct CFData {
    _private: [u8; 0],
}
#[repr(C)]
pub struct CFURL {
    _private: [u8; 0],
}

#[repr(C)]
pub struct QLPreviewRequest {
    _private: [u8; 0],
}
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "QuickLook", kind = "framework")]
#[link(name = "CoreServices", kind = "framework")]
extern "C" {    
    static kUTTypePlainText: *const CFString;
    fn CFEqual(a: *const CFUUID, b: *const CFUUID) -> bool;
    fn CFUUIDCreateFromString(alloc: *const c_void, uuidStr: *const CFString) -> *const CFUUID;
    fn CFStringCreateWithCString(alloc: *const c_void, c_str: *const u8, encoding: u32) -> *const CFString;
    fn CFRelease(o: *const c_void);
    fn CFPlugInAddInstanceForFactory(o: *const CFUUID);
    fn CFPlugInRemoveInstanceForFactory(o: *const CFUUID);
    fn CFUUIDCreateFromUUIDBytes(alloc: *const c_void, uuid: REFIID) -> *const CFUUID;
    fn CFURLGetFileSystemRepresentation(url: *const CFURL, resolveAgainstBase: bool, buffer: *const u8, maxBufLen: isize) -> bool;
    fn CFDataCreate(alloc: *const c_void, buffer: *const u8, len: isize) -> *const CFData;
    fn QLPreviewRequestSetDataRepresentation(preview: *const QLPreviewRequest, data: *const CFData, contentTypeUTI: *const CFString, properties: *const c_void);
}

#[repr(C)]
struct CGSize {
    width: f64,
    height: f64
}

#[repr(C)]
struct REFIID {
    byte0: u8,
    byte1: u8,
    byte2: u8,
    byte3: u8,
    byte4: u8,
    byte5: u8,
    byte6: u8,
    byte7: u8,
    byte8: u8,
    byte9: u8,
    byte10: u8,
    byte11: u8,
    byte12: u8,
    byte13: u8,
    byte14: u8,
    byte15: u8,
}

#[repr(C)]
struct QLGeneratorConduitItf {
    reserved: *const c_void,
    query_interface: unsafe extern fn(this: *mut QLGeneratorPlugin, iid: REFIID, ppv: *mut *mut QLGeneratorPlugin) -> u32,
    add_ref: unsafe extern fn(this: *mut QLGeneratorPlugin) -> u32,
    release: unsafe extern fn(this: *mut QLGeneratorPlugin) -> u32,
    generate_thumbnail_for_url: unsafe extern fn(this: *mut QLGeneratorPlugin, thumbnail: *const c_void, url: *const c_void, contentTypeUTI: *const c_void, options: *const c_void, maxSize: CGSize) -> i32,
    cancel_thumbnail_generation: unsafe extern fn(this: *mut QLGeneratorPlugin, thumbnail: *const c_void),
    generate_preview_for_url: unsafe extern fn(this: *mut QLGeneratorPlugin, preview: *const QLPreviewRequest, url: *const CFURL, contentTypeUTI: *const c_void, options: *const c_void) -> i32,
    cancel_preview_generation: unsafe extern fn(this: *mut QLGeneratorPlugin, preview: *const c_void),
}

#[repr(C)]
pub struct QLGeneratorPlugin {
    conduit_itf: *mut QLGeneratorConduitItf,
    factory_uuid: *const CFUUID,
    ref_count: u32,
}

extern "C" fn cancel_generation(_: *mut QLGeneratorPlugin, _: *const c_void) {
}
extern "C" fn generate_thumbnail_for_url(_: *mut QLGeneratorPlugin, _: *const c_void, _: *const c_void, _: *const c_void, _: *const c_void, _: CGSize) -> i32 {
    0
}
unsafe extern "C" fn generate_preview_for_url(_: *mut QLGeneratorPlugin, preview: *const QLPreviewRequest, url: *const CFURL, _: *const c_void, _: *const c_void) -> i32 {
    let path = [0; 1024];
    CFURLGetFileSystemRepresentation(url, false, path.as_ptr(), 1024);
    let class = CStr::from_ptr(path.as_ptr() as *const i8);
    let mut data = Vec::new();
    File::open(class.to_str().unwrap())
        .expect("file not found")
        .read_to_end(&mut data)
        .unwrap();
    let java = describe::describe(&data);
    let cfd = CFDataCreate(ptr::null(), java.as_ptr(), java.len() as isize);
    QLPreviewRequestSetDataRepresentation(preview, cfd, kUTTypePlainText, ptr::null());
    CFRelease(cfd as *const c_void);
    0
}


unsafe extern "C" fn query_interface(this: *mut QLGeneratorPlugin, iid: REFIID, ppv: *mut *mut QLGeneratorPlugin) -> u32 {
    let requested_uid = CFUUIDCreateFromUUIDBytes(ptr::null(), iid);
    let my_uuid_str = CFStringCreateWithCString(ptr::null(), "865AF5E0-6D30-4345-951B-D37105754F2D\0".as_ptr(), kCFStringEncodingUTF8);
    let my_uuid = CFUUIDCreateFromString(ptr::null(), my_uuid_str);
    let result = if CFEqual(my_uuid, requested_uid) {
        *ppv = this;
        ((*(*this).conduit_itf).add_ref)(this);
        (*(*this).conduit_itf).cancel_preview_generation = cancel_generation;
        (*(*this).conduit_itf).cancel_thumbnail_generation = cancel_generation;
        (*(*this).conduit_itf).generate_thumbnail_for_url = generate_thumbnail_for_url;
        (*(*this).conduit_itf).generate_preview_for_url = generate_preview_for_url;
        0
    } else {
        *ppv = ptr::null_mut();
        0x80000004
    };
    CFRelease(requested_uid as *const c_void);
    CFRelease(my_uuid_str as *const c_void);
    CFRelease(my_uuid as *const c_void);
    result
}
unsafe extern "C" fn add_ref(this: *mut QLGeneratorPlugin) -> u32 {
    (*this).ref_count += 1;
    (*this).ref_count
}

unsafe extern "C" fn release(this: *mut QLGeneratorPlugin) -> u32 {
    (*this).ref_count -= 1;
    if (*this).ref_count == 0 {
        let fid = (*this).factory_uuid;
        CFPlugInRemoveInstanceForFactory(fid);
        CFRelease(fid as *const c_void);
        Box::from_raw((*this).conduit_itf);
        Box::from_raw(this);
        0
    } else {
        (*this).ref_count
    }
}


#[no_mangle]
pub unsafe extern fn quick_look_generator_plugin_factory(_: *const c_void, type_id: *const CFUUID) -> *const QLGeneratorPlugin {
    let ql_uuid_str = CFStringCreateWithCString(ptr::null(), "5E2D9680-5022-40FA-B806-43349622E5B9\0".as_ptr(), kCFStringEncodingUTF8);
    let ql_uuid = CFUUIDCreateFromString(ptr::null(), ql_uuid_str);
    let result = if CFEqual(ql_uuid, type_id) {
        let factory_uuid_str = CFStringCreateWithCString(ptr::null(),
            "39336648-DACF-4E30-8C0F-BAA312BC22BB\0".as_ptr(), kCFStringEncodingUTF8);
        let factory_uuid = CFUUIDCreateFromString(ptr::null(), factory_uuid_str);
        let conduit_itf = Box::new(QLGeneratorConduitItf {
            query_interface, add_ref, release, generate_thumbnail_for_url, generate_preview_for_url,
            reserved: ptr::null(),
            cancel_preview_generation: cancel_generation,
            cancel_thumbnail_generation: cancel_generation
        });
        let this = Box::new(QLGeneratorPlugin {
            factory_uuid, 
            ref_count: 1,
            conduit_itf: Box::into_raw(conduit_itf),
        });
        CFPlugInAddInstanceForFactory(factory_uuid);
        CFRelease(factory_uuid_str as *const c_void);
        Box::into_raw(this)
    } else {
        ptr::null()
    };
    CFRelease(ql_uuid_str as *const c_void);
    CFRelease(ql_uuid as *const c_void);
    result
}