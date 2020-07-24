/*
 * This defines a library extension for relying on restAPIs as opposed to sockets for communication.
 *
 * @namespace jiff_restAPI
 * @version 1.0
 */
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use std::sync::mpsc::{self, TryRecvError, Sender, Receiver};
// Import week days and WeekDay
use crate::client::util::constants;
use crate::ext::RiffClientTrait::*;
use crate::handlers::initialization;
use crate::mailbox::*;
use crate::RiffClient::*;
use clokwerk::Interval;
use clokwerk::Interval::*;
use primes;
use reqwest::Error;
use reqwest::Response;
use serde_json::json;
use serde_json::Value;
use std::time::Duration;
use std::{
    cmp,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

pub struct riffClientRest {
    //base_instance: RiffClient,
    //pub options: HashMap<String, JsonEnum>,
    pub hostname: String,
    pub computation_id: String,
    pub options: HashMap<String, JsonEnum>,
    __ready: bool,
    __initialized: bool,
    Zp: i64,
    pub id: i64,
    pub party_count: i64,
    pub sodium_: bool,
    keymap: Value,
    pub secret_key: Value,
    pub public_key: Value,
    crypto_provider: bool,
    messagesWaitingKeys: Value,
    listeners: Value,
    custom_messages_mailbox: Value,
    barriers: Value,
    wait_callbacks: Value,
    pub initialization_counter: i64,
    extensions: Vec<String>,
    protocols: Value,
    preprocessing_table: Value,
    preprocessingBatchSize: i64,
    pub preprocessing_function_map: HashMap<String, HashMap<String, JsonEnum>>,
    default_preprocessing_protocols: Value,
    currentPreprocessingTasks: Vec<i64>,
    preprocessingCallback: JsonEnum,
    logs: Vec<String>,
    shares: Value,
    counters: Value,
    op_id_seed: String,
    handler: Value,
    maxBatchSize: i64,
    mailbox: Mailbox,
    pollInterval: Option<Sender<String>>,
    flushInterval: Option<Sender<String>>,
}

impl riffClientRest {
    pub fn execute_listeners(riff: Arc<Mutex<riffClientRest>>, event: String, msg: String) {
        let event = event.as_str();
        match event {
            "error" => {}
            "initialization" => {}
            _ => {}
        }
    }
    #[tokio::main]
    async fn post(instance: Arc<Mutex<riffClientRest>>, body: Value) -> Result<(), reqwest::Error> {
        let mut riff = instance.lock().unwrap();
        riff.hostname.push_str("poll");
        let hostname = riff.hostname.as_str();
        let response = reqwest::Client::new()
            .post(hostname)
            .json(&body)
            .send()
            .await?;
        std::mem::drop(riff);
        riffClientRest::restReceive(instance.clone(),response);
        Ok(())
    }

    fn restFlush(riff: Arc<Mutex<riffClientRest>>) {
        let mut instance = riff.lock().unwrap();
        if instance.mailbox.pending != Value::Null {
            return;
        }

        // Construct request body
        let messages = instance.mailbox.current["messages"].clone();
        let sliced = &messages.as_array().unwrap()[0..instance.maxBatchSize as usize];
        let tail = &messages.as_array().unwrap()[instance.maxBatchSize as usize..];
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
        riffClientRest::post(riff, body);
    }

    fn restPoll(riff: Arc<Mutex<riffClientRest>>) {
        let mut instance = riff.lock().unwrap();
        if instance.mailbox.pending != Value::Null {
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
        riffClientRest::post(riff.clone(), body);
    }

    async fn restReceive(riff: Arc<Mutex<riffClientRest>>, response: Response) -> Result<(), reqwest::Error> {
        let mut instance = riff.lock().unwrap();
        if let Err(e) = &response.error_for_status_ref() {
            instance.mailbox.merge_requests();
            return Ok(());
        }

        let body: Value = response.json().await?;
        if !body["success"].as_bool().unwrap() {
            std::mem::drop(instance);
            riffClientRest::execute_listeners(
                riff.clone(),
                String::from("error"),
                json!({
                    "label": body["label"],
                    "error": body["error"],
                })
                .to_string(),
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
            std::mem::drop(instance);
            riffClientRest::execute_listeners(
                riff.clone(),
                String::from("initialization"),
                body["initialization"].clone().to_string(),
            );
            instance = riff.lock().unwrap();
        }

        for i in 0..body["messages"].as_array().unwrap().len() {
            let msg = body["messages"].as_array().unwrap()[i].clone();
            std::mem::drop(instance);
            riffClientRest::execute_listeners(riff.clone(),msg["label"].to_string(), msg["payload"].clone().to_string());
            instance = riff.lock().unwrap();
        }
        // if (jiff.socket.is_empty() && jiff.socket.empty_deferred != null) {
        //     jiff.socket.empty_deferred.resolve();
        //   }
        Ok(())
    }

    fn setup(instance: Arc<Mutex<riffClientRest>>, immediate: bool) {
        
        initialization::connected(instance.clone());
        if immediate != false {
            riffClientRest::restFlush(instance.clone());
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
                        riffClientRest::restPoll(instance_move.clone());
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
                        
                        riffClientRest::restFlush(instance.clone());
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

impl RiffClientTrait for riffClientRest {
    fn new(
        hostname: String,
        computation_id: String,
        mut options: HashMap<String, JsonEnum>,
    ) -> riffClientRest {
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
        let mut id_instance = 0;
        if let Option::Some(data) = options.get(&String::from("party_id")) {
            if let JsonEnum::Number(id) = data {
                id_instance = *id;
            }
        }

        /*
         * Total party count in the computation, parties will take ids between 1 to party_count (inclusive).
         * @type {number}
         */
        let mut party_count_instance = 0;
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
        let mut secretKey_instance = json!([]);
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
        let mut publicKey_instance = json!([]);
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

        riffClientRest {
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
                pending: json!({}),
            },
            pollInterval: None,
            flushInterval: None,
        }
    }

    fn connect(riff: Arc<Mutex<riffClientRest>>, immediate: bool) {
        let sodium;
        {
            let riff_instance = riff.lock().unwrap();
            sodium = riff_instance.sodium_;
        }
        
        if sodium == false {
            riffClientRest::setup(riff.clone(), immediate);
        } else {
            panic!("sodium library loading failed!")
        }
    }

    fn disconnect() {}

    fn is_empty(&mut self) -> bool {
        return self.mailbox.pending == Value::Null
            && self.mailbox.current["initialization"] == Value::Null
            && self.mailbox.current["messages"].as_array().unwrap().len() == 0
            && self.counters["pending_opens"].as_i64().unwrap() == 0;
    }

    fn emit(riff: Arc<Mutex<riffClientRest>>, label: String, msg: String) {
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
