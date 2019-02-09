// use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
  pub year: i32,
  pub month: i32,
  pub day: i32,
  pub hour: i32,
  pub min: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GuardEvents {
  pub id: i32,
  pub sleep_events: Vec<SleepEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SleepEvent {
  pub start_time: Timestamp,
  pub min: i32,
}

pub type Events = Vec<GuardEvents>;
