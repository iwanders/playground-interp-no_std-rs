use crate::syscall;

struct File
{
    fd: u64,
}

impl File
{
    pub fn create(path: &str) -> Result<Self, &'static str>
    {
        let fd = unsafe{syscall::open(path, syscall::O_CREAT | syscall::O_RDWR, 0o0640)};
        if fd == 0
        {
            return Err("Something went wrong... :/ ");
        }
        Ok(File{fd})
    }

    pub fn open(path: &str, mode: &str) -> Result<Self, &'static str>
    {
        let mut mode_mask: i32 = 0;
        if mode.contains("w") &&  mode.contains("r")
        {
            mode_mask |= syscall::O_RDWR;
        }
        else if mode.contains("w")
        {
            mode_mask |= syscall::O_WRONLY;
        }
        else if mode.contains("r")
        {
            mode_mask |= syscall::O_RDONLY;
        }

        if mode.contains("a")
        {
            mode_mask |= syscall::O_APPEND;
        }
        else if !mode.contains("r")
        {
            mode_mask |= syscall::O_TRUNC;
            mode_mask |= syscall::O_CREAT;
        }
        
        
        let fd = unsafe{syscall::open(path, mode_mask, 0o0640)};
        if fd == 0
        {
            return Err("Something went wrong... :/ ");
        }
        Ok(File{fd})
    }

    pub fn write(&self, buffer: &[u8]) -> Result<(), &'static str>
    {
        unsafe {
            let buffer_p: *const u8 = &(buffer[0]) as *const u8;
            let write_res = syscall::write(self.fd, buffer_p, buffer.len() as u64);
            if write_res == buffer.len() as u64
            {
                return Ok(());
            }
        }
        Err("Didn't write?")
    }

    pub fn read_into(&self, buffer: &mut[u8]) -> Result<usize, &'static str>
    {
        // let mut tmp_buffer: [u8; 1024] = [0; 1024];
        let buffer_p: *mut u8 = &mut (buffer[0]) as *mut u8;
        let res = unsafe { syscall::read(self.fd, buffer_p, buffer.len() as u64) };
        if res != u64::MAX
        {
            Ok(res as usize)
        }
        else {
            Err("Something went wrong")
        }
    }

    pub fn size(&self) -> Result<usize, &'static str>
    {
        if let Some(v) = syscall::fstat_filesize(self.fd)
        {
            return Ok(v as usize);
        }
        Err("Something went wrong getting the size.")
    }
}
impl Drop for File
{
    fn drop(&mut self)
    {
        unsafe{syscall::close(self.fd)};
    }
}

// This has got to be the worst function in the history of well... Rust functions? :D
pub fn read_and_leak(path: &str) -> &'static [u8]
{
    // let binary_blob = fs::read("test/test").expect("Can't read binary");
    &[3, 8]
}


pub mod test {
    use super::*;
    pub fn test_all() {
        test_file_io();
    }

    pub fn test_file_io() {
        let test_string = "Hello there";
        // Write
        {
            let f = File::open("/tmp/test_fs_file", "w").expect("should work?");
            f.write(test_string.as_bytes()).expect("Should succeed.");
        }
        // Read
        {
            let mut tmp_buffer: [u8; 1024] = [0; 1024];
            let f = File::open("/tmp/test_fs_file", "r").expect("should work?");
            let size = f.size().expect("If we can open, we can get the size?");
            assert_eq!(size, test_string.len());
            let amount = f.read_into(&mut tmp_buffer).expect("Should succeed");
            let read_back = core::str::from_utf8(&tmp_buffer[..amount]).expect("valid ascii.");
            assert_eq!(read_back, test_string);
        }
    }
}
