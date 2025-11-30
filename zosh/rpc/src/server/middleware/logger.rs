//! Logger middleware

use jsonrpsee::{
    core::middleware::{Batch, Notification},
    server::middleware::rpc::RpcServiceT,
    types::Request,
};
use std::future::Future;

/// Logging service
#[derive(Clone)]
pub struct Logger<S>(pub S);

impl<S> RpcServiceT for Logger<S>
where
    S: RpcServiceT + Send + Sync + Clone + 'static,
{
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn batch<'a>(&self, batch: Batch<'a>) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        self.0.batch(batch)
    }

    fn call<'a>(
        &self,
        request: Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        tracing::debug!("{:?}", request);
        self.0.call(request)
    }

    fn notification<'a>(
        &self,
        n: Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.0.notification(n)
    }
}
