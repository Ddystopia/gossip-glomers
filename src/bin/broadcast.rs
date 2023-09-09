use gossip_glomers::node::{Node, State};
use gossip_glomers::{main_loop, BodySerde, MessageSerde, NodeId};

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
        topology: HashMap<NodeId, Vec<NodeId>>,
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
    neighbors: Vec<NodeId>,
    messages: HashSet<usize>,
    not_known_to_neiborgs: HashMap<NodeId, HashSet<usize>>,
}

enum Res {
    None,
    Reply(gossip_glomers::AtomicResponce<OPayload>),
    Broadcast(Vec<gossip_glomers::AtomicResponce<OPayload>>),
}

impl Iterator for Res {
    type Item = gossip_glomers::AtomicResponce<OPayload>;
    fn next(&mut self) -> Option<Self::Item> {
        match std::mem::replace(self, Self::None) {
            Self::Reply(this) => Some(this),
            Self::Broadcast(mut v) => {
                let result = v.pop();
                *self = Self::Broadcast(v);
                result
            }

            Self::None => None,
        }
    }
}

impl<W> Node<IPayload, OPayload, W> for BroadcastNode<W>
where
    W: Write,
{
    type Response = Res;

    fn with_initial_state(state: State<W>, tx: Sender<MessageSerde<IPayload>>) -> Self {
        let name = state.node_id().clone();

        thread::spawn(move || loop {
            thread::sleep(GOSSIP_INTERVAL);
            let r = tx.send(MessageSerde {
                src: name.clone(),
                dst: name.clone(),
                body: BodySerde {
                    id: None,
                    in_reply_to: None,
                    payload: IPayload::GossipSignal,
                },
            });
            if r.is_err() {
                break;
            }
        });

        Self {
            state,
            messages: HashSet::default(),
            neighbors: Vec::default(),
            not_known_to_neiborgs: HashMap::default(),
        }
    }

    fn get_state(&self) -> &State<W> {
        &self.state
    }
    fn get_state_mut(&mut self) -> &mut State<W> {
        &mut self.state
    }

    fn step(&mut self, payload: IPayload) -> Self::Response {
        match payload {
            IPayload::GossipSignal => Res::Broadcast(
                self.not_known_to_neiborgs
                    .iter()
                    .filter(|(_, s)| !s.is_empty())
                    .map(|(neiborg, not_known)| {
                        gossip_glomers::AtomicResponce::new(
                            neiborg.clone(),
                            OPayload::Gossip {
                                messages: not_known.clone(),
                            },
                        )
                    })
                    .collect(),
            ),

            IPayload::Gossip { messages } => {
                let sender = self.state.get_sender();
                for message in messages.clone() {
                    // if already known, it is already gossiped
                    if self.messages.insert(message) {
                        for not_known in self.not_known_to_neiborgs.values_mut() {
                            not_known.insert(message);
                        }
                    };
                    if let Some(not_known) = self.not_known_to_neiborgs.get_mut(&sender) {
                        not_known.remove(&message);
                    }
                }

                //  TODO: maybe checksum or if of some sort
                Res::Reply(self.state.reply(OPayload::GossipOk { messages }))
            }

            IPayload::GossipOk { messages } => {
                let sender = self.state.get_sender();
                for message in messages {
                    self.not_known_to_neiborgs
                        .get_mut(&sender)
                        .map(|not_known| not_known.remove(&message));
                }
                Res::None
            }

            IPayload::Broadcast { message } => {
                if self.messages.insert(message) {
                    for neiborg in self.not_known_to_neiborgs.values_mut() {
                        neiborg.insert(message);
                    }
                }
                Res::Reply(self.state.reply(OPayload::BroadcastOk))
            }

            IPayload::Read => Res::Reply(self.state.reply(OPayload::ReadOk {
                messages: self.messages.clone(),
            })),

            IPayload::Topology { mut topology } => {
                if let Some(topology) = topology.remove(&self.state.node_id()) {
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

                Res::Reply(self.state.reply(OPayload::TopologyOk))
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<BroadcastNode<StdoutLock>, _, _>()
}
