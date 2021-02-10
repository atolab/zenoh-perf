//
// Copyright (c) 2017, 2020 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//
use async_std::sync::Arc;
use async_trait::async_trait;
use rand::RngCore;
use structopt::StructOpt;
use zenoh_protocol::core::{whatami, CongestionControl, PeerId, Reliability, ResKey};
use zenoh_protocol::io::RBuf;
use zenoh_protocol::link::Locator;
use zenoh_protocol::proto::ZenohMessage;
use zenoh_protocol::session::{
    DummySessionEventHandler, Session, SessionEventHandler, SessionHandler, SessionManager,
    SessionManagerConfig,
};
use zenoh_util::core::ZResult;

struct MySH {}

impl MySH {
    fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SessionHandler for MySH {
    async fn new_session(
        &self,
        _session: Session,
    ) -> ZResult<Arc<dyn SessionEventHandler + Send + Sync>> {
        Ok(Arc::new(DummySessionEventHandler::new()))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "s_pub_thr")]
struct Opt {
    #[structopt(short = "e", long = "peer")]
    peer: Locator,
    #[structopt(short = "m", long = "mode")]
    mode: String,
    #[structopt(short = "p", long = "payload")]
    payload: usize,
}

#[async_std::main]
async fn main() {
    // Enable logging
    env_logger::init();

    // Parse the args
    let opt = Opt::from_args();

    // Initialize the Peer Id
    let mut pid = [0u8; PeerId::MAX_SIZE];
    rand::thread_rng().fill_bytes(&mut pid);
    let pid = PeerId::new(1, pid);

    let whatami = match opt.mode.as_str() {
        "peer" => whatami::PEER,
        "client" => whatami::CLIENT,
        _ => panic!("Unsupported mode: {}", opt.mode),
    };

    let config = SessionManagerConfig {
        version: 0,
        whatami,
        id: pid,
        handler: Arc::new(MySH::new()),
    };
    let manager = SessionManager::new(config, None);

    // Connect to publisher
    let session = manager.open_session(&opt.peer).await.unwrap();

    // Send reliable messages
    let reliability = Reliability::Reliable;
    let congestion_control = CongestionControl::Block;
    let key = ResKey::RName("test".to_string());
    let info = None;
    let payload = RBuf::from(vec![0u8; opt.payload]);
    let reply_context = None;
    let routing_context = None;
    let attachment = None;

    let message = ZenohMessage::make_data(
        key,
        payload,
        reliability,
        congestion_control,
        info,
        routing_context,
        reply_context,
        attachment,
    );

    loop {
        let res = session.handle_message(message.clone()).await;
        if res.is_err() {
            break;
        }
    }
}