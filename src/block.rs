use crate::*;
use core::fmt;
use alloc::boxed::Box;
use arrayvec::ArrayVec;

#[derive(Default)]
pub struct Block<const LEN: usize> {
    next: Option<Box<Block<LEN>>>,
    data: ArrayVec<*mut (), LEN>,
}

impl<const LEN: usize> Block<LEN> {

    #[inline(always)]
    pub fn len(&self) -> usize { self.data.len() }

    #[inline(always)]
    pub fn swap_next(&mut self, next: &mut Option<Box<Block<LEN>>>) {
        core::mem::swap(next, &mut self.next);
    }

    // #[inline(always)]
    // pub fn replace_next(&mut self, next: Box<Block<T, LEN>>) { self.next.replace(next) }

    #[inline(always)]
    pub fn take_next(&mut self) -> Option<Box<Block<LEN>>> { self.next.take() }

    #[inline(always)]
    pub fn push(&mut self, ptr: *mut ()) -> Result<(), *mut ()> {
        if !self.data.is_full() {
            self.data.push(ptr);
            Ok(())
        } else {
            Err(ptr)
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<*mut ()> { self.data.pop() }
}

impl<const LEN: usize> fmt::Debug for Block<LEN> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Block[{}/{}]", self.data.len(), LEN)
    }
}
