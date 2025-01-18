use std::{
	future::Future,
	pin::pin,
	sync::Arc,
	task::{Context, Poll, Wake},
	thread,
};

struct ThreadWaker(thread::Thread);

impl Wake for ThreadWaker {
	fn wake(self: Arc<Self>) {
		self.0.unpark()
	}
}

pub fn block_on<R, F: Future<Output = R>>(fut: F) -> R {
	let mut f = pin!(fut);

	let waker = Arc::new(ThreadWaker(thread::current())).into();
	let mut ctx = Context::from_waker(&waker);

	loop {
		match f.as_mut().poll(&mut ctx) {
			Poll::Ready(r) => break r,
			Poll::Pending => thread::park(),
		}
	}
}
