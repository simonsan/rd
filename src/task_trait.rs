use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct RefTaskTrait<'a>(pub &'a dyn TaskTrait);

impl<'a> PartialEq for RefTaskTrait<'a> {
    fn eq(&self, other: &Self) -> bool {
        // If the addresses of the &dyn TaskTrait ptrs are same then they are the same task.
        self.0 as *const _ as *const u8 as usize == other.0 as *const _ as *const u8 as usize
    }
}

impl<'a> Eq for RefTaskTrait<'a> {}

impl<'a> Hash for RefTaskTrait<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let addr = self.0 as *const _ as *const u8 as usize;
        // The hash is the hash of the address of the task (&dyn TaskTrait).
        addr.hash(state);
    }
}

pub trait TaskTrait {}