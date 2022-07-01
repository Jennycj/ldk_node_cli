use crate::node::disk;
use crate::node::hex_utils;
use crate::node::disk::FilesystemLogger;
use crate::node::bitcoind_client::Target;
// use crate::{
// 	HTLCStatus, InvoicePayer, MillisatAmount, NetworkGraph, NodeAlias, PaymentInfo,
// 	PaymentInfoStorage, PeerManager,
// };
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::Hash;
use bitcoin::network::constants::Network;
use bitcoin::secp256k1::PublicKey;
use lightning::chain;
use lightning::chain::keysinterface::{KeysInterface, KeysManager, Recipient};
use lightning::ln::msgs::NetAddress;
use lightning::ln::peer_handler::SimpleArcPeerManager;
use lightning::ln::{PaymentHash, PaymentPreimage, PaymentSecret};
use lightning::routing::gossip;
use lightning::routing::gossip::NodeId;
use lightning::routing::scoring::ProbabilisticScorer;
use lightning::util::config::{ChannelConfig, ChannelHandshakeLimits, UserConfig};
use lightning::util::events::EventHandler;
use lightning_invoice::payment;
use lightning_invoice::payment::PaymentError;
use lightning_invoice::utils::DefaultRouter;
use lightning_invoice::{utils, Currency, Invoice};
use lightning::ln::channelmanager::{ SimpleArcChannelManager};
use lightning_block_sync::rpc::RpcClient;
use lightning_rapid_gossip_sync::RapidGossipSync;
use std::env;
use std::fs;
use std::fmt;
use std::io;
use std::io::{BufRead, Write};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use lightning::chain::chainmonitor;
use lightning::chain::keysinterface::{InMemorySigner};
use lightning_persister::FilesystemPersister;
use lightning::chain::{Filter};
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use bitcoin::blockdata::transaction::Transaction;
use bitcoin::consensus::encode;
use bitcoin::hash_types::{Txid};
use lightning::util::logger::{Logger, Record};
use chrono::Utc;
use lightning_net_tokio::SocketDescriptor;

pub enum HTLCStatus {
	Pending,
	Succeeded,
	Failed,
}

pub struct MillisatAmount(pub Option<u64>);

impl fmt::Display for MillisatAmount {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.0 {
			Some(amt) => write!(f, "{}", amt),
			None => write!(f, "unknown"),
		}
	}
}

pub struct PaymentInfo {
	pub preimage: Option<PaymentPreimage>,
	pub secret: Option<PaymentSecret>,
	pub status: HTLCStatus,
	pub amt_msat: MillisatAmount,
}

pub type PaymentInfoStorage = Arc<Mutex<HashMap<PaymentHash, PaymentInfo>>>;

pub type PeerManager = SimpleArcPeerManager<
	SocketDescriptor,
	ChainMonitor,
	BitcoindClient,
	BitcoindClient,
	dyn chain::Access + Send + Sync,
	FilesystemLogger,
>;

pub type ChannelManager =
	SimpleArcChannelManager<ChainMonitor, BitcoindClient, BitcoindClient, FilesystemLogger>;

pub type InvoicePayer<E> = payment::InvoicePayer<
	Arc<ChannelManager>,
	Router,
	Arc<Mutex<ProbabilisticScorer<Arc<NetworkGraph>, Arc<FilesystemLogger>>>>,
	Arc<FilesystemLogger>,
	E,
>;

type Router = DefaultRouter<Arc<NetworkGraph>, Arc<FilesystemLogger>>;

type GossipSync<P, G, A, L> =
	lightning_background_processor::GossipSync<P, Arc<RapidGossipSync<G, L>>, G, A, L>;

pub type NetworkGraph = gossip::NetworkGraph<Arc<FilesystemLogger>>;
struct NodeAlias<'a>(&'a [u8; 32]);

impl fmt::Display for NodeAlias<'_> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let alias = self
			.0
			.iter()
			.map(|b| *b as char)
			.take_while(|c| *c != '\0')
			.filter(|c| c.is_ascii_graphic() || *c == ' ')
			.collect::<String>();
		write!(f, "{}", alias)
	}
}
pub struct BitcoindClient {
	bitcoind_rpc_client: Arc<RpcClient>,
	host: String,
	port: u16,
	rpc_user: String,
	rpc_password: String,
	fees: Arc<HashMap<Target, AtomicU32>>,
	handle: tokio::runtime::Handle,
}

impl BroadcasterInterface for BitcoindClient {
	fn broadcast_transaction(&self, tx: &Transaction) {
		let bitcoind_rpc_client = self.bitcoind_rpc_client.clone();
		let tx_serialized = serde_json::json!(encode::serialize_hex(tx));
		self.handle.spawn(async move {
			// This may error due to RL calling `broadcast_transaction` with the same transaction
			// multiple times, but the error is safe to ignore.
			match bitcoind_rpc_client
				.call_method::<Txid>("sendrawtransaction", &vec![tx_serialized])
				.await
			{
				Ok(_) => {}
				Err(e) => {
					let err_str = e.get_ref().unwrap().to_string();
					if !err_str.contains("Transaction already in block chain")
						&& !err_str.contains("Inputs missing or spent")
						&& !err_str.contains("bad-txns-inputs-missingorspent")
						&& !err_str.contains("txn-mempool-conflict")
						&& !err_str.contains("non-BIP68-final")
						&& !err_str.contains("insufficient fee, rejecting replacement ")
					{
						panic!("{}", e);
					}
				}
			}
		});
	}
}

impl FeeEstimator for BitcoindClient {
	fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
		match confirmation_target {
			ConfirmationTarget::Background => {
				self.fees.get(&Target::Background).unwrap().load(Ordering::Acquire)
			}
			ConfirmationTarget::Normal => {
				self.fees.get(&Target::Normal).unwrap().load(Ordering::Acquire)
			}
			ConfirmationTarget::HighPriority => {
				self.fees.get(&Target::HighPriority).unwrap().load(Ordering::Acquire)
			}
		}
	}
}

type ChainMonitor = chainmonitor::ChainMonitor<
	InMemorySigner,
	Arc<dyn Filter + Send + Sync>,
	Arc<BitcoindClient>,
	Arc<BitcoindClient>,
	Arc<FilesystemLogger>,
	Arc<FilesystemPersister>,
>;

pub struct LdkUserInfo {
	pub bitcoind_rpc_username: String,
	pub bitcoind_rpc_password: String,
	pub bitcoind_rpc_port: u16,
	pub bitcoind_rpc_host: String,
	pub ldk_storage_dir_path: String,
	pub ldk_peer_listening_port: u16,
	pub ldk_announced_listen_addr: Vec<NetAddress>,
	pub ldk_announced_node_name: [u8; 32],
	pub network: Network,
}