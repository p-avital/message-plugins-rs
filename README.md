# A Message-Passing Oriented plugin architecture
Plugins can be hard, even when restricting them to Rust, they usually require a lot of work to ensure that they are memory-safe, that optionnal parts of the API won't break the host, and of course, that types can cross the FFI interface freely, since Rust doesn't have a stable ABI yet.

This crate attempts to provide an easy bridge between Rust plugins and a host: write your plugin in Rust, ensure that it implements `Plugin<T>` (only one method to implement :D), and build a dll with a single mandatory function: a constructor that stores your boxed plugin into a pointer.

This plugin architecture is built on message passing: the host sends an counted reference to your message to each plugin.

## Why is there a `tokio-host` feature?
The `tokio-host` feature lets you pick a host that relies on `tokio`'s runtime and channels instead of `std`'s threads and channels. Pick whichever fits most into your application, but do keep in mind that parts of the `tokio-host`'s API are `async` functions instead of normal ones, and that you'll need to `.await` them in order to make them have any effect.

## How memory safe is this?
Very: message passing queues what you would typically do nowadays through method calls, ensuring your plugin's only called sequentially by the host: you can freely mutate your state, as long as you don't have concurrency within your plugin.

Meanwhile, the messages are passed through immutable counted references: unless you break the usual rules, you won't be able to break your own messages even though multiple plugins will run concurrently.

## Wait, messages are immutable references, and `handle_message` only returns `Option<u8>`... How do I send data back to my host application?
To do so, you need to give your plugins a channel that they may use to send data back. You're free to pick, but I would suggest `std::sync::mpsc::sync_channel`s

## Is this compatible with other programming languages?
Honestly, not really: the current interface for dynamically loaded plugins is a `Box<dyn Plugin<T>>`, which you would need to emulate in your alternate languages in order to even start making plugins in other languages...

If you want compatibility with other languages, you may need to build a plugin to interface the plugin system with the plugins you may develop in said lanuages. And honestly, at this point, you might as well just develop your own plugin interface...