use std::sync::mpsc::Sender;

use crate::*;

pub struct State<W> {
    pub name: String,
    pub node_ids: Vec<String>,
    req: MessageDTO,
    id_counter: usize,
    writer: W,
}

impl<W> State<W>
where
    W: Write,
{
    pub fn from(req: Message<InitPayload>, writer: W) -> State<W> {
        let (payload, req) = MessageDTO::from(req);
        let InitPayload::Init(payload) = payload;
        State {
            name: payload.node_id,
            node_ids: payload.node_ids,
            req,
            writer,
            id_counter: 0,
        }
    }

    pub fn convert<P>(self, req: Message<P>) -> (P, State<W>) {
        let (payload, req) = MessageDTO::from(req);
        let state = State {
            name: self.name,
            node_ids: self.node_ids,
            req,
            writer: self.writer,
            id_counter: self.id_counter,
        };
        (payload, state)
    }

    pub fn set_message<P>(&mut self, req: Message<P>) -> P {
        let (payload, req) = MessageDTO::from(req);
        self.req = req;
        payload
    }

    fn generate_message_id(&mut self) -> usize {
        self.id_counter += 1;
        self.id_counter
    }

    // Could reply with any payload.
    pub fn reply<OP>(&mut self, payload: OP) -> anyhow::Result<()>
    where
        OP: Serialize,
    {
        // If we need to send several messages, we cannot avoid copy.
        let msg = Message {
            src: self.name.clone(),
            dst: self.req.src.clone(),
            body: Body {
                id: Some(self.generate_message_id()),
                in_reply_to: self.req.body.id,
                payload,
            },
        };

        serde_json::to_writer(&mut self.writer, &msg).context("Serialize responce")?;
        self.writer.write_all(b"\n").context("Write newline")?;
        Ok(())
    }

    pub fn send_to<OP>(&mut self, dst: String, payload: OP) -> anyhow::Result<()>
    where
        OP: Serialize,
    {
        let msg = Message {
            src: self.name.clone(),
            dst,
            body: Body {
                id: Some(self.generate_message_id()),
                in_reply_to: None,
                payload,
            },
        };

        serde_json::to_writer(&mut self.writer, &msg).context("Serialize responce")?;
        self.writer.write_all(b"\n").context("Write newline")?;
        Ok(())
    }

    pub fn get_sender(&self) -> &str {
        &self.req.src
    }

}

struct MessageDTO {
    src: String,
    #[allow(dead_code)]
    dst: String,
    body: BodyDTO,
}

struct BodyDTO {
    id: Option<usize>,
    #[allow(dead_code)]
    in_reply_to: Option<usize>,
}

impl MessageDTO {
    fn from<P>(msg: Message<P>) -> (P, MessageDTO) {
        let (payload, body) = BodyDTO::from(msg.body);
        let message = MessageDTO {
            src: msg.src,
            dst: msg.dst,
            body,
        };
        (payload, message)
    }
}

impl BodyDTO {
    fn from<P>(body: Body<P>) -> (P, BodyDTO) {
        let payload = body.payload;
        let body = BodyDTO {
            id: body.id,
            in_reply_to: body.in_reply_to,
        };
        (payload, body)
    }
}

pub trait Node<IP, OP, W> {
    fn with_initial_state(state: State<W>, tx: Sender<Message<IP>>) -> Self
    where
        Self: Sized;

    fn step(&mut self, payload: IP) -> anyhow::Result<()>;

    fn get_state(&mut self) -> &mut State<W>;
}
