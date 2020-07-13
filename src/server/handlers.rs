use serde_json::Result as sResult;
use serde_json::Value;

use crate::server::restfulAPI::restfulAPI;
use crate::server::datastructure::intervals::*;
use crate::server::hooks::*;
use sodiumoxide::crypto::box_::PublicKey;
use sodiumoxide::crypto::box_::SecretKey;
use serde::{Deserialize, Serialize};
use serde_json::json;


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
pub fn initializeParty (instance: Arc<Mutex<restfulAPI>>, computation_id : &Value,mut party_id :  &Value, mut party_count : &Value, msg : &Value, _s1 : bool) -> Value{
    //println!("in handler initializeParty");
    let party_id_shadow;
    let mut party_count_Value;
    {
        let mut unlocked_instance = instance.lock().unwrap();
    
    //hooks staff
    //jiffServer.hooks.log(jiffServer, 'initialize with ', computation_id, '-', party_id, ' #', party_count, ' : ', msg, '::', _s1);
    
    // s1 is reserved for server use only!
    
    if _s1 != true && party_id == "s1" {
        return json!({
            "success" : false,
            "error": "Party id s1 is reserved for server computation instances. This incident will be reported!",
        }) 
     } 
    // First: check that a valid party_count is defined internally or provided in the message for this computation
    //let party_count_u64:u64;
    // if party_count == Value::Null {
        
    //     let instance_mg = instance.lock().unwrap();
    //     party_count_u64 = *instance_mg.computationMaps.maxCount.get(&computation_id.to_string()).unwrap();
    // } else {
    //     party_count_u64 = party_count.as_u64().unwrap();
    // }
    ;
    if *party_count == Value::Null {
        //let instance_mg = instance.lock().unwrap();
        party_count_Value = unlocked_instance.computationMaps.maxCount[computation_id.to_string()].clone();
    } else {
        party_count_Value = party_count.clone();
    }

    // Second: initialize intervals structure to keep track of spare/free party ids if uninitialized
    
    //let mut unlocked_instance = instance.lock().unwrap();
    if unlocked_instance.computationMaps.spareIds.get(&computation_id.to_string()) == None {
        let intervals = serverHooks::trackFreeIds(party_count_Value.as_u64().unwrap());
        //instance.lock().unwrap().computationMaps.spareIds.insert(computation_id.to_string(), intervals);
        unlocked_instance.computationMaps.spareIds.entry(computation_id.to_string()).or_insert(intervals);
    }
    
    

    // Third: Valid parameters via hook
    let params = json!({
        "party_id": party_id,
        "party_count": party_count_Value,
    });
    serverHooks::beforeInitialization(unlocked_instance, computation_id.clone(), params);

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
    //let mut party_id_u64: u64 = 999; //999 means 's1'
    let mut unlocked_instance = instance.lock().unwrap();
    if *party_id != Value::Null {
        if party_id != "s1" && !unlocked_instance.computationMaps.spareIds.get(&computation_id.to_string()).unwrap().is_free(party_id.as_u64().unwrap()) {
            // ID is not spare, but maybe it has disconnected and trying to reconnect? maybe a mistaken client? maybe malicious?
            // Cannot handle all possible applications logic, rely on hooks to allow developers to inject case-specific logic.
            // try {
            //     party_id = jiffServer.hooks.onInitializeUsedId(jiffServer, computation_id, party_id, party_count, msg);
            //   } catch (err) {
            //     return { success: false, error: typeof(err) === 'string' ? err : err.message };
            //   }
        }
        party_id_shadow = party_id.clone();
        //party_id_u64 = instance.lock().unwrap().computationMaps.spareIds.get(computation_id).unwrap().create_free().unwrap();
    } else{ // generate spare party_id
        
        party_id_shadow = json!(unlocked_instance.computationMaps.spareIds.get(&computation_id.to_string()).unwrap().create_free().unwrap());
        //println!("{:?}", party_id_shadow);
    }

    // All is good: begin initialization
    // reserve id
    if party_id_shadow != "s1" {
        unlocked_instance.computationMaps.spareIds.get_mut(&computation_id.to_string()).unwrap().reserve(party_id_shadow.as_u64().unwrap());
    }

    // make sure the computation meta-info objects are defined for this computation id
    //let intance = instance.lock().unwrap();
    unlocked_instance.initComputation(&computation_id, &party_id_shadow, &party_count_Value);

    }
    
    
    // Finally: create return initialization message to the client
    let keymap_to_send = storeAndSendPublicKey(instance.clone(), &computation_id, &party_id_shadow, &msg);
    //println!("after store");
    let message = json!({
        "party_id": party_id_shadow,
        "party_count": party_count_Value,
        "public_keys": keymap_to_send,
    });
    //message = jiffServer.hooks.execute_array_hooks('afterInitialization', [jiffServer, computation_id, message], 2);
    return json!({
        "success": true,
        "message": message.to_string(), //it has to be wrapped to be a string, then being parsed in client.
    })
}

//store public key in given msg and return serialized public keys
pub fn storeAndSendPublicKey (instance: Arc<Mutex<restfulAPI>>, computation_id : &Value, party_id : &Value, msg : &Value) -> Value{
    //println!("in storeAndSendPublicKey");
    // store public key in key map
    let mut unlocked_instance = instance.lock().unwrap();
    let sodium = unlocked_instance.sodium;
    let mut tmp = unlocked_instance.computationMaps.keys[&computation_id.to_string()].clone();
    //println!("in storeAndSendPublicKey 1");
    if tmp["s1"] == Value::Null { // generate public and secret key for server if they don't exist
        let genkey = serverHooks::generateKeyPair(sodium);
        //println!("after generateKeyPair 1");
        let secret_key = genkey.1.unwrap();
        unlocked_instance.computationMaps.secretKeys.as_object_mut().unwrap().insert(computation_id.to_string(), json!(secret_key.0.to_vec()));
        let public_key = genkey.0.unwrap();
        tmp.as_object_mut().unwrap().insert(String::from("s1"), json!(public_key.0.to_vec()));
    }

    if party_id != "s1" {
        tmp.as_object_mut().unwrap().insert(party_id.to_string(), json!(serverHooks::parseKey(sodium, &msg["public_key"]).unwrap()));
        //println!("{:?}", tmp);
    }
    unlocked_instance.computationMaps.keys[computation_id.to_string()] = tmp.clone();  

    // Gather and format keys
    let mut keymap_to_send = json!({});
    for (key, _) in tmp.as_object_mut().unwrap() {
        if unlocked_instance.computationMaps.keys[computation_id.to_string()][key] != Value::Null {
            //println!("jinlai");
            //let mut intance_temp = instance.lock().unwrap();
            let dumped_Key =serverHooks::dumpKey(sodium, &unlocked_instance.computationMaps.keys[computation_id.to_string()][key]);
            
            keymap_to_send.as_object_mut().unwrap().insert(key.clone(), json!(dumped_Key));
        }
    }
    let instance_bcm = json!({
        "public_keys": keymap_to_send,
    });
    let broadcast_message = instance_bcm.to_string();

    // Send the public keys to all previously connected parties, except the party that caused this update
    //let mut intance_temp = instance.lock().unwrap();
    let mut send_to_parties = unlocked_instance.computationMaps.clientIds[computation_id.to_string()].clone();
    //println!("{:?}", send_to_parties);
    for receiver in send_to_parties.as_array_mut().unwrap() {
        if receiver != party_id {
            println!("sadsdadsadsa");
            unlocked_instance.safe_emit(String::from("public_keys"), broadcast_message.clone(), computation_id, receiver);
        }
    }
    //println!("{:?}", keymap_to_send);
    return keymap_to_send
}

pub fn share(instance: &mut restfulAPI, computation_id : Value, from_id : Value,mut msg : Value) -> Value {
    //jiffServer.hooks.log(jiffServer, 'share from', computation_id, '-', from_id, ' : ', msg);

    // try {
    //     msg = jiffServer.hooks.execute_array_hooks('beforeOperation', [jiffServer, 'share', computation_id, from_id, msg], 4);
    //   } catch (err) {
    //     return { success: false, error: typeof(err) === 'string' ? err : err.message };
    //   }

    let to_id = msg["party_id"].clone();
    msg["party_id"] = from_id.clone();

    //msg = jiffServer.hooks.execute_array_hooks('afterOperation', [jiffServer, 'share', computation_id, from_id, msg], 4);

    instance.safe_emit(String::from("share"), msg.to_string(), &computation_id, &to_id);
    return json!({
        "success": true,
    })
}

pub fn open(instance: &mut restfulAPI, computation_id : Value, from_id : Value,mut msg : Value) -> Value {
    
    let to_id = msg["party_id"].clone();
    msg["party_id"] = from_id.clone();

    instance.safe_emit(String::from("open"), msg.to_string(), &computation_id, &to_id);
    return json!({
        "success": true,
    })
}

pub fn free(instance: &mut restfulAPI, computation_id : Value, party_id : Value,mut msg : Value) -> Value {
    // jiffServer.hooks.log(jiffServer, 'free', computation_id, '-', party_id);

    // try {
    //   jiffServer.hooks.execute_array_hooks('beforeFree', [jiffServer, computation_id, party_id, msg], -1);
    // } catch (err) {
    //   return { success: false, error: typeof(err) === 'string' ? err : err.message };
    // }

    instance.computationMaps.freeParties[computation_id.to_string()][party_id.to_string()] = json!(true);
    
    // free up all resources related to the computation
    if instance.computationMaps.freeParties[computation_id.to_string()].as_object().unwrap().len() == instance.computationMaps.maxCount[computation_id.to_string()] {
        instance.freeComputation(computation_id);
        //jiffServer.hooks.execute_array_hooks('afterFree', [jiffServer, computation_id, party_id, msg], -1);
    }

    return json!({
        "success": true,
    })

}