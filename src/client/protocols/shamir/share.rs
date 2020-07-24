use crate::server::restfulAPI::*;
use serde_json::Value;
use serde_json::json;
use crate::common::helper;
/*
   * Default way of computing shares (can be overridden using hooks).
   * Compute the shares of the secret (as many shares as parties) using Shamir secret sharing
   * @ignore
   * @function jiff_compute_shares
   * @param {module:jiff-client~JIFFClient} jiff - the jiff instance
   * @param {number} secret - the secret to share.
   * @param {Array} parties_list - array of party ids to share with.
   * @param {number} threshold - the min number of parties needed to reconstruct the secret, defaults to all the receivers.
   * @param {number} Zp - the mod.
   * @returns {object} a map between party number and its share, this means that (party number, share) is a
   *          point from the polynomial.
   *
   */

pub fn jiff_compute_shares (riff: &mut restfulAPI, secret: Value, parties_list: Value, threshold: Value, Zp: Value) -> Value {
    let mut shares = json!({}); // Keeps the shares
    let mut i = 1;

    // Each player's random polynomial f must have
    // degree threshold - 1, so that threshold many points are needed
    // to interpolate/reconstruct.
    let t = (threshold.as_u64().unwrap() - 1) as usize;
    let mut polynomial = vec![Value::Null; t + 1];


    // Each players's random polynomial f must be constructed
    // such that f(0) = secret
    polynomial[0] = secret;

    // Compute the random polynomial f's coefficients
    while i <= t {
        polynomial[i] = json!(helper::random(Zp.clone()));
        i = i + 1;
    }
    //println!{"polynomial: {:?}", polynomial};

    // Compute each players share such that share[i] = f(i)
    for party in parties_list.as_array().unwrap() {
        let p_id = party.clone();
        //println!("party_id: {:?}", p_id);
        //println!("poly lens: {}", polynomial.len());
        shares.as_object_mut().unwrap().insert(p_id.clone().to_string(), polynomial[0].clone());
        let mut power = helper::get_party_number(p_id.clone());

        // let mut j = 1;
        // while j < polynomial.len() {
        //     let tmp = helper::modF(json!(polynomial[j].as_i64().unwrap() * power.as_i64().unwrap()), Zp.clone());
        //     let temp_share = shares[p_id.to_string()].as_i64().unwrap();
        //     shares.as_object_mut().unwrap().insert(p_id.clone().to_string(), json!(helper::modF(json!(temp_share + tmp), Zp.clone())));
        //     power = json!(helper::modF(json!(power.as_i64().unwrap() * helper::get_party_number(p_id.clone()).as_i64().unwrap()), Zp.clone()));
        //     println!("power: {:?}", power);
        //     j = j + 1;
        // }
        for j in 1..polynomial.len() {
            let tmp = helper::modF(json!(polynomial[j].as_i64().unwrap() * power.as_i64().unwrap()), Zp.clone());
            let temp_share = shares[p_id.to_string()].as_i64().unwrap();
            shares.as_object_mut().unwrap().insert(p_id.clone().to_string(), json!(helper::modF(json!(temp_share + tmp), Zp.clone())));
            power = json!(helper::modF(json!(power.as_i64().unwrap() * helper::get_party_number(p_id.clone()).as_i64().unwrap()), Zp.clone()));
            //println!("power: {:?}", power);

        }
    }
    return shares
}
