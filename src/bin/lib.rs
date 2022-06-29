// use crate::{
// 	ChannelManager, HTLCStatus, InvoicePayer, MillisatAmount, NetworkGraph, NodeAlias, PaymentInfo,
// 	PaymentInfoStorage, PeerManager,
// };
use bitcoin::secp256k1::PublicKey;
use lightning::ln::channelmanager::ChannelManager;
use lightning::ln::peer_handler::PeerManager;
use lightning::util::config::{ChannelConfig, ChannelHandshakeLimits, UserConfig};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{SocketAddr};
use std::sync::Arc;
use std::time::Duration;

#[derive(Deserialize,Debug)]
struct Resp {
  result: String,
  error: Option<String>,
  id: String,
}

#[tokio::main]
pub async fn generate_address() {
   
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

    println!("Your new bitcoin address is {:?}", resp_json.unwrap().result);
    
}

fn list_peers(peer_manager: Arc<PeerManager>) {
	println!("\t{{");
	for pubkey in peer_manager.get_peer_node_ids() {
		println!("\t\t pubkey: {}", pubkey);
	}
	println!("\t}},");
}

pub(crate) async fn connect_to_peer(pubkey: PublicKey, peer_addr: SocketAddr, peer_manager: Arc<PeerManager>,) -> Result<(), ()> {
    match lightning_net_tokio::connect_outbound(Arc::clone(&peer_manager), pubkey, peer_addr).await
	{
		Some(connection_closed_future) => {
			let mut connection_closed_future = Box::pin(connection_closed_future);
			loop {
				match futures::poll!(&mut connection_closed_future) {
					std::task::Poll::Ready(_) => {
						return Err(());
					}
					std::task::Poll::Pending => {}
				}
				// Avoid blocking the tokio context by sleeping a bit
				match peer_manager.get_peer_node_ids().iter().find(|id| **id == pubkey) {
					Some(_) => return Ok(()),
					None => tokio::time::sleep(Duration::from_millis(10)).await,
				}
			}
		}
		None => Err(()),
	}
}


pub fn open_channel(
	peer_pubkey: PublicKey, channel_amt_sat: u64, announced_channel: bool,
	channel_manager: Arc<ChannelManager>,
) -> Result<(), ()> {
	let config = UserConfig {
		peer_channel_config_limits: ChannelHandshakeLimits {
			// lnd's max to_self_delay is 2016, so we want to be compatible.
			their_to_self_delay: 2016,
			..Default::default()
		},
		channel_options: ChannelConfig { announced_channel, ..Default::default() },
		..Default::default()
	};

	match channel_manager.create_channel(peer_pubkey, channel_amt_sat, 0, 0, Some(config)) {
		Ok(_) => {
			println!("EVENT: initiated channel with peer {}. ", peer_pubkey);
			return Ok(());
		}
		Err(e) => {
			println!("ERROR: failed to open channel: {:?}", e);
			return Err(());
		}
	}
}

fn main() {
    // run()
}

