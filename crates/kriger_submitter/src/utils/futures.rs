use futures::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
#[must_use]
pub(crate) struct PollPending<'a, St: ?Sized + Stream> {
    stream: &'a mut St,
    limit: usize,
    buf: Option<Vec<St::Item>>,
}

impl<'a, St: ?Sized + Stream> PollPending<'a, St> {
    pub(crate) fn new(stream: &'a mut St, limit: usize) -> Self {
        Self {
            stream,
            limit,
            buf: Some(Vec::new()),
        }
    }
}

impl<St: ?Sized + Stream + Unpin> Unpin for PollPending<'_, St> {}

impl<St: ?Sized + Stream + Unpin> Future for PollPending<'_, St> {
    type Output = Vec<St::Item>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            // This may panic if it's polled after Ready has been returned, but /shrug
            // TERMINATION: each iteration either decreases self.limit - buf.len() or returns
            match self.stream.poll_next_unpin(cx) {
                Poll::Ready(Some(item)) => {
                    let buf = self.buf.as_mut().unwrap();
                    buf.push(item);
                    if buf.len() >= self.limit {
                        return Poll::Ready(self.buf.take().unwrap());
                    }
                }
                Poll::Ready(None) | Poll::Pending => return Poll::Ready(self.buf.take().unwrap()),
            }
        }
    }
}
