use crate::pubsub::PubSubEvent;
use crate::decl::Widget;
pub trait PubSubWidget {
    fn on(event : PubSubEvent );
    fn new(props:Widget);
}
pub mod gauge;