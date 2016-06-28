extern crate futures;

use std::sync::Arc;
use std::sync::mpsc::channel;

use futures::*;

#[test]
fn map() {
    // Whatever runs after a `map` should have dropped the closure by that
    // point.
    let (tx, rx) = channel::<()>();
    let (tx2, rx2) = channel();
    failed::<i32, i32>(1).map(move |a| { drop(tx); a }).map_err(move |_| {
        assert!(rx.recv().is_err());
        tx2.send(()).unwrap()
    }).forget();
    rx2.recv().unwrap();
}

#[test]
fn map_err() {
    // Whatever runs after a `map_err` should have dropped the closure by that
    // point.
    let (tx, rx) = channel::<()>();
    let (tx2, rx2) = channel();
    finished::<i32, i32>(1).map_err(move |a| { drop(tx); a }).map(move |_| {
        assert!(rx.recv().is_err());
        tx2.send(()).unwrap()
    }).forget();
    rx2.recv().unwrap();
}

struct FutureData<F, T> {
    _data: T,
    future: F,
}

impl<F, T, U, V> Future<U, V> for FutureData<F, T>
    where F: Future<U, V>,
          T: Send + 'static,
          U: Send + 'static,
          V: Send + 'static,
{
    fn poll(&mut self, tokens: &Tokens) -> Option<PollResult<U, V>> {
        self.future.poll(tokens)
    }

    fn schedule(&mut self, wake: Arc<Wake>) -> Tokens {
        self.future.schedule(wake)
    }

    fn tailcall(&mut self) -> Option<Box<Future<U, V>>> {
        None
    }
}

#[test]
fn and_then_drops_eagerly() {
    let (p, c) = promise::<(), ()>();
    let (tx, rx) = channel::<()>();
    let (tx2, rx2) = channel();
    FutureData { _data: tx, future: p }.and_then(move |_| {
        assert!(rx.recv().is_err());
        tx2.send(()).unwrap();
        finished(1)
    }).forget();
    assert!(rx2.try_recv().is_err());
    c.finish(());
    rx2.recv().unwrap();
}

// #[test]
// fn or_else_drops_eagerly() {
//     let (p1, c1) = promise::<(), ()>();
//     let (p2, c2) = promise::<(), ()>();
//     let (tx, rx) = channel::<()>();
//     let (tx2, rx2) = channel();
//     p1.map(move |a| { drop(tx); a }).or_else(move |_| {
//         assert!(rx.recv().is_err());
//         p2
//     }).map(move |_| tx2.send(()).unwrap()).forget();
//     assert!(rx2.try_recv().is_err());
//     c1.fail(());
//     assert!(rx2.try_recv().is_err());
//     c2.finish(());
//     rx2.recv().unwrap();
// }
