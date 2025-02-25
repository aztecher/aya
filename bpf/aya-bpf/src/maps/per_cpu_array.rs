use core::{cell::UnsafeCell, marker::PhantomData, mem, ptr::NonNull};

use aya_bpf_cty::c_void;

use crate::{
    bindings::{bpf_map_def, bpf_map_type::BPF_MAP_TYPE_PERCPU_ARRAY},
    helpers::bpf_map_lookup_elem,
    maps::PinningType,
};

#[repr(transparent)]
pub struct PerCpuArray<T> {
    def: UnsafeCell<bpf_map_def>,
    _t: PhantomData<T>,
}

unsafe impl<T> Sync for PerCpuArray<T> {}

impl<T> PerCpuArray<T> {
    pub const fn with_max_entries(max_entries: u32, flags: u32) -> PerCpuArray<T> {
        PerCpuArray {
            def: UnsafeCell::new(bpf_map_def {
                type_: BPF_MAP_TYPE_PERCPU_ARRAY,
                key_size: mem::size_of::<u32>() as u32,
                value_size: mem::size_of::<T>() as u32,
                max_entries,
                map_flags: flags,
                id: 0,
                pinning: PinningType::None as u32,
            }),
            _t: PhantomData,
        }
    }

    pub const fn pinned(max_entries: u32, flags: u32) -> PerCpuArray<T> {
        PerCpuArray {
            def: UnsafeCell::new(bpf_map_def {
                type_: BPF_MAP_TYPE_PERCPU_ARRAY,
                key_size: mem::size_of::<u32>() as u32,
                value_size: mem::size_of::<T>() as u32,
                max_entries,
                map_flags: flags,
                id: 0,
                pinning: PinningType::ByName as u32,
            }),
            _t: PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self, index: u32) -> Option<&T> {
        unsafe {
            // FIXME: alignment
            self.lookup(index).map(|p| p.as_ref())
        }
    }

    #[inline(always)]
    pub fn get_ptr(&self, index: u32) -> Option<*const T> {
        unsafe { self.lookup(index).map(|p| p.as_ptr() as *const T) }
    }

    #[inline(always)]
    pub fn get_ptr_mut(&self, index: u32) -> Option<*mut T> {
        unsafe { self.lookup(index).map(|p| p.as_ptr()) }
    }

    #[inline(always)]
    unsafe fn lookup(&self, index: u32) -> Option<NonNull<T>> {
        let ptr = bpf_map_lookup_elem(
            self.def.get() as *mut _,
            &index as *const _ as *const c_void,
        );
        NonNull::new(ptr as *mut T)
    }
}
