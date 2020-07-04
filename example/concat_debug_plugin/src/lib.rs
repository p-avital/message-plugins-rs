use concat_types::*;
use message_plugins::*;

pub struct DebugPlugin {}

impl Plugin<ConcatCommand> for DebugPlugin {
    fn handle_message(&mut self, message: Message<ConcatCommand>) -> std::option::Option<u8> {
        if let ConcatCommand::Time(_) = message.as_ref() {
            return None
        }
        if &ConcatCommand::Exit == dbg!(message.as_ref()) {
            Some(0)
        } else {
            None
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn plugin_constructor(plugin: *mut Box<dyn Plugin<ConcatCommand>>) {
    insert_instance(plugin, Box::new(DebugPlugin {}));
}
