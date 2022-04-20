use serenity::model::id::GuildId;
use std::collections::HashSet;
// use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct GuildsJoinedCache {
    // guilds: HashMap<u64, AtomicBool>,
    guilds: HashSet<u64>,
}

#[allow(unused)]
impl GuildsJoinedCache {
    pub fn new() -> Self {
        GuildsJoinedCache {
            guilds: HashSet::new(),
        }
    }

    // pub fn get(&self, id: ChannelId) -> AtomicBool {
    // return
    // }

    pub fn insert(&mut self, id: GuildId) {
        // self.channels.insert(id.0);
        // self.guilds.insert(id.0, AtomicBool::new(value));
        self.guilds.insert(id.0);
    }

    // pub fn check_if_joined(&mut self, id: GuildId) -> Option<&AtomicBool> {
    //     // if self.channels.contains(&id.0) {
    //     if self.guilds.contains_key(&id.0) {
    //         return Some(&self.guilds[&id.0]);
    //     }
    //     None
    // }

    pub fn check_if_present(&mut self, id: GuildId) -> bool {
        self.guilds.contains(&id.0)
    }

    pub fn remove(&mut self, id: GuildId) {
        self.guilds.remove(&id.0);
    }

    // pub fn is_empty(&self) -> bool {
    //     self.channels.is_empty()
    // }
}
