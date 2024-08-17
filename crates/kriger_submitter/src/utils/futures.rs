use futures::{Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
#[must_use]
pub(crate) struct PollPending<'a, St: ?Sized + Stream> {
    stream: &'a mut St,
    limit: Option<usize>,
    buf: Vec<St::Item>,
}

impl<'a, St: ?Sized + Stream> PollPending<'a, St> {
    pub(crate) fn new(stream: &'a mut St, limit: Option<usize>) -> Self {
        Self {
            stream,
            limit,
            buf: vec![],
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
                    self.buf.push(item);
                    if let Some(limit) = self.limit {
                        if self.buf.len() >= limit {
                            return Poll::Ready(std::mem::take(&mut self.buf));
                        }
                    }
                }
                Poll::Ready(None) | Poll::Pending => {
                    return Poll::Ready(std::mem::take(&mut self.buf))
                }
            }
        }
    }
}
