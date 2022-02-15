use async_trait::async_trait;
mod v12;
pub mod v11;

pub(crate) trait Parse<T> {
    fn parse(self) -> T;
}

#[async_trait]
pub(crate) trait Parser<X, Y> {
    async fn parse(&self, input: X) -> Option<Y>;
}
