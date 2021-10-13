use crossbeam_channel::{Sender, Receiver};

/// Function that receives the output of a [`RemoteRequest`]
pub type RemoteResponse<R> = Box<dyn Fn(&mut R) + Send + Sync>;

/// Function that remotely controls an instance of type T using some context C
pub type RemoteRequest<T, C, R> = Box<dyn Fn(&mut T, &C) -> RemoteResponse<R> + Send + Sync>;

/// Wrapper type for cross-thread remote control over an instance of type T
pub struct RemoteResponder<T, C, R> {
    rx: Receiver<RemoteRequest<T, C, R>>,
    tx: Sender<RemoteResponse<R>>,
    inner: T,
}

impl<T, C, R> RemoteResponder<T, C, R> {
    pub fn receive_requests(&mut self, context: &C) {
        for request in self.rx.try_iter() {
            let response = request(&mut self.inner, context);
            self.send_response(response);
        }
    }

    pub fn send_response(&self, response: RemoteResponse<R>) {
        self.tx.send(response).unwrap()
    }
}

impl<T, C, R> std::ops::Deref for RemoteResponder<T, C, R> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, C, R> std::ops::DerefMut for RemoteResponder<T, C, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Foreign-thread control interface for a [`RemoteResponder`]
pub struct RemoteRequester<T, C, R> {
    tx: Sender<RemoteRequest<T, C, R>>,
    rx: Receiver<RemoteResponse<R>>,
}

impl<T, C, R> RemoteRequester<T, C, R> {
    pub fn send_request(&self, request: RemoteRequest<T, C, R>) {
        self.tx.send(request).unwrap()
    }

    pub fn receive_responses(&self, context: &mut R) {
        for response in self.rx.try_iter() {
            response(context)
        }
    }
}

/// Constructs a requester-responder pair for remotely addressing a type across threads
pub fn remote_channel<T, C, R>(inner: T) -> (RemoteRequester<T, C, R>, RemoteResponder<T, C, R>) {
    let (request_tx, request_rx) = crossbeam_channel::unbounded();
    let (response_tx, response_rx) = crossbeam_channel::unbounded();

    (
        RemoteRequester {
            tx: request_tx,
            rx: response_rx,
        },
        RemoteResponder {
            rx: request_rx,
            tx: response_tx,
            inner,
        },
    )
}

