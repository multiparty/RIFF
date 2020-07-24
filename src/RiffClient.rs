/*
 * JIFF Client.
 *
 * Exposes the constructor for the {@link module:jiff-client~JIFFClient} class.
 *
 * In the browser, this adds `JIFFClient` as a global identifier.
 *
 * In the browser, this can be accessed via:
 * <pre><code>
 *   &lt;script src="jiff-client.js"&gt;&lt;/script&gt;
 *   &lt;script type="text/javascript"&gt;
 *     var jiffClientInstance = new JIFFClient(hostname, computationId, options);
 *   &lt;/script&gt;
 * </code></pre>
 *
 * In node.js, this can be accessed via:
 * <pre><code>
 *   const JIFFClient = require('jiffClient');
 *   const jiffClientInstance = new JIFFClient(hostname, computationId, options);
 *
 * </code></pre>
 *
 * @module jiff-client
 * @alias jiff-client
 */

// browserify bundles this into our code bundle
use sodiumoxide;

// utils and helpers
// var constants = require('./client/util/constants.js');
// var helpers = require('./client/util/helpers.js');
// var utils = require('./client/util/utils.js');
// var linkedList = require('./common/linkedlist.js');

// hooks
//var Hooks = require('./client/arch/hooks.js');

// extensions management
//var extensions = require('./client/arch/extensions.js');

// op ids and other counters
//var counters = require('./client/arch/counters.js');

// handlers for communication
//var handlers = require('./client/handlers.js');

// secret shares
//var SecretShareMetaClass = require('./client/share.js');
//var share_helpers = require('./client/shareHelpers.js');

// jiff client instance API
//var api = require('./client/api.js');

// preprocessing
//var preprocessingMap = require('./client/preprocessing/map.js');
//var preprocessingAPI = require('./client/preprocessing/api.js');
//var preprocessingDaemon = require('./client/preprocessing/daemon.js');
use std::{
    cmp,
    collections::HashMap,
    env,
    io::Error as IoError,
    sync::{Arc, Mutex},
    thread,
};

use crate::client::util::constants;
use primes;
use serde_json::json;
use serde_json::Value;
use crate::ext::riffClientRest;

type fn1 = fn(Arc<Mutex<riffClientRest>>);
#[derive(Clone)]
pub enum JsonEnum {
    func(fn1),
    String(String),
    Number(i64),
    Bool(bool),
    Value(Value),
    Null,
}
pub struct RiffClient {
    pub hostname: String,
    pub computation_id: String,
    pub options: HashMap<String, JsonEnum>,
    __ready: bool,
    __initialized: bool,
    Zp: i64,
    id: i64,
    party_count: i64,
    sodium_: bool,
    keymap: Value,
    secret_key: Value,
    public_key: Value,
    crypto_provider: bool,
    messagesWaitingKeys: Value,
    listeners: Value,
    custom_messages_mailbox: Value,
    barriers: Value,
    wait_callbacks: Value,
    initialization_counter: i64,
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
}

impl RiffClient {
   /*
     * Returns whether this instance is capable of starting the computation.
     * In other words, the public keys for all parties and servers are known,
     * and this party successfully initialized with the server.
     * @returns {!boolean}
     */
    pub fn isReady(&self) -> bool {
        return self.__ready;
    }

    /*
     * Returns whether this instance initialized successfully with the server.
     * Note that this can be true even when isReady() returns false, in case where some other parties have not
     * initialized yet!
     * @returns {!boolean}
     */
    pub fn isInitialized(&self) -> bool {
        return self.__initialized;
    }
}



    

