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
        #[derive(Debug, Default, Clone)]
        #[repr(transparent)]
        pub struct $name(pub Arc<Mutex<$inner_name>>);

        // Arc is only Send if T is Send so lets mark it as safe here
        unsafe impl Send for $name {}
    };
}
pub(crate) use create_python_object;