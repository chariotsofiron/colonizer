use crate::models::resource::{Resource, N_RESOURCES};
use regex::Regex;
use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hand([u8; N_RESOURCES]);

impl Index<Resource> for Hand {
    type Output = u8;

    fn index(&self, index: Resource) -> &Self::Output {
        &self.0[usize::from(index)]
    }
}

impl IndexMut<Resource> for Hand {
    fn index_mut(&mut self, index: Resource) -> &mut Self::Output {
        &mut self.0[usize::from(index)]
    }
}

impl From<Resource> for Hand {
    fn from(value: Resource) -> Self {
        let mut result = Self::default();
        result[value] = 1;
        result
    }
}

impl IntoIterator for Hand {
    type Item = (Resource, u8);
    type IntoIter = std::array::IntoIter<Self::Item, N_RESOURCES>;

    fn into_iter(self) -> Self::IntoIter {
        [
            (Resource::Lumber, self[Resource::Lumber]),
            (Resource::Brick, self[Resource::Brick]),
            (Resource::Wool, self[Resource::Wool]),
            (Resource::Grain, self[Resource::Grain]),
            (Resource::Ore, self[Resource::Ore]),
        ]
        .into_iter()
    }
}

impl Hand {
    pub fn values(self) -> std::array::IntoIter<u8, N_RESOURCES> {
        self.0.into_iter()
    }
}

impl From<[u8; N_RESOURCES]> for Hand {
    fn from(value: [u8; N_RESOURCES]) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for Hand {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let re = Regex::new(r"(lumber|brick|wool|grain|ore)").unwrap();
        let mut result = Self::default();
        for capture in re.captures_iter(value) {
            let card = Resource::try_from(&capture[0])?;
            result[card] += 1;
        }
        Ok(result)
    }
}
