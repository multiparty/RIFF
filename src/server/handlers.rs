use serde_json::Result as sResult;
use serde_json::Value;
use crate::server::restfulAPI::output_initial;
use crate::server::restfulAPI::restfulAPI;

use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};

pub fn initializeParty (instance: Arc<Mutex<restfulAPI>>, computation_id : &str, party_id : &str, party_count : &Value, msg : &Value, _s1 : bool) -> output_initial{
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
    if *party_count == Value::Null {
        
        let instance_mg = instance.lock().unwrap();
        instance_mg.computationMaps.maxCount.get(computation_id).unwrap();
    }

    

    // Second: initialize intervals structure to keep track of spare/free party ids if uninitialized
    return output_initial {
        success : false,
        error: Some("Party id s1 is reserved for server computation instances. This incident will be reported!".to_string()), 
        initialization: None, 
        party_id:None
    }
}