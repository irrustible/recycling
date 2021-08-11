use crate::{*, recycling::*};
use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::drop_in_place;

use block::Block;

pub struct BulkRecycling<T, const BLOCK: usize> {
    inner: UnsafeCell<Inner<BLOCK>>,
    _phantom: PhantomData<T>
}

impl<T, const BLOCK: usize> Default for BulkRecycling<T, BLOCK> {
    fn default() -> Self {
        BulkRecycling {
            inner: UnsafeCell::new(Inner::default()),
            _phantom: PhantomData,
        }
    }
}

impl<T, const BLOCK: usize> fmt::Debug for BulkRecycling<T, BLOCK> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let stats = self.item_stats();
        write!(fmt, "BulkRecycling[{}]", stats.len)
    }
}

impl<T, const BLOCK: usize> BulkRecycling<T, BLOCK> {
    #[inline(always)]
    pub fn item_stats(&self) -> Stats {
        let inner = unsafe { &*self.inner.get() };
        let metrics = inner.metrics;
        let block_len = inner.items.as_ref().map(|b| b.len()).unwrap_or(0);
        Stats { metrics, len: (BLOCK * metrics.blocks.now) + block_len }
    }

    #[inline(always)]
    pub fn block_stats(&self) -> Stats { unsafe { &*self.inner.get() }.blocks.stats() }

    pub fn boxed(&self, value: T) -> Box<T> {
        let inner = unsafe { &mut *self.inner.get() };
        if let Some(items)  = inner.items.as_mut() {
            if let Some(ptr) = items.pop() {
                inner.metrics.alloc.hit += 1;
                return unsafe { init(ptr.cast(), value) };
            }
            if let Some(mut next) = items.take_next() {
                let ptr = next.pop().unwrap();
                let old = inner.items.replace(next).unwrap();
                inner.blocks.free(old);
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
        if let Some(items)  = inner.items.as_mut() {
            match items.push(ptr.cast()) {
                Ok(_) => inner.metrics.free.hit += 1,
                Err(ptr) => {
                    let mut items = inner.blocks.boxed(Block::default());
                    items.swap_next(&mut inner.items);
                    items.push(ptr).unwrap();
                    inner.items.replace(items);
                    inner.metrics.free.miss += 1;
                    inner.metrics.blocks.inc_now();
                }
            }
        } else {
            let mut items = inner.blocks.boxed(Block::default());
            items.push(ptr.cast()).unwrap();
            inner.items.replace(items);
            inner.metrics.free.miss += 1;
            inner.metrics.blocks.inc_now();
        }
    }
}

struct Inner<const LEN: usize> {
    items:   Option<Box<Block<LEN>>>,
    blocks:  Recycling<Block<LEN>, LEN>,
    metrics: Metrics,
}

impl<const LEN: usize> Default for Inner<LEN> {
    fn default() -> Self {
        Inner {
            items: None,
            blocks: Recycling::default(),
            metrics: Metrics::default(),
        }
    }
}
