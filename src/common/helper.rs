use serde_json::Value;
use sodiumoxide::randombytes::*;
use sodiumoxide;

// Secure randomness via rejection sampling.
pub fn random (max: Value) -> u64 {
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
  let bytesNeeded = (bitsNeeded / 8.0 ).ceil() as u32;
  let maxValue = 256_u64.pow(bytesNeeded);
  
  // Keep trying until we find a random value within bounds
  sodiumoxide::init(); //thread safety
  loop {
      let randomBytes = randombytes(bytesNeeded as usize);
      let randomValue = 0;

      let i = 0;
      while i < bytesNeeded {
          randomValue = randomValue * 256 + (randomBytes[i as usize] as u64);
          i = i + 1;
      }

      // randomValue should be smaller than largest multiple of max within maxBytes
      if randomValue < (maxValue - (maxValue % max_number)) {
        return randomValue % max_number
      }
  }

}