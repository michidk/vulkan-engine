
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    pub fn major(self) -> u32 {
        self.0
    }

    pub fn minor(self) -> u32 {
        self.1
    }

    pub fn patch(self) -> u32 {
        self.2
    }
}

impl From<(u32, u32, u32)> for Version {
    fn from(vals: (u32, u32, u32)) -> Self {
        Self(vals.0, vals.1, vals.2)
    }
}

impl Into<(u32, u32, u32)> for Version {
    fn into(self) -> (u32, u32, u32) {
        (self.0, self.1, self.2)
    }
}

impl From<[u32; 3]> for Version {
    fn from(vals: [u32; 3]) -> Self {
        Self(vals[0], vals[1], vals[2])
    }
}

impl Into<[u32; 3]> for Version {
    fn into(self) -> [u32; 3] {
        [self.0, self.1, self.2]
    }
}
