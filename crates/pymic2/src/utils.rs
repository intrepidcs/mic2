/// Utility Functions and objects for this crate.
/// Create a thread safe python object (struct).
///
/// Note: use `self.0.lock().unwrap()` to obtain the
/// inner object since its wrapped in an Arc<Mutex>.
///
/// # Arguments
/// * `$name:ident` - Name of the struct visible in Rust.
///
/// * `$python_name` - Name of the object visible to Python.
///
/// * `$inner_name:ty` - name of the transparent inner struct we are trying to wrap.
///
/// # Example
/// ```no_run
/// create_python_object!(PyMyStruct, "MyStruct", MyStruct)
/// ```
macro_rules! create_python_object {
    ($name:ident, $python_name:literal, $inner_name:ty) => {
        #[pyclass(name=$python_name)]
        #[repr(transparent)]
        #[derive(Debug, Default, Clone)]
        pub struct $name(pub Arc<Mutex<$inner_name>>);

        // Arc is only Send + Sync if T is Send + Sync so lets mark it as safe here
        // This is safe because we control access through the Mutex
        unsafe impl Send for $name {}
        unsafe impl Sync for $name {}
    };
}
pub(crate) use create_python_object;
