use crate::architecture::counters;
use crate::common::helper;
use crate::ext::RiffClientRest;
use crate::server::restfulAPI::*;
use crate::RiffClient::JsonEnum;
use crate::SecretShare::SecretShare;
use crate::{architecture::hook, RiffClientTrait::RiffClientTrait};
use serde_json::json;
use serde_json::Value;
use std::cmp;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};
use crate::util::helpers as uhelper;

impl SecretShare {

    pub fn cadd(&self, cst: i64) -> SecretShare {
        let sum = self.value + cst;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }
 
    pub fn csub(&self, cst: i64) -> SecretShare {
        let sum = self.value - cst;
        let afterMod = helper::modF(json!(sum), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }

    pub fn cmult(&self, cst: i64) -> SecretShare {
        let product = self.value * cst;
        let afterMod = helper::modF(json!(product), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }

    pub fn cdivfac(&self, cst: i64) -> SecretShare {
        let inv = uhelper::extended_gcd(cst, self.Zp).0;
        let product = self.value * inv;
        let afterMod = helper::modF(json!(product), json!(self.Zp));
        SecretShare {
            value: afterMod,
            holders: self.holders.clone(),
            threshold: self.threshold,
            Zp: self.Zp,
        }
    }








    pub fn sadd(&self, o: SecretShare) -> SecretShare {
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
            threshold: cmp::max(self.threshold, o.threshold),
            Zp: self.Zp,
        }
    }

    pub fn ssub(&self, o: SecretShare) -> SecretShare {
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
            threshold: cmp::max(self.threshold, o.threshold),
            Zp: self.Zp,
        }
    }
}
