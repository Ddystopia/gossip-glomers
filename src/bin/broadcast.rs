use rustengun::node::{Node, State};
use rustengun::*;

use std::collections::{HashMap, HashSet};
use std::io::{StdoutLock, Write};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

const GOSSIP_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum IPayload {
    GossipSignal, // TODO: verify that is sent by the same node
    Gossip {
        messages: HashSet<usize>,
    },
    GossipOk {
        messages: HashSet<usize>,
    },
    Broadcast {
        message: usize,
    },
    Read,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum OPayload {
    Gossip {
        messages: HashSet<usize>,
    },
    GossipOk {
        messages: HashSet<usize>,
    },
    BroadcastOk,
    ReadOk {
        messages: HashSet<usize>,
        // messages: Vec<usize>,
    },
    TopologyOk,
}

struct BroadcastNode<W> {
    state: State<W>,
    pub neighbors: Vec<String>,
    pub messages: HashSet<usize>,
    pub not_known_to_neiborgs: HashMap<String, HashSet<usize>>,
}

impl<W> Node<IPayload, OPayload, W> for BroadcastNode<W>
where
    W: Write,
{
    fn with_initial_state(state: State<W>, tx: Sender<Message<IPayload>>) -> Self {
        let name = state.name.clone();

        thread::spawn(move || loop {
            thread::sleep(GOSSIP_INTERVAL);
            let r = tx.send(Message {
                src: name.clone(),
                dst: name.clone(),
                body: Body {
                    id: None,
                    in_reply_to: None,
                    payload: IPayload::GossipSignal,
                },
            });
            if r.is_err() {
                break;
            }
        });

        BroadcastNode {
            state,
            messages: HashSet::default(),
            neighbors: Vec::default(),
            not_known_to_neiborgs: HashMap::default(),
        }
    }

    fn get_state(&mut self) -> &mut State<W> {
        &mut self.state
    }

    fn step(&mut self, payload: IPayload) -> anyhow::Result<()> {
        let payload = match payload {
            IPayload::GossipSignal => {
                for (neiborg, not_known) in self
                    .not_known_to_neiborgs
                    .iter()
                    .filter(|(_, s)| !s.is_empty())
                {
                    self.state.send_to::<OPayload>(
                        neiborg.to_string(),
                        OPayload::Gossip {
                            messages: not_known.clone(),
                        },
                    )?;
                }
                None
            }

            IPayload::Gossip { messages } => {
                let sender = self.state.get_sender();
                for message in messages.clone() {
                    // if already known, it is already gossiped
                    if self.messages.insert(message) {
                        for not_known in self.not_known_to_neiborgs.values_mut() {
                            not_known.insert(message);
                        }
                    };
                    if let Some(not_known) = self.not_known_to_neiborgs.get_mut(sender) {
                        not_known.remove(&message);
                    }
                }

                //  TODO: maybe checksum or if of some sort
                Some(OPayload::GossipOk { messages })
            }
            IPayload::GossipOk { messages } => {
                let sender = self.state.get_sender();
                for message in messages {
                    self.not_known_to_neiborgs
                        .get_mut(sender)
                        .map(|not_known| not_known.remove(&message));
                }
                None
            }
            IPayload::Broadcast { message } => {
                if self.messages.insert(message) {
                    for neiborg in self.not_known_to_neiborgs.values_mut() {
                        neiborg.insert(message);
                    }
                }
                Some(OPayload::BroadcastOk)
            }
            IPayload::Read => Some(OPayload::ReadOk {
                messages: self.messages.clone(),
            }),
            IPayload::Topology { mut topology } => {
                if let Some(topology) = topology.remove(&self.state.name) {
                    self.neighbors = topology;
                }

                if self.not_known_to_neiborgs.is_empty() {
                    self.not_known_to_neiborgs = self
                        .neighbors
                        .iter()
                        .cloned()
                        .map(|nid| (nid, HashSet::default()))
                        .collect();
                } else {
                    unimplemented!()
                }

                Some(OPayload::TopologyOk)
            }
        };

        if let Some(payload) = payload {
            self.state.reply(payload)?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode<StdoutLock>, _, _>()
}
