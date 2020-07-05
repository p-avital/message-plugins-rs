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


use std::any::Any;
#[no_mangle]
pub unsafe extern "C" fn plugin_constructor(plugin: *mut Box<dyn Plugin<ConcatCommand>>, args: Option<&dyn Any>) {
    dbg!(&args);
    if let Some(args) = args {
        if let Some(map) = args.downcast_ref::<std::collections::HashMap<&str, &str>>() {
            println!("Debug args: {:?}", map);
            insert_instance(plugin, Box::new(DebugPlugin {}));
        }
    }
}