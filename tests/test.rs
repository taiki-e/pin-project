#![feature(futures_api)]
#![deny(warnings)]

use pin_project::{unsafe_fields, unsafe_project};
use std::marker::Unpin;
use std::pin::Pin;

#[test]
fn test_unsafe_project() {
    #[unsafe_project]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let foo = Pin::new(&mut foo).project();

    let x: Pin<&mut i32> = foo.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.field2;
    assert_eq!(*y, 2);
}

#[test]
fn test_unsafe_fields() {
    #[unsafe_fields]
    struct Foo<T, U> {
        #[pin]
        field1: T,
        field2: U,
    }

    impl<T: Unpin, U> Unpin for Foo<T, U> {} // Conditional Unpin impl

    let mut foo = Foo {
        field1: 1,
        field2: 2,
    };

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().field2();
    assert_eq!(*y, 2);

    #[unsafe_fields]
    struct Bar<T, U> {
        #[pin]
        field1: T,
        field2: U,
        #[skip]
        _field3: (),
    }

    impl<T: Unpin, U> Unpin for Bar<T, U> {} // Conditional Unpin impl

    let mut foo = Bar {
        field1: 1,
        field2: 2,
        _field3: (),
    };

    let mut foo = Pin::new(&mut foo);

    let x: Pin<&mut i32> = foo.as_mut().field1();
    assert_eq!(*x, 1);

    let y: &mut i32 = foo.as_mut().field2();
    assert_eq!(*y, 2);

    // let _z = foo.as_mut()._field3(); // ERROR
}

mod examples {
    #![allow(dead_code)]

    // These codes copied from https://github.com/Pauan/rust-signals/blob/master/src/future.rs

    // MIT License

    // Copyright (c) 2018

    // Permission is hereby granted, free of charge, to any person obtaining a copy
    // of this software and associated documentation files (the "Software"), to deal
    // in the Software without restriction, including without limitation the rights
    // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    // copies of the Software, and to permit persons to whom the Software is
    // furnished to do so, subject to the following conditions:

    // The above copyright notice and this permission notice shall be included in all
    // copies or substantial portions of the Software.

    // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    // SOFTWARE.

    use std::{
        future::Future,
        marker::Unpin,
        pin::Pin,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
        task::{LocalWaker, Poll, Waker},
    };

    #[derive(Debug)]
    struct CancelableFutureState {
        is_cancelled: AtomicBool,
        waker: Mutex<Option<Waker>>,
    }

    mod unsafe_project {
        use super::*;
        use pin_project::unsafe_project;

        #[unsafe_project]
        #[derive(Debug)]
        #[must_use = "Futures do nothing unless polled"]
        pub struct CancelableFuture<A, B> {
            state: Arc<CancelableFutureState>,
            #[pin]
            future: Option<A>,
            when_cancelled: Option<B>,
        }

        impl<A, B> Unpin for CancelableFuture<A, B> where A: Unpin {}

        impl<A, B> Future for CancelableFuture<A, B>
        where
            A: Future,
            B: FnOnce() -> A::Output,
        {
            type Output = A::Output;

            fn poll(self: Pin<&mut Self>, waker: &LocalWaker) -> Poll<Self::Output> {
                let this = self.project();
                if this.state.is_cancelled.load(Ordering::SeqCst) {
                    Pin::set(this.future, None);
                    let callback = this.when_cancelled.take().unwrap();
                    Poll::Ready(callback())
                } else {
                    match this.future.as_pin_mut().unwrap().poll(waker) {
                        Poll::Pending => {
                            *this.state.waker.lock().unwrap() = Some(waker.clone().into_waker());
                            Poll::Pending
                        }
                        a => a,
                    }
                }
            }
        }
    }

    mod unsafe_fields {
        use super::*;
        use pin_project::unsafe_fields;

        #[unsafe_fields]
        #[derive(Debug)]
        #[must_use = "Futures do nothing unless polled"]
        pub struct CancelableFuture<A, B> {
            state: Arc<CancelableFutureState>,
            #[pin]
            future: Option<A>,
            when_cancelled: Option<B>,
        }

        impl<A, B> Unpin for CancelableFuture<A, B> where A: Unpin {}

        impl<A, B> Future for CancelableFuture<A, B>
        where
            A: Future,
            B: FnOnce() -> A::Output,
        {
            type Output = A::Output;

            fn poll(mut self: Pin<&mut Self>, waker: &LocalWaker) -> Poll<Self::Output> {
                if self.as_mut().state().is_cancelled.load(Ordering::SeqCst) {
                    Pin::set(self.as_mut().future(), None);
                    let callback = self.when_cancelled().take().unwrap();
                    Poll::Ready(callback())
                } else {
                    match self.as_mut().future().as_pin_mut().unwrap().poll(waker) {
                        Poll::Pending => {
                            *self.state().waker.lock().unwrap() = Some(waker.clone().into_waker());
                            Poll::Pending
                        }
                        a => a,
                    }
                }
            }
        }
    }
}
