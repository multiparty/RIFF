use crate::common::helper;
use serde_json::json;
//pub fn 
// pub fn egcd(b: i64, n: i64) -> (i64, i64, i64) {
//     // the multiplicative inverse of b mod n is the second value returned in the tuple
//     if b == 0 {
//         (n, 0, 1)
//     } else {
//         let (gcd, x, y) = egcd(modulo(n, b), b);
//         (gcd, y - (n/b) * x, x)
//     }
// }

/**
   * Extended Euclidean for finding inverses.
   * @method
   * @memberof helpers
   * @param {number} a - the number to find inverse for.
   * @param {number} b - the mod.
   * @return {number[]} [inverse of a mod b, coefficient for a, coefficient for b].
   */
pub fn extended_gcd (a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return (1, 0, a)
    }

    let temp = extended_gcd(b, helper::modF(json!(a), json!(b)));
    let x = temp.0;
    let y = temp.1;
    let d = temp.2;
    return (y, x - (a/b) * y, d)
}