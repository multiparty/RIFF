/*
 * This defines a library extension for relying on restAPIs as opposed to sockets for communication.
 *
 * @namespace jiff_restAPI
 * @version 1.0
 */
#![allow(non_snake_case)]
use crate::handlers::events;
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};
// Import week days and WeekDay
use crate::client::util::constants;
use crate::client::RiffClientTrait::*;
use crate::handlers::initialization;
use crate::mailbox::*;
use crate::RiffClient::*;

use primes;
use reqwest::Response;
use serde_json::json;
use serde_json::Value;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
};
use crate::SecretShare::SecretShare;
use crate::protocols::shamir;
pub struct RiffClientRest {
    client: reqwest::Client,
    //base_instance: RiffClient,
    //pub options: HashMap<String, JsonEnum>,
    pub hostname: String,
    pub computation_id: String,
    pub options: HashMap<String, JsonEnum>,
    pub __ready: bool,
    pub __initialized: bool,
    pub Zp: i64,
    pub id: Value,
    pub party_count: i64,
    pub sodium_: bool,
    pub keymap: Value,
    pub secret_key: Value,
    pub public_key: Value,
    pub crypto_provider: bool,
    pub messagesWaitingKeys: Value,
    pub listeners: Value,
    pub custom_messages_mailbox: Value,
    pub barriers: Value,
    pub wait_callbacks: Value,
    pub initialization_counter: i64,
    pub extensions: Vec<String>,
    pub protocols: Value,
    pub preprocessing_table: Value,
    pub preprocessingBatchSize: i64,
    pub preprocessing_function_map: HashMap<String, HashMap<String, JsonEnum>>,
    pub default_preprocessing_protocols: Value,
    pub currentPreprocessingTasks: Vec<i64>,
    pub preprocessingCallback: JsonEnum,
    pub logs: Vec<String>,
    pub shares: Value,
    pub counters: Value,
    pub op_id_seed: String,
    pub handler: Value,
    pub maxBatchSize: i64,
    pub mailbox: Mailbox,
    pub pollInterval: Option<Sender<String>>,
    pub flushInterval: Option<Sender<String>>,
    pub deferreds: Value,
    pub op_count: Value,
    //pub future_map: HashMap<String, HashMap<i64, Arc<fMutex<SecretShare>>>>,
    pub share_map: HashMap<String, HashMap<i64, i64>>,
    
}

impl RiffClientRest {
    pub fn execute_listeners(riff: Arc<Mutex<RiffClientRest>>, event: String, msg: Value) {
        let event = event.as_str();
        match event {
            "error" => {}
            "initialization" => initialization::initialized(riff.clone(), msg.clone()),
            "connect" => initialization::connected(riff.clone()),
            "public_keys" => events::handler_public_keys(riff.clone(), msg.clone()),
            "share" => events::handler_share(riff.clone(), msg.clone()),
            _ => {}
        }
    }
    #[tokio::main]
    async fn post(instance: Arc<Mutex<RiffClientRest>>, body: Value) -> Result<(), reqwest::Error> {
        //println!("post!");
        let mut riff = instance.lock().unwrap();
        if riff.hostname.ends_with("poll") {

        } else {
            riff.hostname.push_str("poll");
        }
        let hostname = riff.hostname.as_str();
        println!("client send: {:?}", body);
        let response = riff.client
            .post(hostname)
            .json(&body)
            .send()
            .await?;
        std::mem::drop(riff);
        //println!("postbeforereceive");
        RiffClientRest::restReceive(instance.clone(),response).await?;
        Ok(())
    }

    fn restFlush(riff: Arc<Mutex<RiffClientRest>>) {
        //println!("restFlush");
        let mut instance = riff.lock().unwrap();
        if instance.mailbox.pending != Value::Null {
            //println!("restFlush");
            return;
        }

        // Construct request body
        let messages = instance.mailbox.current["messages"].clone();
        let sliced: Vec<Value>;
        let tail: Vec<Value>;
        if let Value::Array(messagesArray) = messages {
            let slice_length;
            if instance.maxBatchSize as usize > messagesArray.len() {
                slice_length = messagesArray.len();
            } else {
                slice_length = instance.maxBatchSize as usize;
            }
            
            sliced = messagesArray[0..slice_length].to_vec();
            tail = messagesArray[slice_length..].to_vec();
        } else {
            sliced = Vec::new();
            tail = Vec::new();
        }
        // let sliced = sliced.clone();
        // let tail = tail.clone();
        let body = json!({
            "ack": instance.mailbox.current["ack"],
            "messages": json!(sliced),
            "initialization": instance.mailbox.current["initialization"],
            "computation_id": instance.computation_id,
            "from_id": instance.id,
        });

        // Mark mailbox with a pending request
        instance.mailbox.pending = body.clone();
        instance.mailbox.current = json!({
            "messages": json!(tail),
        });
        std::mem::drop(instance);
        RiffClientRest::post(riff, body);
    }

    fn restPoll(riff: Arc<Mutex<RiffClientRest>>) {
        //println!("restPoll!");
        let mut instance = riff.lock().unwrap();
        //println!("{:?}", instance.mailbox.pending);
        if instance.mailbox.pending != Value::Null  {
            //println!("restPoll!");
            return;
        }

        // Construct request body
        let body = json!({
            "ack": instance.mailbox.current["ack"],
            "messages": json!([]),
            "initialization": instance.mailbox.current["initialization"],
            "computation_id": instance.computation_id,
            "from_id": instance.id,
        });

        // Mark mailbox with a pending request
        instance.mailbox.pending = body.clone();
        instance.mailbox.current = json!({
            "messages": instance.mailbox.current["messages"],
        });

        //let body = body.to_string();
        std::mem::drop(instance);
        RiffClientRest::post(riff.clone(), body);
    }

    async fn restReceive(riff: Arc<Mutex<RiffClientRest>>, response: Response) -> Result<(), reqwest::Error> {
        //println!("restReceive");
        let mut instance = riff.lock().unwrap();
        if let Err(e) = &response.error_for_status_ref() {
            instance.mailbox.merge_requests();
            println!("{}",e);
            return Ok(());
        }

        let body: Value = response.json().await?;
        println!("Client received: {:?}", body);
        if !body["success"].as_bool().unwrap() {
            std::mem::drop(instance);
            RiffClientRest::execute_listeners(
                riff.clone(),
                String::from("error"),
                json!({
                    "label": body["label"],
                    "error": body["error"],
                })
                ,
            );
            return Ok(());
        }

        // No pending requests!
        instance.mailbox.pending = Value::Null;

        // handle ack, initialization, and remaining messages
        instance.mailbox
            .current
            .as_object_mut()
            .unwrap()
            .insert(String::from("ack"), body["ack"].clone());
        if body["initialization"] != Value::Null {
            //println!("before");
            std::mem::drop(instance);
            //println!("{}",body["initialization"].clone().as_str().unwrap());
            RiffClientRest::execute_listeners(
                riff.clone(),
                String::from("initialization"),
                body["initialization"].clone(),
            );
            instance = riff.lock().unwrap();
        }

        for i in 0..body["messages"].as_array().unwrap().len() {
            let msg = body["messages"].as_array().unwrap()[i].clone();
            std::mem::drop(instance);
            RiffClientRest::execute_listeners(riff.clone(),msg["label"].as_str().unwrap().to_string(), msg["payload"].clone());
            instance = riff.lock().unwrap();
        }
        // if (jiff.socket.is_empty() && jiff.socket.empty_deferred != null) {
        //     jiff.socket.empty_deferred.resolve();
        //   }
        Ok(())
    }

    fn setup(instance: Arc<Mutex<RiffClientRest>>, immediate: bool) {
        //println!("setup");
        initialization::connected(instance.clone());
        if immediate != false {
            RiffClientRest::restFlush(instance.clone());
        }
        
        let temp_instance = instance.clone();
        let mut unlocked = temp_instance.lock().unwrap();
        // Run poll and flush periodically.

        if let JsonEnum::Number(n) = unlocked.options.get(&String::from("pollInterval")).unwrap() {
            
            let n = *n;
            if n != 0 {
                std::mem::drop(unlocked);
                let (tx, rx) = mpsc::channel();
                let instance_move = Arc::clone(&instance);
                thread::spawn(move || 
                    
                    loop {
                        //println!("poll!");
                        RiffClientRest::restPoll(instance_move.clone());
                        thread::sleep(Duration::from_millis(n as u64));
                        match rx.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => {
                                println!("Terminating.");
                                break;
                            }
                            Err(TryRecvError::Empty) => {}
                        }
                    }
                 );
                unlocked = temp_instance.lock().unwrap();
                unlocked.pollInterval = Some(tx);
            } else {
                unlocked.pollInterval = Option::None;
            }
        }

        if let JsonEnum::Number(n) = unlocked.options.get(&String::from("flushInterval")).unwrap() {
            let n = *n;
            if n != 0 {
                let (tx, rx) = mpsc::channel();
                std::mem::drop(unlocked);
                thread::spawn(move || 
                    //let instance = Arc::clone(&instance);
                    loop {
                        //println!("flush");
                        RiffClientRest::restFlush(instance.clone());
                        thread::sleep(Duration::from_millis(n as u64));
                        match rx.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => {
                                println!("Terminating.");
                                break;
                            }
                            Err(TryRecvError::Empty) => {}
                        }
                    }
                 );
                unlocked = temp_instance.lock().unwrap();
                unlocked.flushInterval = Option::Some(tx);
            } else {
                unlocked.flushInterval = Option::None;
            }
        }
    }
}

impl RiffClientTrait for RiffClientRest {
    fn share(riff: Arc<Mutex<RiffClientRest>>, secret: i64, options: HashMap<String, JsonEnum>) -> Vec<SecretShare> {
        let mut instance = riff.lock().unwrap();
        if secret < 0 {
            panic!("secret must be a non-negative whole number");
        }
        let mut Zp_l: i64 = 0;
        if let Some(data) = options.get(&String::from("Zp")) {
            if let JsonEnum::Number(Zp) = data {
                Zp_l = *Zp;
            }
        } else {
            Zp_l = instance.Zp;
        }
        if secret >= Zp_l {
            panic!("secret must fit inside Zp");
        }
        std::mem::drop(instance);
        return shamir::riff_share(riff.clone(), secret, options);
        
    }

    fn new(
        hostname: String,
        computation_id: String,
        mut options: HashMap<String, JsonEnum>,
    ) -> RiffClientRest {
        //base instance initialization

        let hostname = hostname.trim();
        let mut hostname = hostname.to_string();
        if !hostname.ends_with("/") {
            hostname.push_str("/");
        }

        // Parse and verify options
        let t_mir = options.get_mut(&String::from("maxInitializationRetries"));
        match t_mir {
            Some(_) => (),
            None => {
                options.insert(
                    String::from("maxInitializationRetries"),
                    JsonEnum::Number(constants::maxInitializationRetries),
                );
            }
        }
        if let Option::Some(data) = options.get(&String::from("Zp")) {
            if let JsonEnum::Number(Zp) = data {
                if let Option::Some(data1) = options.get(&String::from("safemod")) {
                    if let JsonEnum::Bool(safemod) = data1 {
                        if !primes::is_prime(*Zp as u64) {
                            panic!("Zp = {} is not prime. Please use a prime number for the modulus or set safemod to false.", Zp);
                        }
                    }
                }
            }
        }
        /*
         * The default Zp for this instance.
         * @type {!number}
         */
        let mut Zp_instance = 0;
        if let Option::Some(data) = options.get(&String::from("Zp")) {
            if let JsonEnum::Number(Zp) = data {
                Zp_instance = *Zp;
            }
        } else {
            Zp_instance = constants::gZp;
        }

        /*
         * The id of this party.
         * @type {number}
         */
        let mut id_instance = Value::Null;
        if let Option::Some(data) = options.get(&String::from("party_id")) {
            if let JsonEnum::Number(id) = data {
                id_instance = json!(*id);
            }
        } 

        /*
         * Total party count in the computation, parties will take ids between 1 to party_count (inclusive).
         * @type {number}
         */
        let mut party_count_instance = 2;
        if let Option::Some(data) = options.get(&String::from("party_count")) {
            if let JsonEnum::Number(party_count) = data {
                party_count_instance = *party_count;
            }
        }

        /*
         * sodium wrappers either imported via require (if in nodejs) or from the bundle (in the browser).
         * This will be false if options.sodium is false.
         * @see {@link https://www.npmjs.com/package/libsodium-wrappers}
         * @type {?sodium}
         */
        let mut sodium_instance = false;
        if let Option::Some(data) = options.get(&String::from("sodium")) {
            if let JsonEnum::Bool(sodium) = data {
                sodium_instance = *sodium;
            }
        }

        /*
         * A map from party id to public key. Where key is the party id (number), and
         * value is the public key, which by default follows libsodium's specs (Uint8Array).
         * @see {@link https://download.libsodium.org/doc/public-key_cryptography/authenticated_encryption.html}
         * @type {!object}
         */
        let mut keymap_instance = json!({});
        if let Option::Some(data) = options.get(&String::from("public_keys")) {
            if let JsonEnum::Value(keymap) = data {
                keymap_instance = keymap.clone();
            }
        }

        /*
         * The secret key of this party, by default this follows libsodium's specs.
         * @see {@link https://download.libsodium.org/doc/public-key_cryptography/authenticated_encryption.html}
         * @type {?Uint8Array}
         */
        let mut secretKey_instance = Value::Null;
        if let Option::Some(data) = options.get(&String::from("secret_key")) {
            if let JsonEnum::Value(secret_key) = data {
                secretKey_instance = secret_key.clone();
            }
        }

        /*
         * The public key of this party, by default this follows libsodium's specs.
         * @see {@link https://download.libsodium.org/doc/public-key_cryptography/authenticated_encryption.html}
         * @type {?Uint8Array}
         */
        let mut publicKey_instance = Value::Null;
        if let Option::Some(data) = options.get(&String::from("public_key")) {
            if let JsonEnum::Value(public_key) = data {
                publicKey_instance = public_key.clone();
            }
        }

        /*
         * Flags whether to use the server as a fallback for objects that were not pre-processed properly
         * @type {!boolean}
         */
        let mut crypto_provider_instance = false;
        if let Option::Some(data) = options.get(&String::from("crypto_provider")) {
            if let JsonEnum::Bool(crypto_provider) = data {
                crypto_provider_instance = *crypto_provider;
            }
        }

        /*
         * Stores messages that are received with a signature prior to acquiring the public keys of the sender.
         * { 'party_id': [ { 'label': 'share/open/custom', <other attributes of the message> } ] }
         * @type {object}
         */

        /*
         * A map from tags to listeners (functions that take a sender_id and a string message).
         *
         * Stores listeners that are attached to this JIFF instance, listeners listen to custom messages sent by other parties.
         * @type {!object}
         */
        let mut listeners_instance = json!({});
        if let Option::Some(data) = options.get(&String::from("listeners")) {
            if let JsonEnum::Value(listeners) = data {
                listeners_instance = listeners.clone();
            }
        }

        let mut preprocessingBatchSize_instance = 10;
        if let Option::Some(data) = options.get(&String::from("preprocessingBatchSize")) {
            if let JsonEnum::Number(preprocessingBatchSize) = data {
                preprocessingBatchSize_instance = *preprocessingBatchSize;
            }
        }

        let mut protocols_instance = json!({});
        protocols_instance
            .as_object_mut()
            .unwrap()
            .insert(String::from("bits"), json!({}));

        // internal server computation instance is not rest nor sockets, ignore.
        if let Some(_) = options.get(&String::from("__internal_socket")) {
            panic!("restful extension failed");
        }

        // Default parameters
        if let None = options.get(&String::from("pollInterval")) {
            options.insert(String::from("pollInterval"), JsonEnum::Number(500));
        }

        if let None = options.get(&String::from("flushInterval")) {
            options.insert(String::from("flushInterval"), JsonEnum::Number(1000));
        }

        let mut maxBatchSize_instance = 0;
        match options.get(&String::from("maxBatchSize")) {
            Some(data) => {
                if let JsonEnum::Number(maxBatchSize) = data {
                    maxBatchSize_instance = *maxBatchSize;
                }
            }
            None => {
                maxBatchSize_instance = 150;
            }
        }

        // Stop the socket just in case it got connected somehow (if user forgot to disabled autoConnect)

        // Preprocessing here is trivial
        //to-do
        //preprocessing_function_map.insert(String::from("restAPI"), HashMap::new());

        // restAPI properties and functions

        //mailboxRestAPI

        // fn merge_requests (riffClientRest:&mut riffClientRest) {

        //     let mailbox =  &mut riffClientRest.mailbox;
        //     if let JsonEnum::Null = mailbox.get(&String::from("pending")).unwrap() {
        //         return
        //     }

        //     let mut temp_initiall = json!({});
        //     if let JsonEnum::Value(pending) = mailbox.get(&String::from("pending")).unwrap() {
        //         temp_initiall = pending["initialization"].clone();
        //     }
        //     if let JsonEnum::Value(current) = mailbox.get_mut(&String::from("current")).unwrap() {
        //             if let Value::Null = current["initialization"] {
        //                 current.as_object_mut().unwrap().insert(String::from("initialization"), temp_initiall);

        //             }
        //     }

        //     let mut temp_ack = json!({});
        //     if let JsonEnum::Value(pending) = mailbox.get(&String::from("pending")).unwrap() {
        //         temp_ack = pending["ack"].clone();
        //     }
        //     if let JsonEnum::Value(current) = mailbox.get_mut(&String::from("current")).unwrap() {
        //         if let Value::Null = current["ack"] {
        //             current.as_object_mut().unwrap().insert(String::from("ack"), temp_ack);

        //         }
        //     }

        //     let mut temp_cur_message = json!({});

        //     if let JsonEnum::Value(current) = mailbox.get(&String::from("current")).unwrap() {
        //         temp_cur_message = current["messages"].clone();
        //     }
        //     let mut temp_cur_message = temp_cur_message.as_array_mut().unwrap();

        //     let mut temp_pending_message = json!({});
        //     if let JsonEnum::Value(pennding) = mailbox.get(&String::from("pending")).unwrap() {
        //         temp_pending_message = pennding["messages"].clone();
        //     }

        //     temp_cur_message.append(temp_pending_message.as_array_mut().unwrap());

        //     if let JsonEnum::Value(current) = mailbox.get_mut(&String::from("current")).unwrap() {
        //         current.as_object_mut().unwrap().insert(String::from("messages"), json!(temp_cur_message));
        //     }

        //     mailbox.insert(String::from("pending"), JsonEnum::Null);

        // }

        RiffClientRest {
            maxBatchSize: maxBatchSize_instance,
            hostname: hostname,
            computation_id: computation_id,
            __ready: false,
            __initialized: false,
            options: options,
            Zp: Zp_instance,
            id: id_instance,
            party_count: party_count_instance,
            keymap: keymap_instance,
            sodium_: sodium_instance,
            secret_key: secretKey_instance,
            public_key: publicKey_instance,
            crypto_provider: crypto_provider_instance,
            messagesWaitingKeys: json!({}),
            listeners: listeners_instance,
            custom_messages_mailbox: json!({}),
            barriers: json!({}),
            wait_callbacks: json!([]),
            initialization_counter: 0,
            extensions: vec![String::from("base")],
            protocols: protocols_instance,
            preprocessing_table: json!({}),
            preprocessingBatchSize: preprocessingBatchSize_instance,
            preprocessing_function_map: HashMap::new(),
            default_preprocessing_protocols: json!({}),
            currentPreprocessingTasks: Vec::new(),
            preprocessingCallback: JsonEnum::Null,
            logs: Vec::new(),
            shares: json!({}),
            counters: json!({}),
            op_id_seed: String::from(""),
            handler: json!({}),
            mailbox: Mailbox {
                current: json!({}),
                pending: Value::Null,
            },
            pollInterval: None,
            flushInterval: None,
            client: reqwest::Client::new(),
            deferreds: json!({}),
            op_count: json!({}),
            //future_map: HashMap::new(),
            share_map: HashMap::new(),
        }
    }

    fn connect(riff: Arc<Mutex<RiffClientRest>>, immediate: bool) {
        let sodium;
        {
            let riff_instance = riff.lock().unwrap();
            sodium = riff_instance.sodium_;
        }
        
        if sodium == false {
            RiffClientRest::setup(riff.clone(), immediate);
        } else {
            RiffClientRest::setup(riff.clone(), immediate);
            //panic!("sodium library loading failed!")
        }
    }

    fn disconnect() {}

    fn is_empty(&mut self) -> bool {
        return self.mailbox.pending == Value::Null
            && self.mailbox.current["initialization"] == Value::Null
            && self.mailbox.current["messages"].as_array().unwrap().len() == 0
            && self.counters["pending_opens"].as_i64().unwrap() == 0;
    }

    fn emit(riff: Arc<Mutex<RiffClientRest>>, label: String, msg: String) {
        let msg: Value = serde_json::from_str(msg.as_str()).unwrap();
        let mut riff = riff.lock().unwrap();
        if label == String::from("initialization") {
            riff.mailbox
                .current
                .as_object_mut()
                .unwrap()
                .insert(String::from("initialization"), msg);
            return;
        }
        riff.mailbox.current["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "label": label,
                "payload": msg,
            }));
    }
}
