use std::fmt::Debug;

use num::{Num, ToPrimitive};
use num::integer::Average;
use serde::{Deserialize, Serialize};

pub mod party_client;
pub mod party_server;
pub mod dp_client;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum UpdateSearchRange {
    FoundK,
    SearchBelow,
    SearchAbove,
    Abort,
}

pub trait TypeTrait: Num + Clone + Average + ToPrimitive + Debug + Send + Sync + From<i32> {}

impl<T> TypeTrait for T where T: Num + Clone + Average + ToPrimitive + Debug + Send + Sync + From<i32> {}

