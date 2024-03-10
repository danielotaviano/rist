use crate::object::Object;

pub fn ls_tree(tree_hash: String, name_only: bool) {
    let object = Object::from_hash(tree_hash);
    print!("{}", object.content(Some(name_only)));
}
