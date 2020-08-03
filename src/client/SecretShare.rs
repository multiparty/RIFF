
use futures::Future;

pub enum NumberOrFuture {
     Number(i64),
     Future(Box<dyn Future<Output = i64>>),
 }
pub struct SecretShare {
    holders: Vec<i64>,
    threshold: i64,
    Zp: i64,
    value: i64,

}

impl SecretShare {
    pub fn new (value: i64, holders: Vec<i64>, threshold: i64, Zp: i64) -> SecretShare {
        // sort holders
        //jiff.helpers.sort_ids(holders);
        SecretShare {
            value: value,
            holders: holders,
            threshold: threshold,
            Zp: Zp,
        }
    }
}