pub(crate) fn assert_send<T: Send>() {}
pub(crate) fn assert_sync<T: Send>() {}

pub(crate) struct NotSendSync(*const ());
