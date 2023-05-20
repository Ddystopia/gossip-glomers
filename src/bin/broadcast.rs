use rustengun::node::{Node, State};
use rustengun::*;

use std::collections::{HashMap, HashSet};
use std::io::{StdoutLock, Write};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
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
    fn with_initial_state(state: State<W>) -> Self {
        let not_known_to_neiborgs = state
            .node_ids
            .iter()
            .cloned()
            .map(|nid| (nid, HashSet::default()))
            .collect();

        BroadcastNode {
            state,
            messages: HashSet::default(),
            neighbors: Vec::default(),
            not_known_to_neiborgs,
        }
    }

    fn get_state(&mut self) -> &mut State<W> {
        &mut self.state
    }

    fn step(&mut self, payload: IPayload) -> anyhow::Result<()> {
        let payload = match payload {
            IPayload::GossipSignal => {
                for (neiborg, not_known) in &self.not_known_to_neiborgs {
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
                    self.messages.insert(message);
                    self.not_known_to_neiborgs
                        .get_mut(sender)
                        .map(|not_known| not_known.remove(&message));
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
                self.messages.insert(message);
                for neiborg in self.not_known_to_neiborgs.values_mut() {
                    neiborg.insert(message);
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
