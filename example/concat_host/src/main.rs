use message_plugins::*;

fn main() {
    use concat_types::ConcatCommand::*;
    let mut host = Host::<concat_types::ConcatCommand>::new();
    let debug_plugin = construct_plugin("../concat_debug_plugin/target/release/libconcat_debug_plugin.dylib").unwrap();
    host.attach(debug_plugin);
    let concat_plugin = construct_plugin("../concat_plugin/target/release/libconcat_plugin.dylib").unwrap();
    host.attach(concat_plugin);
    println!("Hello, world!");
    for command in &[Concat("Hello".to_owned()), Concat(", ".to_owned()), Concat("World".to_owned()), Print] {
        host.send(command.clone());
    }
    for _ in 0usize..1_000_000 {
        host.send(Time(std::time::Instant::now()));
    }
    host.send(Print);
}
