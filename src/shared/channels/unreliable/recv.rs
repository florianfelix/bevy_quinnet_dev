use bevy::utils::tracing::trace;
use bytes::{Buf, Bytes};
use std::{fmt::Display, io::Cursor};
use tokio::sync::{
    broadcast,
    mpsc::{self},
};

use crate::shared::channels::{ChannelId, CHANNEL_ID_LEN};

pub(crate) async fn unreliable_channel_receiver_task<T: Display>(
    task_id: T,
    connection: quinn::Connection,
    mut close_recv: broadcast::Receiver<()>,
    bytes_incoming_send: mpsc::Sender<(ChannelId, Bytes)>,
) {
    tokio::select! {
        _ = close_recv.recv() => {
            trace!("Listener for unreliable datagrams with id {} received a close signal", task_id)
        }
        _ = async {
            while let Ok(msg_bytes) = connection.read_datagram().await {
                if msg_bytes.len() <= CHANNEL_ID_LEN {
                    continue;
                }
                let mut msg = Cursor::new(&msg_bytes);
                let channel_id = msg.get_u8();
                // TODO Clean: error handling
                bytes_incoming_send.send((channel_id, msg_bytes.into())).await.unwrap();
            }
        } => {
            trace!("Listener for unreliable datagrams with id {} ended", task_id)
        }
    };
}
