use serde_json::Value;
use serde_json::json;
pub struct Mailbox {
    pub pending: Value,
    pub current: Value,
}

impl Mailbox {
    pub fn merge_requests (&mut self) {
        if self.pending == Value::Null {
            return
        }

        if self.current["initialization"] == Value::Null {
            self.current.as_object_mut().unwrap().insert(String::from("initialization"), self.pending["initialization"].clone());
        }

        if self.current["ack"] == Value::Null {
            self.current.as_object_mut().unwrap().insert(String::from("ack"), self.pending["ack"].clone());
        } 

        let mut current = self.current["messages"].clone().as_array_mut().unwrap().to_owned();
        let mut pending = self.pending["messages"].clone().as_array_mut().unwrap().to_owned();
        pending.append(&mut current);
        self.current.as_object_mut().unwrap().insert(String::from("messages"), json!(pending));
        self.pending = Value::Null;
    }
}