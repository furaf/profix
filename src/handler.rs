use exchange::Action;

use detail::FixDeserializable;
use FixClient;

pub trait FixHandler<SessionMsg : FixDeserializable, AppMsg : FixDeserializable> {
    fn handle_session(&mut self, client : &mut FixClient, msg : SessionMsg) -> Result<(),(&'static str)>;
    fn handle_app(&mut self, client :  &mut FixClient, msg : AppMsg) -> Result<(),()>;

    fn handle_action(&mut self, client : &mut FixClient, action : Action) ;

    fn is_logged(&self) -> bool;
}