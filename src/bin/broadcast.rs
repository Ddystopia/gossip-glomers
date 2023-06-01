use gossip_glomers::node::{Node, State};
use gossip_glomers::*;

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
        id: String,
        messages: HashSet<usize>,
        already_aware_nodes: HashSet<String>,
    },
    GossipOk {
        id: String,
    },
    Broadcast {
        message: usize,
    },
    Read,
    Topology {
        topology: HashMap<String, HashSet<String>>,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum OPayload {
    Gossip {
        id: String,
        messages: HashSet<usize>,
        already_aware_nodes: HashSet<String>,
    },
    GossipOk {
        id: String,
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
    neighbors: HashSet<String>,
    messages: HashSet<usize>,
    known_to_neiborgs: HashMap<String, HashSet<String>>, // message_id -> { node_id }
    messages_to_gossip: HashMap<String, HashSet<usize>>, // message_id -> { message }
    processed_messages: HashSet<String>,                 // { message_id }
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
            messages: Default::default(),
            neighbors: Default::default(),
            known_to_neiborgs: Default::default(),
            messages_to_gossip: Default::default(),
            processed_messages: Default::default(),
        }
    }

    fn get_state(&mut self) -> &mut State<W> {
        &mut self.state
    }

    fn step(&mut self, payload: IPayload) -> anyhow::Result<()> {
        let payload = match payload {
            IPayload::GossipSignal => {
                for (message_id, message) in self.messages_to_gossip {
                    let unaware_neibs = self
                        .neighbors
                        .difference(self.known_to_neiborgs.get(&message_id).unwrap());

                    for neib in unaware_neibs {
                        self.state.send_to(
                            neib.to_string(),
                            OPayload::Gossip {
                                id: message_id.clone(),
                                messages: message.clone(),
                                already_aware_nodes: self
                                    .known_to_neiborgs
                                    .keys()
                                    .cloned()
                                    .collect(),
                            },
                        )?;
                    }
                }
                None
            }

            IPayload::Gossip {
                id,
                messages,
                mut already_aware_nodes,
            } => {
                if !self.processed_messages.contains(&id) {
                    let sender = self.state.get_sender();
                    already_aware_nodes.insert(sender.to_string());
                    self.known_to_neiborgs
                        .entry(id)
                        .or_default()
                        .extend(already_aware_nodes);

                    if !self.messages_to_gossip.contains_key(&id) {
                        self.messages_to_gossip.insert(id.clone(), messages.clone());
                    }
                }

                Some(OPayload::GossipOk { id })
            }

            // known_to_neiborgs: HashMap<String, HashSet<String>>, // message_id -> { node_id }
            // messages_to_gossip: HashMap<String, HashSet<usize>>, // message_id -> { message }
            IPayload::GossipOk { id } => {
                let sender = self.state.get_sender();
                self.known_to_neiborgs
                    .get(&id)
                    .expect("If node sends me acknowledgment, then I've sent gossip to it.")
                    .insert(sender.to_string());

                let unaware_neibs = self
                    .neighbors
                    .difference(self.known_to_neiborgs.get(&id).unwrap())
                    .count();

                if unaware_neibs == 0 {
                    // if everyone from my topology is aware of the message, I dont need to store it anymore
                    self.messages_to_gossip.remove(&id);
                    self.known_to_neiborgs.remove(&id);
                    self.processed_messages.insert(id);
                }

                None
            }

            IPayload::Broadcast { message } => {
                if self.messages.insert(message) {
                    self.known_to_neiborgs.insert(message.to_string(), Default::default());
                    self.messages_to_gossip.insert(message.to_string())
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
