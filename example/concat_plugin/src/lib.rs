use concat_types::*;
use message_plugins::*;

pub struct ConcatPlugin {
    data: String,
    times: Vec<std::time::Duration>,
}

impl Plugin<ConcatCommand> for ConcatPlugin {
    fn handle_message(&mut self, message: Message<ConcatCommand>) -> std::option::Option<u8> {
        match message.as_ref() {
            ConcatCommand::Exit => return Some(0),
            ConcatCommand::Clear => self.data.clear(),
            ConcatCommand::Concat(string) => self.data += string,
            ConcatCommand::Time(instant) => {
                self.times.push(instant.elapsed());
                // println!("{}", self.times.len())
            }
            ConcatCommand::Print => {
                dbg!(&self.data);
                // println!("{:?}", &self.times);
                println!("Average delay: {}", self.avg_delay())
            }
        }
        None
    }
}

impl ConcatPlugin {
    fn avg_delay(&self) -> f64 {
        self.times.iter().fold(0., |acc, it| acc + it.as_secs_f64()) / self.times.len() as f64
    }
}

#[no_mangle]
pub unsafe extern "C" fn plugin_constructor(plugin: *mut Box<dyn Plugin<ConcatCommand>>) {
    insert_instance(
        plugin,
        Box::new(ConcatPlugin {
            data: Default::default(),
            times: Default::default(),
        }),
    );
}
