
use crate::server::restfulAPI::*;
use serde_json::Value;
use serde_json::json;
use crate::common::helper;
use std::{
    time::Duration,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};
use crate::ext::RiffClientRest;
use crate::RiffClient::JsonEnum;
use crate::architecture::counters;
use crate::{RiffClientTrait::RiffClientTrait, architecture::hook};
use crate::SecretShare::SecretShare;
use std::cmp;

impl SecretShare {
    pub fn sadd (&self, o: SecretShare) -> SecretShare {
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (+)");
        }
    
        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (+)");
        }
        
        let sum = self.value + o.value;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: cmp::max(self.threshold,o.threshold),
            Zp: self.Zp,
        }
    }

    pub fn ssub (&self, o: SecretShare) -> SecretShare {
        if !self.Zp == o.Zp {
            panic!("shares must belong to the same field (+)");
        }
    
        if !(self.holders == o.holders) {
            panic!("shares must be held by the same parties (+)");
        }
        
        let sum = self.value - o.value;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: cmp::max(self.threshold,o.threshold),
            Zp: self.Zp,
        }
    }
}
