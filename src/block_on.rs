use futures::Future;

pub fn block_on<T: Future>(t: T) -> <T as futures::Future>::Output {
    tokio::runtime::Runtime::new().unwrap().block_on(t)
}