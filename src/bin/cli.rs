use reqwest;
use std::collections::HashMap;
use serde::{Deserialize};

#[derive(Deserialize,Debug)]
struct Resp {
  result: String,
  error: Option<String>,
  id: String,
}

#[tokio::main]
async fn main() {
   
    let port: u16 = 38333;
    let url = format!("http://127.0.0.1:{}/", port);
    let mut map = HashMap::new();
    map.insert("jsonrpc", "1.0");
    map.insert("id", "curltest");
    map.insert("method", "getnewaddress");
    
    let client = reqwest::Client::new();
    let res = client.post(url)
        .basic_auth("test", Some("test"))
        .json(&map)
        .send()
        .await;
    
    let resp_json = res.unwrap().json::<Resp>().await;

    println!("{:?}", resp_json.unwrap().result);
    
}
