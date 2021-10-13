/// VID / PID Pair
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId {
    vid: u16,
    pid: u16,
}

impl DeviceId {
    pub fn new(vid: u16, pid: u16) -> Self {
        DeviceId { vid, pid }
    }

    pub fn vid(&self) -> u16 {
        self.vid
    }

    pub fn pid(&self) -> u16 {
        self.pid
    }
}
