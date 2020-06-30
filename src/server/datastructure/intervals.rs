#[derive(PartialEq)]
pub struct interval {
    pub start: u64,
    pub end: u64,
}
#[derive(PartialEq)]
pub struct intervals {
    pub val: Vec<interval>,
}

impl intervals {
    pub fn create_free (&self) -> Option<u64> {
        return self.get_first_point()
     }

    pub fn get_first_point (&self) -> Option<u64> {
        if self.val.len() == 0 {
            return Option::None
        } 
        Some(self.val[0].start)
    }
}

// Create a new collection of intervals initially covering [ start, end ] inclusive.
// An interval is an array of interval objects, and represents the union of all them..
// The intervals in the array are guaranteed to be non-overlapping given proper initialization,
// and are sorted from the interval with the smallest left end point to the one with the largest one.
// intervals are inclusive of endpoints: [ start, end ].
pub fn intervals_fn(start: u64, end: u64) -> intervals {
    let mut val = Vec::new();
    val.push(interval{start:start, end:end});
    let intervals_instance = intervals {val: val};
    return intervals_instance
}