use crate::pubsub::PubSubEvent;
use crate::decl::Widget;
use tokio::sync::mpsc;

pub trait PubSubWidget {
    fn on(&mut self,event : PubSubEvent );
    fn set_publish_channel(&mut self,channel : mpsc::Sender<PubSubEvent>);
    fn new(props:Widget) -> Self;
}
pub mod gauge;
pub mod status;