use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::error::{is_fatal_server_error, Error, Result};
use crate::protocol::{WsEvent, WsRequest};

const WS_BASE_URL: &str = "wss://highrise.game/web/botapi";
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(15);
const READ_TIMEOUT: Duration = Duration::from_secs(45);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_SESSION_AGE: Duration = Duration::from_secs(1500); // 25m proactive refresh before Highrise 30m ghost timeout

type PendingMap = Arc<Mutex<HashMap<String, oneshot::Sender<Result<Value>>>>>;

/// Thread-safe handle for sending WebSocket requests and awaiting `rid` acks.
#[derive(Clone)]
pub struct Transport {
    outbound: mpsc::Sender<String>,
    pending: PendingMap,
}

impl Transport {
    /// Connects to Highrise, spawns the writer + keepalive tasks, and returns
    /// `(Transport, event_rx)` for the client loop to consume.
    pub async fn connect(
        token: &str,
        room_id: &str,
        events: &[&str],
    ) -> Result<(Self, mpsc::Receiver<Result<WsEvent>>)> {
        let url = format!("{WS_BASE_URL}?events={}", events.join(","));
        let mut req = url
            .into_client_request()
            .map_err(|e| Error::Config(e.to_string()))?;

        let headers = req.headers_mut();
        headers.insert(
            "api-token",
            token.parse().map_err(|_| Error::Config("invalid api-token header".into()))?,
        );
        headers.insert(
            "room-id",
            room_id.parse().map_err(|_| Error::Config("invalid room-id header".into()))?,
        );

        let (ws_stream, _) = time::timeout(CONNECT_TIMEOUT, connect_async(req))
            .await
            .map_err(|_| Error::Timeout)??;
        let (mut ws_sink, mut ws_source) = ws_stream.split();

        let (out_tx, mut out_rx) = mpsc::channel::<String>(128);
        let (evt_tx, evt_rx) = mpsc::channel::<Result<WsEvent>>(256);
        let pending: PendingMap = Arc::new(Mutex::new(HashMap::new()));

        let transport = Self {
            outbound: out_tx.clone(),
            pending: Arc::clone(&pending),
        };

        // Writer + 15s keepalive ticker in one task so WS sink never contends.
        tokio::spawn(async move {
            let keepalive_json = serde_json::to_string(&WsRequest::KeepaliveRequest {})
                .unwrap_or_else(|_| r#"{"_type":"KeepaliveRequest"}"#.to_string());
            let mut ticker = time::interval(KEEPALIVE_INTERVAL);
            ticker.tick().await; // skip immediate tick at t=0

            loop {
                tokio::select! {
                    maybe_msg = out_rx.recv() => {
                        let Some(text) = maybe_msg else { break };
                        if let Err(e) = ws_sink.send(Message::Text(text.into())).await {
                            warn!("websocket send failed: {e}");
                            break;
                        }
                    }
                    _ = ticker.tick() => {
                        if let Err(e) = ws_sink.send(Message::Text(keepalive_json.clone().into())).await {
                            warn!("keepalive send failed: {e}");
                            break;
                        }
                    }
                }
            }
            let _ = ws_sink.close().await;
        });

        // Reader task: resolves pending `rid` acks first, else emits `WsEvent`.
        let reader_pending = Arc::clone(&pending);
        tokio::spawn(async move {
            let session_deadline = time::Instant::now() + MAX_SESSION_AGE;
            loop {
                if time::Instant::now() >= session_deadline {
                    debug!("proactive 25m session refresh to prevent Highrise ghost session");
                    break;
                }
                let next_frame = match time::timeout(READ_TIMEOUT, ws_source.next()).await {
                    Ok(Some(frame)) => frame,
                    Ok(None) => break,
                    Err(_) => {
                        warn!("no websocket frames or keepalive responses for {READ_TIMEOUT:?}; closing zombie connection");
                        let _ = evt_tx.send(Err(Error::Timeout)).await;
                        break;
                    }
                };
                match next_frame {
                    Ok(Message::Text(text)) => {
                        let raw: Value = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(e) => {
                                warn!("dropping malformed json frame: {e}");
                                continue;
                            }
                        };

                        if let Some(rid) = raw.get("rid").and_then(|v| v.as_str()) {
                            if let Some(tx) = reader_pending.lock().await.remove(rid) {
                                let is_err = raw.get("_type").and_then(|v| v.as_str()) == Some("Error");
                                if is_err {
                                    let msg = raw
                                        .get("message")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown server error")
                                        .to_string();
                                    let _ = tx.send(Err(Error::Server(msg)));
                                } else {
                                    let _ = tx.send(Ok(raw));
                                }
                                continue;
                            }
                        }

                        match serde_json::from_value::<WsEvent>(raw.clone()) {
                            Ok(WsEvent::KeepaliveResponse {}) => {}
                            Ok(WsEvent::Error { message, .. }) => {
                                if is_fatal_server_error(&message) {
                                    let _ = evt_tx.send(Err(Error::Fatal(message))).await;
                                    break;
                                }
                                warn!("server error event: {message}");
                            }
                            Ok(evt) => {
                                if evt_tx.send(Ok(evt)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let kind = raw.get("_type").and_then(|v| v.as_str()).unwrap_or("?");
                                if kind == "SessionMetadata" {
                                    warn!("failed to decode SessionMetadata: {e}; raw={raw}");
                                } else {
                                    debug!("failed to decode event {kind}: {e}");
                                }
                            }
                        }
                    }
                    Ok(Message::Close(frame)) => {
                        debug!("websocket closed by server: {frame:?}");
                        break;
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) | Ok(Message::Binary(_)) | Ok(Message::Frame(_)) => {}
                    Err(e) => {
                        let _ = evt_tx.send(Err(Error::from(e))).await;
                        break;
                    }
                }
            }

            // Fail any in-flight callers immediately when the socket closes.
            let mut map = reader_pending.lock().await;
            for (_, tx) in map.drain() {
                let _ = tx.send(Err(Error::NotConnected));
            }
        });

        Ok((transport, evt_rx))
    }

    /// Sends a request without waiting for an acknowledgment.
    pub async fn fire_and_forget(&self, req: &WsRequest) -> Result<()> {
        let payload = serde_json::to_string(req)?;
        self.outbound
            .send(payload)
            .await
            .map_err(|_| Error::NotConnected)
    }

    /// Sends a request with a generated `rid` and awaits the matching response.
    pub async fn request(&self, req: &WsRequest) -> Result<Value> {
        let rid = Uuid::new_v4().to_string();
        let mut obj = serde_json::to_value(req)?;
        if let Some(map) = obj.as_object_mut() {
            map.insert("rid".to_string(), Value::String(rid.clone()));
        }
        let payload = serde_json::to_string(&obj)?;

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(rid.clone(), tx);

        if self.outbound.send(payload).await.is_err() {
            self.pending.lock().await.remove(&rid);
            return Err(Error::NotConnected);
        }

        match time::timeout(REQUEST_TIMEOUT, rx).await {
            Ok(Ok(res)) => res,
            Ok(Err(_)) => Err(Error::NotConnected),
            Err(_) => {
                self.pending.lock().await.remove(&rid);
                Err(Error::Timeout)
            }
        }
    }
}
