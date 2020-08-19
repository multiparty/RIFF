use crate::server::restfulAPI::*;
use serde_json::Value;
use crate::common::helper;
use serde_json::json;
// Possible fallback for when pre-processing elements are depleted, actual fallback is configurable by clients.
pub struct CryptoProviderHandlers {

}

impl CryptoProviderHandlers {

    // Default Crypto Handlers
    pub fn triplet (instance: &mut restfulAPI, computation_id: Value, receivers_list: Value, threshold: Value, Zp: Value, params: Value) -> Value {
        let a = helper::random(Zp.clone());
        let b = helper::random(Zp.clone());
        let c = (a * b) % (Zp.as_u64().unwrap());
        return json!({
            "secrets": json!([a, b, c]),
        })
    }

    pub fn quotient (instance: &mut restfulAPI, computation_id: Value, receivers_list: Value, threshold: Value, Zp: Value, params: Value) -> Value {
        let constant = params["constant"].clone();
        let noise = helper::random(Zp);
        let quotient = noise / constant.as_u64().unwrap();
        return json!({
            "secrets": json!([noise, quotient]),
        })
    }

    pub fn numbers (instance: &mut restfulAPI, computation_id: Value, receivers_list: Value, threshold: Value, Zp: Value, params: Value) -> Value {
        let count = params["count"].clone();
        let bit = params["bit"].clone();
        let mut min = params["min"].clone();
        let mut max = params["max"].clone();
        let mut number = params["number"].clone();
        let bitLength = params["bitLength"].clone();

        if min == Value::Null {
            min = json!(0);
        }
        if max == Value::Null {
            max = Zp;
        }
        if !bit.is_null() && bit.as_bool().unwrap() == true {
            max = json!(2);
        }

        let mut numbers = Vec::new();
        let mut c = 0;
        while c < count.as_u64().unwrap() {
            let mut n = number.clone();
            if number == Value::Null {
                n = json!(helper::random(json!(max.as_u64().unwrap() - min.as_u64().unwrap())) + min.as_u64().unwrap());
            }

            if bitLength == Value::Null {
                numbers.push(n);
            } else {
                numbers.append(&mut helper::number_to_bits(n.clone(), bitLength.clone()));
            }
            c = c + 1;
        }

        return json!({
            "secrets": json!(numbers),
        })
    }
}