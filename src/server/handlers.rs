use serde_json::Result as sResult;
use serde_json::Value;
use crate::server::restfulAPI::output_initial;
use crate::server::restfulAPI::restfulAPI;
use crate::server::datastructure::intervals::*;
use crate::server::hooks::*;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};
#[derive(Serialize, Deserialize)]
pub struct Broad_Cast_Message {
    pub public_keys: HashMap<u64, String>,
}
//party_id : &str, 
pub fn initializeParty (instance: Arc<Mutex<restfulAPI>>, computation_id : &str,party_id : &Value, party_count : &Value, msg : &Value, _s1 : bool) -> output_initial{
    
    //hooks staff

    // s1 is reserved for server use only!
    
    if _s1 != true && party_id == "s1" {
        return output_initial {
            success : false,
            error: Some("Party id s1 is reserved for server computation instances. This incident will be reported!".to_string()), 
            initialization: None, 
            party_id:None
        }
        
     } 
    // First: check that a valid party_count is defined internally or provided in the message for this computation
    let party_count_u64:u64;
    if *party_count == Value::Null {
        
        let instance_mg = instance.lock().unwrap();
        party_count_u64 = *instance_mg.computationMaps.maxCount.get(&computation_id.to_string()).unwrap();
    } else {
        party_count_u64 = party_count.as_u64().unwrap();
    }

    // Second: initialize intervals structure to keep track of spare/free party ids if uninitialized
    if instance.lock().unwrap().computationMaps.spareIds.get(computation_id) == None {
        let intervals = serverHooks::trackFreeIds(party_count_u64);

        instance.lock().unwrap().computationMaps.spareIds.entry(computation_id.to_string()).or_insert(intervals);
    }

    // Third: Valid parameters via hook

    // Fourth: Make sure party id is fine.
    // if party_id is given, try to reserve it if free.
    // if no party_id is given, generate a new free one.
    // let party_id_u64: u64;
    // if *party_id != Value::Null {
    //     if party_id != "s1" && !instance.lock().unwrap().computationMaps.spareIds.get(computation_id).unwrap().is_free(party_id.as_u64().unwrap()) {
    //         // ID is not spare, but maybe it has disconnected and trying to reconnect? maybe a mistaken client? maybe malicious?
    //         // Cannot handle all possible applications logic, rely on hooks to allow developers to inject case-specific logic.
    //     }
    // } else { // generate spare party_id
    //     party_id_u64 = instance.lock().unwrap().computationMaps.spareIds.get(computation_id).unwrap().create_free().unwrap();
    // }
    let mut party_id_u64: u64 = 999; //999 means 's1'
    if *party_id == Value::Null {
        party_id_u64 = instance.lock().unwrap().computationMaps.spareIds.get(computation_id).unwrap().create_free().unwrap();
    } else if party_id != "s1"{
        party_id_u64 = party_id.as_u64().unwrap();
    }

    // All is good: begin initialization
    // reserve id
    if party_id != "s1" {
        instance.lock().unwrap().computationMaps.spareIds.get_mut(computation_id).unwrap().reserve(party_id_u64);
    }

    // make sure the computation meta-info objects are defined for this computation id
    //let intance = instance.lock().unwrap();
    instance.lock().unwrap().initComputation(computation_id, party_id_u64, party_count_u64);

    // Finally: create return initialization message to the client

    return output_initial {
        success : false,
        error: Some("Party id s1 is reserved for server computation instances. This incident will be reported!".to_string()), 
        initialization: None, 
        party_id:None
    }
}

//store public key in given msg and return serialized public keys
pub fn storeAndSendPublicKey (instance: Arc<Mutex<restfulAPI>>, computation_id : &str, party_id : &Value, msg : &Value, party_id_u64: u64) -> HashMap<u64, String>{
    // store public key in key map
    let mut intance_temp = instance.lock().unwrap();
    let mut tmp = intance_temp.computationMaps.keys.get_mut(&computation_id.to_string()).unwrap();
    if tmp.get(&(999 as u64)) == None { // generate public and secret key for server if they don't exist
        let genkey = serverHooks::generateKeyPair(instance.clone());
        let secret_key = genkey.1.unwrap();
        instance.lock().unwrap().computationMaps.secretKeys.insert(computation_id.to_string(), secret_key.0.to_vec());
        let public_key = genkey.0.unwrap();
        tmp.insert(999, public_key.0.to_vec());
    }

    if party_id != "s1" {
        tmp.insert(party_id_u64, serverHooks::parseKey(instance.clone(), &msg["public_key"]).unwrap());
    }

    // Gather and format keys
    let mut keymap_to_send = HashMap::new();
    for (key, _) in tmp {
        if instance.lock().unwrap().computationMaps.keys.get(&computation_id.to_string()).unwrap().get(key) != None {
            let mut intance_temp = instance.lock().unwrap();
            let temp_map = intance_temp.computationMaps.keys.get(&computation_id.to_string()).unwrap();
            keymap_to_send.insert(key.clone(), serverHooks::dumpKey(instance.clone(), temp_map.get(key).unwrap()));
        }
    }
    let instance_bcm = Broad_Cast_Message {public_keys: keymap_to_send.clone()};
    let broadcast_message = serde_json::to_string(&instance_bcm).unwrap();

    // Send the public keys to all previously connected parties, except the party that caused this update
    let mut intance_temp = instance.lock().unwrap();
    let send_to_parties = intance_temp.computationMaps.clientIds.get(computation_id).unwrap();
    for receiver in send_to_parties {
        if receiver != party_id {

        }
    }
    return keymap_to_send
}