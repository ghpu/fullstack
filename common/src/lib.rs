use bimap::BiMap;
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::ops::Range;
use std::str::FromStr;
use std::vec::Vec;
use chrono::prelude::*;

#[derive(Serialize,Deserialize,Debug)]
pub struct Connection {
    pub time:DateTime<Utc>,
    pub login:String,
    pub channel:String
}

#[derive(Serialize,Deserialize,Debug)]
pub enum Message{
    Connection(Connection),
}
