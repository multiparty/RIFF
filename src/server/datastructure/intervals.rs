use math::round;
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

    

    // Searches for the point in the given intervals.
    // If the point is contained by the intervals, the
    // index of the exact interval containing it is
    // returned.
    // Otherwise, -1 is returned.
    fn find_point(&self, point: u64) -> i64 {
        if self.val.len() == 0 {
            return -1 // empty intervals
        }
        // special case optimization
        if self.interval_contains(0, point) {
            return 0
        }

        // binary search
        let mut st = 0;
        let mut nd = self.val.len();
        while st < nd {
            //let mid = round::floor((st + nd) / 2 as f64, 0);
            let mid = (st + nd) / 2 ;
            if self.interval_contains(mid, point) {
                return mid as i64
            } else if self.go_left(mid, point) {
                nd = mid;
            } else {
                st = mid + 1;
            }
        }

        return -1
    }

    fn interval_contains(&self, index: usize, point: u64) -> bool {
        self.val[index].start <= point && point <= self.val[index].end
    }

    

    fn go_left (&self, index: usize, point: u64) -> bool {
        return point < self.val[index].start
    }

    pub fn is_free (&self, point:u64) -> bool {
        return self.find_point(point) > - 1
    }

    pub fn reserve (&mut self, point: u64) -> bool {
        return self.remove_point(point)
    }

    // Remove a point from the collection of intervals
    // will splice the interval containing point into two intervals (or less) [ start, point-1 ], [ point+1, end ]
    // return true if the point was removed, false if the point was not contained in the intervals.
    fn remove_point (&mut self, point: u64) -> bool {
        if self.val.len() == 0 {
            return false // empty intervals
        }

        // Find the interval containing the point, then remove it.
        let index = self.find_point(point);
        if index == -1 {
            return false;
        }

        return self.remove_from_interval(index as usize, point)

    }

    // assumes that: intervals[index].start <= point <= intervals[index].end
    //   and index is in range.
    // removes the point from the interval at the given index.
    fn remove_from_interval(&mut self, index: usize, point: u64) -> bool {
        let current_interval = &self.val[index];
        let current_interval_start = current_interval.start;
        let current_interval_end = current_interval.end;
        if current_interval_start == current_interval_end {
            self.val.remove(index);
        } else if current_interval_start == point {
            self.val[index] = interval {start: current_interval_start + 1, end: current_interval_end};
        } else if current_interval_end == point {
            self.val[index] = interval {start: current_interval_start, end: current_interval_end - 1};
        } else {
            self.val[index] = interval {start: current_interval_start, end: point - 1};
            self.val.insert(index + 1, interval {start: point + 1, end: current_interval_end});
        }

        return true
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