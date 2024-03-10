use crate::object::Object;

pub fn hash_object(path: String, write: bool) {
    let object = Object::from_path(path);

    if write {
        object.write().expect("Error trying to write the object");
    };

    println!("{}", object.hash());
}
