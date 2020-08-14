use serde_json::Value;
use sodiumoxide;
use sodiumoxide::randombytes::*;

use serde_json::json;
// Secure randomness via rejection sampling.
pub fn random(max: Value) -> u64 {
    // Use rejection sampling to get random value within bounds
    // Generate random Uint8 values of 1 byte larger than the max parameter
    // Reject if random is larger than quotient * max (remainder would cause biased distribution), then try again

    // Values up to 2^53 should be supported, but log2(2^49) === log2(2^49+1), so we lack the precision to easily
    // determine how many bytes are required
    let max_number = max.as_u64().unwrap();
    if max_number > 562949953421312 {
        panic!("Max value should be smaller than or equal to 2^49");
    }

    let bitsNeeded = ((max_number as f64).ln() / 2_f64.ln()).ceil();
    let bytesNeeded = (bitsNeeded / 8.0).ceil() as u32;
    let maxValue = 256_u64.pow(bytesNeeded);

    // Keep trying until we find a random value within bounds
    //thread safety
    loop {
        let randomBytes = randombytes(bytesNeeded as usize);
        let mut randomValue = 0;

        let mut i = 0;
        while i < bytesNeeded {
            randomValue = randomValue * 256 + (randomBytes[i as usize] as u64);
            i = i + 1;
        }

        // randomValue should be smaller than largest multiple of max within maxBytes
        if randomValue < (maxValue - (maxValue % max_number)) {
            return randomValue % max_number;
        }
    }
}

// transform number to bit array
pub fn number_to_bits(number: Value, length: Value) -> Vec<Value> {
    let number = number.as_u64().unwrap();
    let number = format!("{:b}", number);
    let mut bits: Vec<Value> = Vec::new();
    let iterator = number.chars();
    for bit in iterator {
        // 1234 4321
        bits.insert(0, json!(bit.to_digit(2).unwrap()));
    }

    while length != Value::Null && bits.len() < length.as_u64().unwrap() as usize {
        bits.push(json!(0));
    }
    return bits;
}

// get the party number from the given party_id, the number is used to compute/open shares
pub fn get_party_number(party_id: Value) -> Value {
    if party_id.is_number() {
        return party_id;
    }
    if party_id.is_string() && party_id.as_str().unwrap().starts_with("s") {
        let temp: i64 = party_id.as_str().unwrap()[1..].parse().unwrap();
        return json!((-1) * temp);
    }
    return party_id.as_str().unwrap().parse().unwrap();
}

// actual mode
pub fn modF(x: Value, y: Value) -> i64 {
    let x = x.as_i64().unwrap();
    let y = y.as_i64().unwrap();
    if x < 0 {
        return (x % y) + y;
    }
    return x % y;
}
