use walle_core::prelude::{OneBot, PushToValueMap};
use walle_core::segment::{self, Segments};

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[segment]
pub struct Node {
    pub user_id: String,
    pub time: f64,
    pub user_name: String,
    pub message: Segments,
}

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[segment]
pub struct Face {
    pub id: Option<i32>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[segment]
pub struct Xml {
    pub service_id: i32,
    pub data: String,
}

#[derive(Debug, Clone, PushToValueMap, OneBot)]
#[segment]
pub struct Image {
    pub file_id: String,
    pub url: Option<String>,
    pub flash: Option<bool>,
}

#[derive(Debug, Clone, OneBot)]
#[segment]
pub enum WQSegment {
    Text(segment::Text),
    MentionAll {},
    Mention(segment::Mention),
    Reply(segment::Reply),
    Face(Face),
    Image(Image),
    Xml(Xml),
    Voice(segment::Voice), // todo ForWard
}

// impl Node {
//     #[async_recursion::async_recursion]
//     pub(crate) async fn to_forward_message(
//         self,
//         target: i64,
//         group: bool,
//         cli: &Client,
//         wqdb: &WQDatabase,
//     ) -> Result<ForwardMessage, RespError> {
//         Ok(match self.message {
//             MaybeNodes::Nodes(nodes) => {
//                 let mut fwd_nodes = Vec::new();
//                 for node in nodes {
//                     match node {
//                         NodeEnum::Node(n) => {
//                             let fwd_node = n.to_forward_message(target, group, cli, wqdb).await?;
//                             fwd_nodes.push(fwd_node);
//                         }
//                     }
//                 }
//                 ForwardNode {
//                     sender_id: self
//                         .user_id
//                         .parse()
//                         .map_err(|_| error::bad_param("user_id"))?,
//                     time: (self.time / 1000.0) as i32,
//                     sender_name: self.user_name,
//                     nodes: fwd_nodes,
//                 }
//                 .into()
//             }
//             MaybeNodes::Standards(message) => {
//                 let elements: MessageChain = match if group {
//                     MsgChainBuilder::group_chain_builder(cli, target, message)
//                         .build(wqdb)
//                         .await?
//                 } else {
//                     MsgChainBuilder::private_chain_builder(cli, target, message)
//                         .build(wqdb)
//                         .await?
//                 } {
//                     RQSendItem::Chain(chain) => chain,
//                     RQSendItem::Forward(_) => return Err(error::bad_param("node")),
//                     RQSendItem::Voice(_) => {
//                         let mut chain = MessageChain::default();
//                         chain.push(ricq::msg::elem::Text::new("[语音]".to_string()));
//                         chain
//                     }
//                 };
//                 MessageNode {
//                     sender_id: self
//                         .user_id
//                         .parse()
//                         .map_err(|_| error::bad_param("user_id"))?,
//                     time: (self.time / 1000.0) as i32,
//                     sender_name: self.user_name,
//                     elements,
//                 }
//                 .into()
//             }
//         })
//     }
// }
