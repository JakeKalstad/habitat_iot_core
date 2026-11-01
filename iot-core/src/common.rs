use std::collections::HashMap;
pub const MAX_POST_LEN: usize = 256;

pub fn parse_form_data(body: &str) -> HashMap<String, String> {
    // Parse the form data from the request body
    let mut form_data = HashMap::new();
    for pair in body.split('&') {
        let mut iter = pair.split('=');
        if let (Some(key), Some(value)) = (iter.next(), iter.next()) {
            form_data.insert(key.to_string(), value.to_string());
        }
    }
    form_data
}


pub struct ScopeCall<F: FnMut()> {
   pub c: F,
}
impl<F: FnMut()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        (self.c)();
    }
}
