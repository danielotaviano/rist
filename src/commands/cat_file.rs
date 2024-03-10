use crate::object::Object;

pub fn cat_file(object_hash: String) {
    let object = Object::from_hash(object_hash);

    print!("{}", object.content(None));
}
