use crate::meld::Meld;

#[derive(Debug, Clone, Default)]
pub struct Board {
    pub melds: Vec<Meld>,
}

impl Board {
    pub fn is_valid(&self) -> bool {
        self.melds.iter().all(|m| m.is_valid())
    }
}
