use std::{future::Future, marker::PhantomData};

use async_trait::async_trait;
use bytes::BytesMut;

pub trait CallbackInput {
	fn deserialize(input: &mut BytesMut) -> Self;
}

pub trait CallbackOutput {
	fn serialize(self) -> BytesMut;
}

#[async_trait]
pub trait Callback<Ctx>: Send + Sync {
	async fn call(&self, context: Ctx, input: BytesMut) -> BytesMut;
}

#[derive(Clone)]
pub struct CallbackHolder<F, T>(F, PhantomData<T>);

impl<F, T> CallbackHolder<F, T> {
	pub fn new(callback: F) -> Self {
		Self(callback, PhantomData)
	}
}

#[async_trait]
impl<Ctx, F, Fut, Res, I> Callback<Ctx> for CallbackHolder<F, I>
where
	Ctx: Send + Sync + 'static,
	F: Fn(Ctx, I) -> Fut + Clone + Send + Sync + 'static,
	Fut: Future<Output = Res> + Send,
	Res: CallbackOutput,
	I: CallbackInput + Send + Sync,
{
	async fn call(&self, context: Ctx, mut input: BytesMut) -> BytesMut {
		self.0(context, I::deserialize(&mut input)).await.serialize()
	}
}
