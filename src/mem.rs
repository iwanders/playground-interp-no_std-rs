use crate::io::*;
use crate::println;

#[derive(Debug)]
/// Struct to hold a chunk of memory on the heap that's freed when struct is dropped.
pub struct Chunk {
    p: *mut u8,
    amount: usize,
}

impl Chunk {
    pub fn len(&self) -> usize {
        self.amount
    }
    pub fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.p, self.len()) }
    }
    pub fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.p, self.len()) }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // println!("Drop on {:?}", self.p);
        unsafe { free(self.p) }
    }
}

// This is a pretty bad allocator...
#[derive(Debug)]
#[repr(C)]
struct Record {
    index: usize,
    previous: *mut u8,
    in_use: bool,
    next: *mut u8,
}
const RECORD_SIZE: usize = core::mem::size_of::<Record>(); // 32 atm

unsafe fn record_at<'a>(current_position: *mut u8) -> &'a mut Record {
    let current_record =
        core::mem::transmute::<_, *mut Record>(current_position.offset(-(RECORD_SIZE as isize)));
    return &mut *current_record;
}

/// Raw memory allocation function, free must be called on this pointer.
/// Note, this is a rust function, without no_mangle, for internal use.
/// Also, really, don't use this anywhere besides this example.
pub unsafe fn malloc(amount: usize) -> *mut u8 {
    let current_position = crate::syscall::sbrk(0);

    // The previous record is just before that.
    let current_record = record_at(current_position);

    // Cool, now we can grow our memory by the requested amount and the next record size.
    let next_position = crate::syscall::sbrk((amount + RECORD_SIZE) as i64);

    // Instantiate the record there, and update the previous and new record.
    let next_record = record_at(next_position);
    next_record.index = current_record.index + 1;
    next_record.previous = current_position;
    current_record.next = next_position;
    current_record.in_use = true;

    // Return the current position, there's 'amount' memory ready for use there.
    current_position
}

/// Allocate a chunk of a certain size, the Chunk guarantees memory is freed when it goes out of
/// scope.
pub fn malloc_chunk(amount: usize) -> Chunk {
    let p = unsafe { malloc(amount) };
    Chunk { p, amount }
}

/// Struct to record some stats on the amount of memory allocated.
#[derive(Debug, Default)]
pub struct RecordStats {
    total: usize,
    in_use: usize,
    not_in_use: usize,
    top_index: usize,
}

/// Iterate through the chain of memory blocks and collect stats.
pub fn record_stats() -> RecordStats {
    let mut stats: RecordStats = Default::default();
    let mut current_position = crate::syscall::sbrk(0);
    loop {
        let current_record = unsafe { record_at(current_position) };
        if current_record.previous == 0 as *mut u8 {
            return stats;
        }
        stats.total += 1;
        if current_record.in_use {
            stats.in_use += 1;
        } else {
            stats.not_in_use = 1;
        }
        stats.top_index = core::cmp::max(stats.top_index, current_record.index);
        current_position = current_record.previous;
    }
}

/// Function to free memory at the end.
unsafe fn clean_freed_records() {
    let mut current_position = crate::syscall::sbrk(0);
    loop {
        // println!("in clean, position; {:?} ", current_position);
        let current_record = record_at(current_position);
        if current_record.in_use == false && current_record.index != 0 {
            // println!("Current not in use, freeing! {:?} ", current_record);
            let delta = (current_record.previous as i64) - (current_position as i64);
            // println!("delta {:?} ", delta);
            current_position = crate::syscall::sbrk(delta);
        } else {
            return;
        }
    }
}

/// Function to 'free' memory associated to the pointer. This may or may not actually shrink the
/// heap, if the freed pointer is not at the end, no release can happen.
unsafe fn free(v: *mut u8) {
    // Need to find the record that has this position, iterate from the rear, hopefully clean up
    // is soon...
    let mut current_position = crate::syscall::sbrk(0);
    loop {
        let current_record = record_at(current_position);
        if current_position == v {
            current_record.in_use = false;
            // println!("Marking {:?} as not in use", current_record);
            clean_freed_records();
            return; // we found it!
        }
        if current_record.index == 0 {
            break;
        }
        current_position = current_record.previous;
    }
    panic!("Well, we didn't find the thing, :/ Programmer error is to blame!");
}

/// Setup function to ensure we can allocate the first record.
pub fn setup() {
    // Allocate size for the first record.
    let current_position = crate::syscall::sbrk(RECORD_SIZE as i64);
    unsafe {
        let current_record = record_at(current_position);
        current_record.previous = 0 as *mut u8;
        current_record.next = 0 as *mut u8;
        current_record.in_use = false;
        current_record.index = 0;
    }
}

pub mod test {
    use super::*;
    pub fn test_all() {
        unsafe { test_malloc() };
        test_chunks();
    }
    unsafe fn test_malloc() {
        let start_position = crate::syscall::sbrk(0);
        let v0 = malloc(3);
        *v0 = 1;
        *(v0.offset(1)) = 2;
        *(v0.offset(2)) = 3;
        let v1 = malloc(3);
        let v2 = malloc(3);
        println!("record_stats: {:?}", record_stats());
        *v2 = 1;
        *(v2.offset(1)) = 2;
        *(v2.offset(2)) = 3;
        let v3 = malloc(3);
        free(v3);
        free(v1);
        println!("record_stats: {:?}", record_stats());
        free(v2);
        let end_position = crate::syscall::sbrk(0);
        println!("record_stats: {:?}", record_stats());
        assert_eq!(start_position, end_position);
    }
    unsafe fn _test_large_malloc() {
        let v0 = malloc(1000 * 1024 * 1024);
        *v0 = 1;
        for _i in 0..1000000 {
            let v1 = malloc(1024 * 1024 * 10);
            *v1 = 1;
            println!("v1: {:?}", v1);
        }
    }

    fn test_chunks() {
        println!("test_chunks");
        println!("record_stats: {:?}", record_stats());
        let _c0 = malloc_chunk(100);
        {
            let _c1 = malloc_chunk(100);
            let _c2 = malloc_chunk(100);
            println!("record_stats: {:?}", record_stats());
        }
        println!("record_stats: {:?}", record_stats());
    }
}
