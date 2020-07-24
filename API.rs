use RIFF::RIFFClient;
use RIFF::SecretShare;


async fn compute(client: &RIFFClient) {
    let input: u64 = 2784;
    let shares: Hashmap<party_id, SecretShare> = RIFF_Instance.share(input); // returns future/promise

    let sum: SecretShare = shares[RIFF_Instance.id].sadd(shares["2"]).await(); // my party_id = 1
    println!("Sum = ", sum);
}

let RIFF_Instance = RIFFClient::new(options);
RIFF_Instance.connect(options); // I think we can define compute to take the type `fn(&RIFFClient)` as a parameter
