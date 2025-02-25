use anyhow::{Result, bail};
use tokio::sync::{mpsc, oneshot};

pub fn channel<M>(buf_size: usize) -> (ActorHandle<M>, ActorInbox<M>) {
    let (sender_inner, receiver_inner) = mpsc::channel(buf_size);
    let sender = ActorHandle { inner: sender_inner };
    let receiver = ActorInbox { inner: receiver_inner };
    (sender, receiver)
}

#[derive(Debug)]
pub struct ActorHandle<M> {
    inner: mpsc::Sender<M>,
}

impl<M> ActorHandle<M> {
    pub async fn send<I, O>(&self, input: I) -> Result<O>
    where
        M: From<ActorCall<I, O>>,
    {
        let (callback_sender, callback_receiver) = oneshot::channel();
        let call = ActorCall { input, callback: callback_sender };
        self.forward(call).await?;
        callback_receiver.await?
    }

    pub async fn forward<C>(&self, call: C) -> Result<()>
    where
        M: From<C>,
    {
        if self.inner.send(call.into()).await.is_err() {
            tracing::warn!("callee has closed");
            bail!("callee actor disconnected");
        }
        Ok(())
    }
}

impl<M> Clone for ActorHandle<M> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

#[derive(Debug)]
pub struct ActorCall<I, O> {
    input: I,
    callback: oneshot::Sender<Result<O>>,
}

impl<I, O> ActorCall<I, O> {
    pub fn input(&self) -> &I {
        &self.input
    }

    pub async fn handle<F, A>(self, handler: F) -> bool
    where
        F: FnOnce(I) -> A,
        A: Future<Output = Result<O>>,
    {
        let output = handler(self.input).await;
        let success = self.callback.send(output).is_ok();
        if !success {
            tracing::warn!("caller has closed");
        }
        success
    }
}

#[derive(Debug)]
pub struct ActorInbox<M> {
    inner: mpsc::Receiver<M>,
}

impl<M> ActorInbox<M> {
    pub async fn recv(&mut self) -> Option<M> {
        self.inner.recv().await
    }
}
