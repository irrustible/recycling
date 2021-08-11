use crate::*;
use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::drop_in_place;

use block::Block;

pub struct Recycling<T, const BLOCK: usize> {
    inner: UnsafeCell<Inner<BLOCK>>,
    _phantom: PhantomData<T>,
}

impl<T, const BLOCK: usize> Default for Recycling<T, BLOCK> {
    fn default() -> Self {
        Recycling {
            inner: UnsafeCell::new(Inner::default()),
            _phantom: PhantomData,
        }
    }
}

impl<T, const BLOCK: usize> fmt::Debug for Recycling<T, BLOCK> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let stats = self.stats();
        write!(fmt, "Recycling[{}]", stats.len)
    }
}

impl<T, const BLOCK: usize> Recycling<T, BLOCK> {
    #[inline(always)]
    pub fn stats(&self) -> Stats {
        let inner = unsafe { &*self.inner.get() };
        let metrics = inner.metrics;
        let block_len = inner.block.as_ref().map(|b| b.len()).unwrap_or(0);
        Stats { metrics, len: (BLOCK * metrics.blocks.now) + block_len }
    }

    pub fn boxed(&self, value: T) -> Box<T> {
        let inner = unsafe { &mut *self.inner.get() };
        if let Some(block)  = inner.block.as_mut() {
            if let Some(ptr) = block.pop() {
                inner.metrics.alloc.hit += 1;
                return unsafe { init(ptr.cast(), value) };
            }
            if let Some(mut next) = block.take_next() {
                let ptr = next.pop().unwrap();
                let _old = inner.block.replace(next).unwrap();
                inner.metrics.alloc.hit += 1;
                inner.metrics.blocks.dec_now();
                return unsafe { init(ptr.cast(), value) }
            }
            // falling through here will have the effect of leaving
            // the last block in place.
        }
        inner.metrics.alloc.miss += 1;
        Box::new(value)
    }
    pub fn free(&self, item: Box<T>) {
        let ptr = Box::leak(item) as *mut T;
        unsafe { drop_in_place(ptr); }
        let inner = unsafe { &mut *self.inner.get() };
        if let Some(block)  = inner.block.as_mut() {
            match block.push(ptr.cast()) {
                Ok(_) => inner.metrics.free.hit += 1,
                Err(ptr) => {
                    let mut block = Box::new(Block::default());
                    block.swap_next(&mut inner.block);
                    block.push(ptr).unwrap();
                    inner.block.replace(block);
                    inner.metrics.free.miss += 1;
                    inner.metrics.blocks.inc_now();
                }
            }
        } else {
            let mut block = Box::new(Block::default());
            block.push(ptr.cast()).unwrap();
            inner.block.replace(block);
            inner.metrics.free.miss += 1;
            inner.metrics.blocks.inc_now();
        }
    }
}

#[derive(Default)]
struct Inner<const LEN: usize> {
    block: Option<Box<Block<LEN>>>,
    metrics: Metrics,
}
