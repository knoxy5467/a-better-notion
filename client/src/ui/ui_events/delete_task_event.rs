pub struct CallbackUser {
    callbacks_vec: Vec<Box<FnMut()>>,
}
impl Callbacks for CallbackUser {
    fn add_callback_to_delete(f: T) {
        self.callbacks_vec.push(Box::new(f))
    }
    fn on_event() {
        for mut c in self.callbacks_vec.drain() {
            c();
        }
    }
}
