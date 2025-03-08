use crate::event::{Balance, GetBalancesResponse};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::Message;

pub enum GetBalancesError {
    WebsocketError(tungstenite::Error),
    ParseError(serde_json::Error),
}

impl Debug for GetBalancesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetBalancesError::WebsocketError(e) => write!(f, "WebsocketError({:?})", e),
            GetBalancesError::ParseError(e) => write!(f, "ParseError({:?})", e),
        }
    }
}

impl From<tungstenite::Error> for GetBalancesError {
    fn from(e: tungstenite::Error) -> Self {
        GetBalancesError::WebsocketError(e)
    }
}

impl From<serde_json::Error> for GetBalancesError {
    fn from(e: serde_json::Error) -> Self {
        GetBalancesError::ParseError(e)
    }
}

pub async fn get_balances(
    write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    read: &mut SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
) -> Result<HashMap<String, Balance>, GetBalancesError> {
    // this will return ALL the non-zero balances
    let get_balances = json!({
        "action": "privateGetBalance",
    });

    write.send(Message::Text(get_balances.to_string().into())).await?;

    // now we wait until the response arrives
    let msg = read.next().await.expect("privateGetBalance failed")?;

    let response = serde_json::from_str::<GetBalancesResponse>(msg.to_string().as_str())?;
    let balances = response
        .response
        .into_iter()
        .map(|balance| (balance.symbol.clone(), balance.clone()))
        .collect::<HashMap<_, _>>();

    log::debug!("got balances: {:?}", balances);

    Ok(balances)
}
