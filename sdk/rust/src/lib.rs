//! Bluefelt SDK 0.1.1 – minimal helpers for WASM hooks.

#![no_std]

extern crate alloc;


/* ------------------------------------------------------------------------
   Allocator (tiny, no_std)
   -------------------------------------------------------------------- */
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/* ------------------------------------------------------------------------
   Raw host imports – isolated so names never clash with wrappers.
   -------------------------------------------------------------------- */
mod host_raw {
    #![allow(non_snake_case)]
    #[link(wasm_import_module = "host")]
    extern "C" {
        #[link_name = "emit"]
        pub fn host_emit(ptr: *const u8, len: u32);

        #[link_name = "zone_len"]
        pub fn host_zone_len(ptr: *const u8, len: u32) -> u32;

        #[link_name = "owner_of"]
        pub fn host_owner_of(ptr: *const u8, len: u32) -> u32; // returns *const i8 (CStr) or 0

        #[link_name = "get_grid"]
        pub fn host_get_grid(zone_ptr: *const u8, zone_len: u32, out_ptr: *mut u32) -> u32;

        #[link_name = "advance_turn"]
        pub fn host_advance_turn();

        #[link_name = "round_end"]
        pub fn host_round_end(ptr: *const u8, len: u32);
    }
}

/* ------------------------------------------------------------------------
   Public safe wrappers
   -------------------------------------------------------------------- */
pub mod host {
    use super::{alloc::vec::Vec, host_raw, JsonValue};
    use alloc::string::String;

    /* ------------------ JSON helpers ------------------ */

    #[inline]
    pub fn read_json(ptr: u32, len: u32) -> JsonValue {
        let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
        serde_json::from_slice(slice).unwrap_or(JsonValue::Null)
    }

    /* ------------------ Emit diff ------------------ */

    pub fn emit(val: &JsonValue) {
        let s = serde_json::to_string(val).unwrap();
        unsafe { host_raw::host_emit(s.as_ptr(), s.len() as u32) }
    }

    /* ------------------ Zone helpers ------------------ */

    pub fn zone_len(zone_id: &str) -> u32 {
        unsafe { host_raw::host_zone_len(zone_id.as_ptr(), zone_id.len() as u32) }
    }

    /// Owner player id for an entity (if any).
    pub fn owner_of(ent_id: &str) -> Option<&'static str> {
        let ptr = unsafe { host_raw::host_owner_of(ent_id.as_ptr(), ent_id.len() as u32) };
        if ptr == 0 {
            None
        } else {
            unsafe {
                let c_str = core::ffi::CStr::from_ptr(ptr as *const i8);
                c_str.to_str().ok()
            }
        }
    }

    /* ------------------ Grid helper ------------------ */

    /// Deserialize a `shape: grid` zone into `Vec<Vec<Option<String>>>`.
    pub fn grid(zone_id: &str) -> Vec<Vec<Option<String>>> {
        let mut data_ptr: u32 = 0;
        let len = unsafe {
            host_raw::host_get_grid(zone_id.as_ptr(), zone_id.len() as u32, &mut data_ptr)
        };
        if len == 0 {
            return Vec::new();
        }
        let bytes = unsafe { core::slice::from_raw_parts(data_ptr as *const u8, len as usize) };
        serde_json::from_slice(bytes).unwrap()
    }

    /* ------------------ Control helpers ------------------ */

    pub fn advance_turn() {
        unsafe { host_raw::host_advance_turn() }
    }

    pub fn round_end(winner: &str) {
        unsafe { host_raw::host_round_end(winner.as_ptr(), winner.len() as u32) }
    }
}

/* ------------------------------------------------------------------------
   Re-export JsonValue so hooks can `use bluefelt_sdk::JsonValue`.
   -------------------------------------------------------------------- */
pub use serde_json::Value as JsonValue;
