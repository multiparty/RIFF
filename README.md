# RIFF
This project aims to re-implement the functionality of the JIFF library for secure multiparty computation in the Rust programming language. JIFF is a Javascript library which aims to provide a developer-friendly framework for building and deploying applications which rely on multi-party computation. JIFF  is particularly suitable for web applications. The Rust implementation will provide a similar utility in the Rust language ecosystem, and may lend itself more to applications which do not require a user-facing web component. The resulting library will be fully interoperable with the existing javascript codebase, but will also be able to be run independently. Re-developing the library in Rust will allow for optimizations in the client-side computation and will be more suitable for performance critical applications.

# Api Usage 

## Client
1. create a new client instance
```rust
let mut options = HashMap::new();
options.insert(String::from("sodium"), JsonEnum::Bool(true));
options.insert(String::from("crypto_provider"), JsonEnum::Bool(true));
options.insert(String::from("party_count"), JsonEnum::Number(3));
let my_client = RiffClientRest::new(String::from("http://127.0.0.1:8080"), String::from("test1"), options);
```

2. wrap client instance in Mutex lock and call connect() api
```rust
let client_access = Arc::new(Mutex::new(my_client));
RiffClientRest::connect(client_access.clone(), true);
```

3. call share() api to generate array of shares
```rust
let mut options_share = HashMap::new();
let secret: i64 = args[1].parse().unwrap();
let shares: Vec<SecretShare> = RiffClientRest::share(client_access.clone(), secret, options_share);
```

4. do the computations.
```rust
let mut product = shares[1].clone();
let mut clinet_instance = client_access.lock().unwrap();
for i in 2..clinet_instance.party_count + 1 {
    std::mem::drop(clinet_instance);
    product = product.smult(shares[i as usize].clone(), None,client_access.clone());
    clinet_instance = client_access.lock().unwrap();
}
```

5. open the results.
```rust
let options_open = HashMap::new();
let result = RiffClientRest::open(client_access.clone(), product, options_open);
println!("result: {}", result.unwrap());
```

6. disconnect our client
```rust
RiffClientRest::disconnect(client_access.clone());
```

## Server
1. create a new server instance
```rust
let c_map = restfulAPI::computationMaps {
        clientIds: json!({}),
        maxCount: json!({}),
        keys: json!({}),
        secretKeys: json!({}),
        freeParties: json!({}),
        spareIds: HashMap::new(),
    };
let serverHooks = hooks::serverHooks {};
let maps = maps {
    tags: json!({}),
    pendingMessages: json!({}),
};
let restfulAPI_instance = restfulAPI::restfulAPI {
    mail_box: json!({}),
    computationMaps: c_map,
    hooks: serverHooks,
    maps: maps,
    sodium: true,
    log: true,
    cryptoMap: json!({}),
};
```

2. start listening
```rust
restfulAPI::restfulAPI::on(Arc::new(Mutex::new(restfulAPI_instance)));
```

# Test Suite
Run test suite for all test
```shell
cargo test --package riff --test test_suite -- test_suite --exact --nocapture
```

Add the test you would like to do in this array
```rust
let all_tests = vec![String::from("sadd"), String::from("smult")];
```

Config your test parameter
```rust
let mut config = HashMap::new();
config.insert(String::from("sadd"), HashMap::new());
config
    .get_mut(&String::from("sadd"))
    .unwrap()
    .insert(String::from("party_count"), String::from("3"));
config
    .get_mut(&String::from("sadd"))
    .unwrap()
    .insert(String::from("number_of_tests"), String::from("10"));
```





        
        
        
        

        

        
        
        

        
        
       


