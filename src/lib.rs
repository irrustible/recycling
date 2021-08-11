#![no_std]
extern crate alloc;

use alloc::boxed::Box;

mod block;

mod recycling;
mod bulk;
pub use recycling::*;
pub use bulk::*;

// pub mod batch;
// use batch::Batch;

pub trait Pool<T> : Sized {
    fn alloc(&self, value: T) -> Box<T>;
    fn free(&self, item: Box<T>);
}

#[derive(Clone,Copy)]
pub struct GlobalAllocator;

pub const ALLOCATOR: GlobalAllocator = GlobalAllocator;

impl<T> Pool<T> for GlobalAllocator {
    #[inline(always)]
    fn alloc(&self, value: T) -> Box<T> { Box::new(value) }
    #[inline(always)]
    fn free(&self, _item: Box<T>) {}
}

// type SmolBatch<T, const BLOCK: usize> =
//     Batch<T, GlobalAllocator, BLOCK>;

// type HugeBatch<T, const BLOCK: usize> =
//     Batch<T, SmolRecycling<batch::Block<BLOCK>, BLOCK>, BLOCK>;

// pub struct SmolRecycling<T, const BLOCK: usize>(SmolBatch<T, BLOCK>);

// impl<T, const BLOCK: usize> Default for SmolRecycling<T, BLOCK> {
//     #[inline(always)]
//     fn default() -> Self { SmolRecycling(Batch::from(ALLOCATOR)) }
// }

// impl<T: 'static, const BLOCK: usize> Pool<T> for SmolRecycling<T, BLOCK> {
//     #[inline(always)]
//     fn alloc(&self, value: T) -> Box<T> { self.0.alloc(value) }
//     #[inline(always)]
//     fn free(&self, item: Box<T>) { self.0.free(item) }
// }

// pub struct HugeRecycling<T, const BLOCK: usize>(HugeBatch<T, BLOCK>);

// impl<T: 'static, const BLOCK: usize> Default for HugeRecycling<T, BLOCK> {
//     #[inline(always)]
//     fn default() -> Self { HugeRecycling(Batch::from(SmolRecycling::default())) }
// }

#[derive(Copy,Clone,Debug,Default,Eq,PartialEq)]
pub struct HitMiss {
    pub hit:  usize,
    pub miss: usize,
}

#[derive(Copy,Clone,Debug,Default,Eq,PartialEq)]
pub struct MinMaxNow {
    pub min: usize,
    pub max: usize,
    pub now: usize,
}

impl MinMaxNow {
    #[inline(always)]
    fn dec_now(&mut self) {
        self.now -= 1;
        self.min = usize::min(self.now, self.min);
    }

    #[inline(always)]
    fn inc_now(&mut self) {
        self.now += 1;
        self.max = usize::max(self.now, self.max);
    }
}
#[derive(Copy,Clone,Debug,Default,Eq,PartialEq)]
pub struct Metrics {
    pub alloc:  HitMiss,
    pub free:   HitMiss,
    pub blocks: MinMaxNow,
}

#[derive(Copy,Clone,Debug,Default,Eq,PartialEq)]
pub struct Stats {
    pub metrics: Metrics,
    pub len:     usize,
}

unsafe fn init<T>(ptr: *mut T, value: T) -> Box<T> {
    ptr.write(value);
    Box::from_raw(ptr)
}

