use super::hand::Hand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Settlement,
    City,
    Road,
    DevelopmentCard,
}

impl Item {
    /// Returns the resource cost of the item to purchase.
    pub fn cost(self) -> Hand {
        match self {
            Self::Road => Hand::from([1, 1, 0, 0, 0]),
            Self::Settlement => Hand::from([1, 1, 1, 1, 0]),
            Self::City => Hand::from([0, 0, 0, 2, 3]),
            Self::DevelopmentCard => Hand::from([0, 0, 1, 1, 1]),
        }
    }
}

impl TryFrom<&str> for Item {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "settlement" => Ok(Self::Settlement),
            "city" => Ok(Self::City),
            "road" => Ok(Self::Road),
            "development card" => Ok(Self::DevelopmentCard),
            _ => Err(()),
        }
    }
}
