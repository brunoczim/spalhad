use anyhow::{Result, bail};
use tokio::{
    select,
    sync::{mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;

use spalhad_task::TaskManager;

pub use spalhad_actor_macros::CallSuperset;

pub trait CallSuperset {
    fn reply_error<E>(self, error: E) -> bool
    where
        E: Into<anyhow::Error>;
}

pub trait CallInjection<C: CallConnectors>: CallSuperset + Sized {
    fn inject(call: C) -> Self;
}

impl<I, O> CallInjection<Self> for ActorCall<I, O> {
    fn inject(call: Self) -> Self {
        call
    }
}

pub trait CallConnectors {
    type Input;
    type Output;
}

impl<I, O> CallConnectors for ActorCall<I, O> {
    type Input = I;
    type Output = O;
}

#[trait_variant::make(Send)]
pub trait Actor {
    type Call;

    async fn start(
        self,
        inbox: ActorInbox<Self::Call>,
        cancellation_token: CancellationToken,
    ) -> Result<()>;
}

#[trait_variant::make(Send)]
pub trait TrivialLoopActor {
    type Call;

    async fn on_call(&mut self, call: Self::Call) -> Result<()>;
}

impl<T> Actor for T
where
    T: TrivialLoopActor,
    T::Call: Send,
{
    type Call = T::Call;

    async fn start(
        mut self,
        mut inbox: ActorInbox<Self::Call>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        loop {
            let result = select! {
                _ = cancellation_token.cancelled() => break Ok(()),
                message = inbox.recv() => message,
            };
            let Some(call) = result else { break Ok(()) };
            self.on_call(call).await?;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActorOptions<'a> {
    task_manager: &'a TaskManager,
    channel_size: usize,
}

impl<'a> ActorOptions<'a> {
    pub fn new(task_manager: &'a TaskManager) -> Self {
        Self { task_manager, channel_size: 10 }
    }

    pub fn set_channel_size(&mut self, size: usize) -> &mut Self {
        self.channel_size = size;
        self
    }

    pub fn with_channel_size(mut self, size: usize) -> Self {
        self.set_channel_size(size);
        self
    }

    pub fn spawn<A>(&self, actor: A) -> ActorHandle<A::Call>
    where
        A: Actor + Send + 'static,
        A::Call: Send + 'static,
    {
        let (sender, receiver) = mpsc::channel(self.channel_size);
        let handle = ActorHandle { inner: sender };
        let inbox = ActorInbox { inner: receiver };
        let cancellation_token = self.task_manager.cancellation_token();
        let task = async move { actor.start(inbox, cancellation_token).await };
        self.task_manager.spawn(task);
        handle
    }
}

#[derive(Debug)]
pub struct ActorHandle<M> {
    inner: mpsc::Sender<M>,
}

impl<M> ActorHandle<M> {
    pub async fn send<I, O>(&self, input: I) -> Result<O>
    where
        M: CallInjection<ActorCall<I, O>>,
    {
        let (sender, receiver) = oneshot::channel();
        let callback = ActorCallback { sender };
        let call = ActorCall { input, back: callback };
        self.forward(M::inject(call)).await?;
        receiver.await?
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
pub struct ActorCallback<O> {
    sender: oneshot::Sender<Result<O>>,
}

impl<O> ActorCallback<O> {
    pub fn reply(self, output: Result<O>) -> bool {
        let success = self.sender.send(output).is_ok();
        if !success {
            tracing::warn!("caller has closed");
        }
        success
    }

    pub fn reply_ok(self, output: O) -> bool {
        self.reply(Ok(output))
    }
}

impl<O> CallSuperset for ActorCallback<O> {
    fn reply_error<E>(self, error: E) -> bool
    where
        E: Into<anyhow::Error>,
    {
        self.reply(Err(error.into()))
    }
}

#[derive(Debug)]
pub struct ActorCall<I, O> {
    pub input: I,
    pub back: ActorCallback<O>,
}

impl<I, O> ActorCall<I, O> {
    pub async fn handle<F, A>(self, handler: F) -> bool
    where
        F: FnOnce(I) -> A,
        A: Future<Output = Result<O>>,
    {
        let output = handler(self.input).await;
        self.back.reply(output)
    }
}

impl<I, O> CallSuperset for ActorCall<I, O> {
    fn reply_error<E>(self, error: E) -> bool
    where
        E: Into<anyhow::Error>,
    {
        self.back.reply_error(error)
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
