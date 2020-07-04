use std::{ffi::OsStr, sync::Arc};

#[cfg(not(feature = "tokio-host"))]
pub use std_runtime::*;
#[cfg(feature = "tokio-host")]
pub use tokio_runtime::*;

#[cfg(feature = "tokio-host")]
pub mod tokio_runtime {
    use super::*;
    use tokio::sync::mpsc;
    /// Meant as the main way of sending commands to plugins.
    /// A structure that holds the input ends of the queues to each plugin,
    /// as well as the `JoinHandle`s to their tasks.
    pub struct Host<T> {
        plugins: Vec<mpsc::Sender<Message<T>>>,
        pub tasks: Vec<tokio::task::JoinHandle<Option<u8>>>,
    }

    impl<T: Sync + Send + 'static> Host<T> {
        pub fn new() -> Self {
            Host {
                plugins: Vec::new(),
                tasks: Vec::new(),
            }
        }

        /// By default, plugins will communicate with the host using a queue capable of holding this many `Message`s.
        pub const DEFAULT_CHANNEL_CAPACITY: usize = 4;

        /// Sends a message to all the attached `Plugin`s.
        pub async fn send(&mut self, message: impl Into<Message<T>>) {
            let message = message.into();
            futures::future::join_all(
                self.plugins
                    .iter_mut()
                    .map(|plugin| plugin.send(message.clone())),
            )
            .await;
        }

        /// Enables a `Plugin` by attaching it to the `Host`: a channel is built, the input is given to the host;
        /// a task running the `Plugin`'s `handle_message` method on every `Message` sent over the channel is spawned, and the `JoinHandle` to this task is added to the host's handles.
        pub async fn attach(&mut self, plugin: impl Plugin<T>) {
            self.attach_with_capacity(plugin, Self::DEFAULT_CHANNEL_CAPACITY)
                .await
        }

        /// Like `attach`, but allows you to choose your own channel capacity.
        pub async fn attach_with_capacity(&mut self, mut plugin: impl Plugin<T>, capacity: usize) {
            let (tx, mut rx) = mpsc::channel(capacity);
            self.plugins.push(tx);
            self.tasks.push(tokio::spawn(async move {
                while let Some(message) = rx.recv().await {
                    if let Some(status) = plugin.handle_message(message) {
                        return Some(status);
                    }
                }
                None
            }))
        }

        /// Drops every channel end, closing them, then waits for all plugins to finish processing the remaining messages.
        pub fn end(&mut self) -> futures::future::JoinAll<tokio::task::JoinHandle<Option<u8>>> {
            self.plugins.clear();
            futures::future::join_all(self.tasks.drain(..))
        }
    }
}

#[cfg(not(feature = "tokio-host"))]
pub mod std_runtime {
    use super::*;
    use std::sync::mpsc;
    /// Meant as the main way of sending commands to plugins.
    /// A structure that holds the input ends of the queues to each plugin,
    /// as well as the `JoinHandle`s to their tasks.
    pub struct Host<T> {
        plugins: Vec<mpsc::SyncSender<Message<T>>>,
        pub tasks: Vec<std::thread::JoinHandle<Option<u8>>>,
    }

    impl<T> Drop for Host<T> {
        #[allow(unused_must_use)]
        fn drop(&mut self) {
            self.plugins.clear();
            for task in self.tasks.drain(..) {
                task.join();
            }
        }
    }

    impl<T: Sync + Send + 'static> Host<T> {
        pub fn new() -> Self {
            Host {
                plugins: Vec::new(),
                tasks: Vec::new(),
            }
        }

        /// By default, plugins will communicate with the host using a queue capable of holding this many `Message`s.
        pub const DEFAULT_CHANNEL_CAPACITY: usize = 4;

        /// Sends a message to all the attached `Plugin`s.
        #[allow(unused_must_use)]
        pub fn send(&mut self, message: impl Into<Message<T>>) {
            let message = message.into();
            for plugin in self.plugins.iter() {
                plugin.send(message.clone());
            }
        }

        /// Enables a `Plugin` by attaching it to the `Host`: a channel is built, the input is given to the host;
        /// a task running the `Plugin`'s `handle_message` method on every `Message` sent over the channel is spawned, and the `JoinHandle` to this task is added to the host's handles.
        pub fn attach(&mut self, plugin: impl Plugin<T>) {
            self.attach_with_capacity(plugin, Self::DEFAULT_CHANNEL_CAPACITY)
        }

        /// Like `attach`, but allows you to choose your own channel capacity.
        pub fn attach_with_capacity(&mut self, mut plugin: impl Plugin<T>, capacity: usize) {
            let (tx, rx) = mpsc::sync_channel(capacity);
            self.plugins.push(tx);
            self.tasks.push(std::thread::spawn(move || {
                while let Ok(message) = rx.recv() {
                    if let Some(status) = plugin.handle_message(message) {
                        return Some(status);
                    }
                }
                None
            }))
        }

        /// Drops every channel end, closing them, then waits for all plugins to finish processing the remaining messages.
        pub fn end(&mut self) -> Vec<std::thread::Result<Option<u8>>> {
            self.plugins.clear();
            self.tasks.drain(..).map(|t| t.join()).collect()
        }
    }
}

/// Represents a single message to be sent to every plugin.
pub struct Message<T> {
    pub content: Arc<T>,
}

impl<T> AsRef<T> for Message<T> {
    fn as_ref(&self) -> &T {
        self.content.as_ref()
    }
}

impl<T> Clone for Message<T> {
    fn clone(&self) -> Self {
        Message {
            content: self.content.clone(),
        }
    }
}

impl<T> Message<T> {
    pub fn new(value: T) -> Self {
        Message {
            content: Arc::new(value),
        }
    }
}

impl<T> From<Arc<T>> for Message<T> {
    fn from(content: Arc<T>) -> Self {
        Message { content }
    }
}

impl<T> From<T> for Message<T> {
    fn from(content: T) -> Self {
        Message {
            content: Arc::new(content),
        }
    }
}

/// In this architectures, plugins are purely slaves: they simply react to messages.
/// Their only way of returning information by default is by returning Some(status) to signal that they wish to be dropped.
/// If you want your plugin to be able to communicate back to your application after some of your messages, you should hand them a channel to do so through your message type.
pub trait Plugin<T>: Sync + Send + 'static {
    fn handle_message(&mut self, message: Message<T>) -> Option<u8>;
}

/// Loads a dynamic library at `path`, and calls the function called `constructor` in order to instanciate a `Plugin`.
/// The constructor function is the only function where you need to dirty your hands with `extern "C"`. Its sole purpose is to insert your boxed plugin into a pointer.
/// I suggest writing a constructor of the style:
/// ```rust
/// #[no_mangle]
/// unsafe extern "C" fn plugin_constructor(ptr: *mut Box<dyn Plugin<YourMessageType>>) {
///     let plugin = Box::new(YourPlugin::new());
///     insert_instace(ptr, plugin);
/// }
/// ```
pub fn construct_plugin_with_constructor<T>(
    path: impl AsRef<OsStr>,
    constructor: impl AsRef<[u8]>,
) -> Result<Box<dyn Plugin<T>>, libloading::Error> {
    let lib = libloading::Library::new(path)?;
    let mut instance = std::mem::MaybeUninit::uninit();
    Ok(unsafe {
        lib.get::<FfiPluginInit<T>>(constructor.as_ref())?(instance.as_mut_ptr());
        instance.assume_init()
    })
}

/// A default for `construct_plugin_with_constructor`, which will call a function named `plugin_constructor`.
/// The constructor function is the only function where you need to dirty your hands with `extern "C"`. Its sole purpose is to insert your boxed plugin into a pointer.
/// I suggest writing a constructor of the style:
/// ```rust
/// #[no_mangle]
/// unsafe extern "C" fn plugin_constructor(ptr: *mut Box<dyn Plugin<YourMessageType>>) {
///     let plugin = Box::new(YourPlugin::new());
///     insert_instace(ptr, plugin);
/// }
/// ```
pub fn construct_plugin<T>(
    path: impl AsRef<OsStr>,
) -> Result<Box<dyn Plugin<T>>, libloading::Error> {
    construct_plugin_with_constructor(path, b"plugin_constructor")
}

/// Inserts a plugin into an uninitialized pointer, preventing the drop on the uninitialized memory that would happen with a simple assignment
pub fn insert_instance<T>(ptr: *mut Box<dyn Plugin<T>>, mut plugin: Box<dyn Plugin<T>>) {
    unsafe { std::mem::swap(&mut plugin, &mut *ptr) };
    std::mem::forget(plugin);
}

impl<T: 'static, B: AsMut<dyn Plugin<T>> + Sync + Send + 'static> Plugin<T> for B {
    fn handle_message(&mut self, message: Message<T>) -> Option<u8> {
        self.as_mut().handle_message(message)
    }
}

pub type FfiPluginInit<T> = unsafe extern "C" fn(*mut Box<dyn Plugin<T>>);
