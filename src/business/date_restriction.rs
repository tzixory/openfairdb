pub enum DateRestriction {
    now,
    until(TimeStamp)
}

type TimeStamp = u64;
