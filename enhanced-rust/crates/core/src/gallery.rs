#[derive(Debug, Default)]
pub struct Gallery {
    pub index: usize,
    pub paths: Vec<String>,
    pub loaded: u64,
    pub total_bytes: u64,
    pub error: Option<String>,
}
impl Gallery {
    pub fn select(&mut self, id: u8) -> bool {
        if !(1..=20).contains(&id) {return false;}
        self.index = usize::from(id - 1); true
    }
    pub fn install(&mut self, paths: Vec<String>) -> Option<String> {
        self.paths = paths; self.error = None;
        self.first()
    }
    pub fn first(&self) -> Option<String> {
        (self.loaded == self.total_bytes).then(|| self.paths.get(self.index).cloned()).flatten()
    }
    pub fn next(&mut self) -> Option<String> {
        // Enhanced behavior: fix original index-before-loading guard (3143432).
        if self.loaded != self.total_bytes || self.index + 1 >= self.paths.len() {return None;}
        self.index += 1; self.paths.get(self.index).cloned()
    }
    pub fn previous(&mut self) -> Option<String> {
        if self.index == 0 {return None;}
        self.index -= 1; self.paths.get(self.index).cloned()
    }
    pub fn counter(&self) -> String {format!("{} / {}", self.index+1,self.paths.len())}
}
