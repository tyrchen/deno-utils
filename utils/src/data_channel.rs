use std::task::Poll;

use deno_core::error::AnyError;
use futures::future::poll_fn;
use tokio::sync::oneshot::{self, error::TryRecvError};

pub struct ClientChannel<Req, Res> {
    rx: Option<oneshot::Receiver<Res>>,
    tx: Option<oneshot::Sender<Req>>,
}

pub struct ServerChannel<Req, Res> {
    rx: Option<oneshot::Receiver<Req>>,
    tx: Option<oneshot::Sender<Res>>,
}

pub fn create_client_server_channel<Req, Res>() -> (ClientChannel<Req, Res>, ServerChannel<Req, Res>)
{
    let (tx, rx) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    (
        ClientChannel {
            rx: Some(rx),
            tx: Some(tx2),
        },
        ServerChannel {
            rx: Some(rx2),
            tx: Some(tx),
        },
    )
}

impl<Req, Res> ClientChannel<Req, Res> {
    pub fn send(&mut self, req: Req) -> Result<(), AnyError> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| AnyError::msg("client tx already taken"))?;
        tx.send(req)
            .map_err(|_e| AnyError::msg("Error: failed to send request"))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Res, AnyError> {
        if let Some(rx) = self.rx.as_mut() {
            poll_fn(move |_cx| match rx.try_recv() {
                Ok(res) => Poll::Ready(Ok(res)),
                Err(TryRecvError::Empty) => Poll::Pending,
                _ => Poll::Ready(Err(AnyError::msg("Error: failed to receive response"))),
            })
            .await
        } else {
            Err(AnyError::msg("Error: client rx already taken"))
        }
    }
}

impl<Req, Res> ServerChannel<Req, Res> {
    pub fn take_tx(&mut self) -> Result<oneshot::Sender<Res>, AnyError> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| AnyError::msg("server tx already taken"))?;
        Ok(tx)
    }

    pub fn send(&mut self, res: Res) -> Result<(), AnyError> {
        let tx = self.take_tx()?;
        tx.send(res)
            .map_err(|_e| AnyError::msg("Error: failed to send response"))?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Req, AnyError> {
        let rx = self
            .rx
            .take()
            .ok_or_else(|| AnyError::msg("server rx already taken"))?;

        Ok(rx.await?)
    }
}
