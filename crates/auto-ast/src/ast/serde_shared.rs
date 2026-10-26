use std::cell::RefCell;
use std::rc::Rc;

pub fn serialize<T, S>(value: &Rc<RefCell<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: serde::Serialize,
    S: serde::Serializer,
{
    value.borrow().serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Rc<RefCell<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    T::deserialize(deserializer).map(|value| Rc::new(RefCell::new(value)))
}
