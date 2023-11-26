pub const N_RESOURCES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Lumber,
    Brick,
    Wool,
    Grain,
    Ore,
}

impl TryFrom<&str> for Resource {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "lumber" => Ok(Self::Lumber),
            "brick" => Ok(Self::Brick),
            "wool" => Ok(Self::Wool),
            "grain" => Ok(Self::Grain),
            "ore" => Ok(Self::Ore),
            _ => Err(()),
        }
    }
}

impl From<usize> for Resource {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Lumber,
            1 => Self::Brick,
            2 => Self::Wool,
            3 => Self::Grain,
            4 => Self::Ore,
            _ => panic!("Invalid resource index"),
        }
    }
}

impl From<Resource> for usize {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Lumber => 0,
            Resource::Brick => 1,
            Resource::Wool => 2,
            Resource::Grain => 3,
            Resource::Ore => 4,
        }
    }
}
