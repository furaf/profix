use std::convert::From;

use exchange::Action;

use detail::FixDeserializable;
use FixClient;
use client::MessageValidationErr;

#[derive(Debug)]
pub enum HandleErr {
    MessageValidationErr(MessageValidationErr),
    Unknown,
}

impl From<MessageValidationErr> for HandleErr {
    fn from(e: MessageValidationErr) -> Self {
        HandleErr::MessageValidationErr(e)
    }
}

pub trait FixHandler<SessionMsg: FixDeserializable, AppMsg: FixDeserializable> {
    fn handle_session(&mut self, client: &mut FixClient, msg: SessionMsg) -> Result<(), HandleErr>;
    fn handle_app(&mut self, client: &mut FixClient, msg: AppMsg) -> Result<(), ()>;

    fn handle_action(&mut self, client: &mut FixClient, action: Action);

    fn poll(&mut self, client: &mut FixClient) {}

    fn is_logged(&self) -> bool;
}
