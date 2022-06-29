#![allow(unused_variables, unused_assignments, dead_code)]
// use crate::disk;
// use crate::hex_utils;
// use crate::{
// 	ChannelManager, HTLCStatus, InvoicePayer, MillisatAmount, NetworkGraph, NodeAlias, PaymentInfo,
// 	PaymentInfoStorage, PeerManager,
// };
// use bitcoin::secp256k1::PublicKey;
// use reqwest;
// use std::collections::HashMap;
use std::env;
use generate_address;
// use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
// use std::sync::Arc;
use serde::{Deserialize};
// use std::time::Duration;

#[derive(Deserialize,Debug)]
struct Resp {
  result: String,
  error: Option<String>,
  id: String,
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let commands: Vec<&str> = vec![
		"generateaddress",
		"connectpeer",
		"listpeers",
		"openchannel",
		"listchannels",
		"getinvoice",
		"sendpayment",
		"listpayments",
		// "nodeinfo",
		// "closechannel",
    // "help",
		// "forceclosechannel",
		// "signmessage",
	];

  let input_command:&str = &args[2];
  if commands.contains(&input_command) {
    
  } else {
    println!("{} is not a valid command", &input_command);
  }

}





